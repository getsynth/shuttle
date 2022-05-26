use core::default::Default;
use std::collections::HashMap;
use std::fs::DirEntry;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::mpsc::SyncSender;
use std::sync::Arc;

use anyhow::{anyhow, Context as AnyhowContext};
use chrono::{DateTime, Utc};
use futures::prelude::*;
use libloading::Library;
use rocket::data::ByteUnit;
use rocket::{tokio, Data};
use shuttle_common::project::ProjectName;
use shuttle_common::{
    DeploymentApiError, DeploymentId, DeploymentMeta, DeploymentStateMeta, Host, LogItem, Port,
};
use shuttle_service::loader::Loader;
use shuttle_service::logger::Log;
use shuttle_service::ServeHandle;
use tokio::sync::{mpsc, RwLock};

use crate::build::Build;
use crate::router::Router;
use crate::{database, BuildSystem, ShuttleFactory};

// This controls the maximum number of deploys an api instance can run
// This is mainly needed because tokio::task::spawn_blocking keeps an internal pool for the number of blocking threads
// and we call this method to run each incoming service. Therefore, this variable directly maps to this maximum pool
// when the runtime is setup in main()
// The current tokio default for this pool is 512
// https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html#method.max_blocking_threads
pub const MAX_DEPLOYS: usize = 512;

/// Inner struct of a deployment which holds the deployment itself
/// and the some metadata
pub(crate) struct Deployment {
    meta: Arc<RwLock<DeploymentMeta>>,
    state: RwLock<DeploymentState>,
}

impl Deployment {
    fn new(meta: DeploymentMeta, state: DeploymentState) -> Self {
        Self {
            meta: Arc::new(RwLock::new(meta)),
            state: RwLock::new(state),
        }
    }

    fn from_bytes(fqdn: &str, project: ProjectName, crate_bytes: Vec<u8>) -> Self {
        Self {
            meta: Arc::new(RwLock::new(DeploymentMeta::queued(fqdn, project))),
            state: RwLock::new(DeploymentState::queued(crate_bytes)),
        }
    }

    /// Initialise a deployment from a directory
    fn from_directory(fqdn: &str, dir: DirEntry) -> Result<Self, anyhow::Error> {
        let project_path = dir.path();
        let project_name = dir
            .file_name()
            .into_string()
            .map_err(|err| anyhow!("invalid project name: {:?}", err))
            .and_then(|name| name.parse::<ProjectName>().map_err(|err| err.into()))?;
        // find marker which points to so file
        let marker_path = project_path.join(".shuttle_marker");
        let so_path_str = std::fs::read(&marker_path).context(anyhow!(
            "could not find so marker file at {:?}",
            marker_path
        ))?;

        let so_path: PathBuf = String::from_utf8_lossy(&so_path_str)
            .parse()
            .context("could not parse contents of marker file to a valid path")?;

        let meta = DeploymentMeta::built(fqdn, project_name);
        let state = DeploymentState::built(Build { so_path });
        Ok(Self::new(meta, state))
    }

    /// Gets a `clone`ed copy of the metadata.
    pub(crate) async fn meta(&self) -> DeploymentMeta {
        trace!("trying to get meta");
        self.meta.read().await.clone()
    }

    pub(crate) async fn deployment_active(&self) -> bool {
        matches!(*self.state.read().await, DeploymentState::Deployed(_))
    }

    /// Evaluates if the deployment can be advanced. If the deployment
    /// has reached a state where it can no longer `advance`, returns `false`.
    pub(crate) async fn deployment_finished(&self) -> bool {
        match *self.state.read().await {
            DeploymentState::Queued(_) | DeploymentState::Built(_) | DeploymentState::Loaded(_) => {
                false
            }
            DeploymentState::Deployed(_) | DeploymentState::Error(_) | DeploymentState::Deleted => {
                true
            }
        }
    }

    /// Tries to advance the deployment one stage. Does nothing if the deployment
    /// is in a terminal state.
    pub(crate) async fn advance(
        &self,
        context: &Context,
        db_context: &database::Context,
        run_logs_tx: SyncSender<Log>,
    ) {
        {
            trace!("waiting to get write on the state");
            let meta = self.meta().await;
            let mut state = self.state.write().await;

            *state = match state.take() {
                DeploymentState::Queued(queued) => {
                    debug!("deployment '{}' build starting...", &meta.id);

                    let console_writer = BuildOutputWriter::new(self.meta.clone());
                    match context
                        .build_system
                        .build(
                            &queued.crate_bytes,
                            meta.project.as_str(),
                            Box::new(console_writer),
                        )
                        .await
                    {
                        Ok(build) => DeploymentState::built(build),
                        Err(e) => {
                            dbg!("failed to build with error: {}", &e);
                            DeploymentState::Error(e)
                        }
                    }
                }
                DeploymentState::Built(built) => {
                    debug!(
                        "deployment '{}' loading shared object and service...",
                        &meta.id
                    );

                    match Loader::from_so_file(&built.build.so_path) {
                        Ok(loader) => DeploymentState::loaded(loader),
                        Err(e) => {
                            debug!("failed to load with error: {}", &e);
                            DeploymentState::Error(e.into())
                        }
                    }
                }
                DeploymentState::Loaded(loader) => {
                    let port = identify_free_port();

                    debug!(
                        "deployment '{}' getting deployed on port {}...",
                        meta.id, port
                    );

                    debug!("{}: factory phase", meta.project);
                    let db_state = database::State::new(&meta.project, db_context);

                    let mut factory = ShuttleFactory::new(db_state);
                    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);
                    match loader.load(&mut factory, addr, run_logs_tx, meta.id).await {
                        Err(e) => {
                            debug!("{}: factory phase FAILED: {:?}", meta.project, e);
                            DeploymentState::Error(e.into())
                        }
                        Ok((handle, so)) => {
                            debug!("{}: factory phase DONE", meta.project);
                            self.meta.write().await.database_deployment =
                                factory.to_database_info();

                            // Remove stale active deployments
                            if let Some(stale_id) = context.router.promote(meta.host, meta.id).await
                            {
                                debug!("removing stale deployment `{}`", &stale_id);
                                context.deployments.write().await.remove(&stale_id);
                            }

                            DeploymentState::deployed(so, port, handle)
                        }
                    }
                }
                deployed_or_error => deployed_or_error, /* nothing to do here */
            };
        }

        // ensures that the metadata state is inline with the actual state. This
        // can go when we have an API layer.
        self.update_meta_state().await;
    }

    async fn update_meta_state(&self) {
        self.meta.write().await.state = self.state.read().await.meta()
    }

    async fn port(&self) -> Option<Port> {
        match &*self.state.read().await {
            DeploymentState::Deployed(deployed) => Some(deployed.port),
            _ => None,
        }
    }

    async fn add_runtime_log(&self, datetime: DateTime<Utc>, log: LogItem) {
        self.meta.write().await.runtime_logs.insert(datetime, log);
    }
}

/// Provides a `Write` wrapper around the build logs - i.e., the build output
/// is written into our build logs using this wrapper.
struct BuildOutputWriter {
    meta: Arc<RwLock<DeploymentMeta>>,
    buf: String,
}

impl BuildOutputWriter {
    pub fn new(meta: Arc<RwLock<DeploymentMeta>>) -> Self {
        Self {
            meta,
            buf: String::new(),
        }
    }
}

impl Write for BuildOutputWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let write_len = buf.len();
        let is_new_line = buf[write_len - 1] == b'\n';
        if let Ok(buf) = std::str::from_utf8(buf) {
            self.buf.push_str(buf);

            // The flush step introduces async code which can potentially execute out of order.
            // For example, if the write is called with inputs a, b, and c then a new threads will be
            // spawned to add a, b, and c to meta. However, the threads might execute in others b, a,
            // and c if they are too close to one another. This `write` method seems to be called for
            // every character which causes many threads with overlapping lifetimes and therefore
            // many out of order executions which just mess up the log output.
            // Since line orders rarely matter and only spawning a thread for each output line also
            // reduces the likelihood of threads with overlapping lifetimes, the guard exists.
            if is_new_line {
                // Safe to unwrap since `flush` has no errors internally
                self.flush().unwrap();
            }

            return Ok(write_len);
        }

        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let handle = tokio::runtime::Handle::current();
        let meta = self.meta.clone();
        let buf = self.buf.clone();
        self.buf = String::new();

        // Can't call `block_on` on a thread that already has a tokio executor, so spawn a new one
        std::thread::spawn(move || {
            handle.block_on(async {
                meta.write()
                    .await
                    .build_logs
                    .get_or_insert("".to_string())
                    .push_str(&buf)
            });
        });

        Ok(())
    }
}

// Make sure to clean the buffer one last time
impl Drop for BuildOutputWriter {
    fn drop(&mut self) {
        self.flush().unwrap();
    }
}

type Deployments = HashMap<DeploymentId, Arc<Deployment>>;

/// The top-level manager for deployments. Is responsible for their creation
/// and lifecycle.
pub(crate) struct DeploymentSystem {
    deployments: Arc<RwLock<Deployments>>,
    job_queue: JobQueue,
    router: Arc<Router>,
    fqdn: String,
}

const JOB_QUEUE_SIZE: usize = 200;

struct JobQueue {
    send: mpsc::Sender<Arc<Deployment>>,
}

impl JobQueue {
    async fn new(
        context: Context,
        db_context: database::Context,
        run_logs_tx: SyncSender<Log>,
    ) -> Self {
        let (send, mut recv) = mpsc::channel::<Arc<Deployment>>(JOB_QUEUE_SIZE);

        log::debug!("starting job processor task");

        tokio::spawn(async move {
            while let Some(deployment) = recv.recv().await {
                let id = deployment.meta().await.id;

                log::debug!("started deployment job for deployment '{}'", id);

                while !deployment.deployment_finished().await {
                    let run_logs_tx = run_logs_tx.clone();

                    deployment.advance(&context, &db_context, run_logs_tx).await;
                }

                debug!("ended deployment job for id: '{}'", id);
            }

            log::debug!("job processor task ended");
        });

        Self { send }
    }

    async fn push(&self, deployment: Arc<Deployment>) {
        self.send
            .send(deployment)
            .await
            .unwrap_or_else(|_| panic!("deployment job queue unexpectedly closed"));
    }
}

/// Convenience struct used to store a bunch of stuff needed for the job
/// processor.
pub(crate) struct Context {
    router: Arc<Router>,
    build_system: Box<dyn BuildSystem>,
    deployments: Arc<RwLock<Deployments>>,
}

impl DeploymentSystem {
    pub(crate) async fn new(build_system: Box<dyn BuildSystem>, fqdn: String) -> Self {
        let router: Arc<Router> = Default::default();
        let (tx, rx) = std::sync::mpsc::sync_channel::<Log>(64);

        let deployments = Arc::new(RwLock::new(
            Self::initialise_from_fs(&build_system.fs_root(), &fqdn).await,
        ));

        let deployments_log = deployments.clone();

        tokio::spawn(async move {
            loop {
                let res = rx.recv();

                if let Ok(log) = res {
                    let mut deployments_log = deployments_log.write().await;
                    if let Some(deployment) = deployments_log.get_mut(&log.deployment_id) {
                        deployment.add_runtime_log(log.datetime, log.item).await;
                    }
                } else {
                    break;
                }
            }
        });

        let context = Context {
            router: router.clone(),
            build_system,
            deployments: deployments.clone(),
        };

        let db_context = database::Context::new()
            .await
            .expect("failed to create lazy connection to database");

        let job_queue = JobQueue::new(context, db_context, tx).await;

        debug!("loading deployments into job processor");
        for deployment in deployments.read().await.values() {
            debug!("loading deployment: {:?}", deployment.meta);
            job_queue.push(deployment.clone()).await;
        }

        Self {
            deployments,
            job_queue,
            router,
            fqdn,
        }
    }

    /// Traverse the build directory re-create deployments.
    /// If a project could not be re-created, this will get logged and skipped.
    async fn initialise_from_fs(fs_root: &Path, fqdn: &str) -> Deployments {
        let mut deployments = HashMap::default();
        for project_dir in std::fs::read_dir(fs_root).unwrap() {
            let project_dir = match project_dir {
                Ok(project_dir) => project_dir,
                Err(e) => {
                    warn!("failed to read directory for project with error `{:?}`", e);
                    warn!("skipping...");
                    continue;
                }
            };
            let project_name = project_dir.file_name();
            match Deployment::from_directory(fqdn, project_dir) {
                Err(e) => {
                    warn!(
                        "failed to re-create deployment for project `{:?}` with error: {:?}",
                        project_name, e
                    );
                }
                Ok(deployment) => {
                    let deployment = Arc::new(deployment);
                    let info = deployment.meta().await;
                    deployments.insert(info.id, deployment.clone());
                }
            }
        }
        deployments
    }

    /// Returns the port for a given host. If the host does not exist, returns
    /// `None`.
    pub(crate) async fn port_for_host(&self, host: &Host) -> Option<Port> {
        let id_for_host = self.router.id_for_host(host).await?;
        self.deployments
            .read()
            .await
            .get(&id_for_host)?
            .port()
            .await
    }

    /// Retrieves a clone of the deployment information
    pub(crate) async fn get_deployment(
        &self,
        id: &DeploymentId,
    ) -> Result<DeploymentMeta, DeploymentApiError> {
        match self.deployments.read().await.get(id) {
            Some(deployment) => Ok(deployment.meta().await),
            None => Err(DeploymentApiError::NotFound(format!(
                "could not find deployment for id '{}'",
                &id
            ))),
        }
    }

    /// Retrieves a clone of the deployment information
    /// for a given project. If there are multiple deployments
    /// for a given project, will return the latest.
    pub(crate) async fn get_deployment_for_project(
        &self,
        project_name: &ProjectName,
    ) -> Result<DeploymentMeta, DeploymentApiError> {
        let mut candidates = Vec::new();

        for deployment in self.deployments.read().await.values() {
            if deployment.meta.read().await.project == *project_name {
                candidates.push(deployment.meta().await);
            }
        }

        let latest = candidates
            .into_iter()
            .max_by(|d1, d2| d1.created_at.cmp(&d2.created_at));

        match latest {
            Some(latest) => Ok(latest),
            None => Err(DeploymentApiError::NotFound(format!(
                "could not find deployment for project '{}'",
                &project_name
            ))),
        }
    }

    pub(crate) async fn kill_deployment_for_project(
        &self,
        project_name: &ProjectName,
    ) -> Result<DeploymentMeta, DeploymentApiError> {
        let id = self.get_deployment_for_project(project_name).await?.id;
        self.kill_deployment(&id).await
    }

    /// Remove a deployment from the deployments hash map and, if it has
    /// already been deployed, kill the Tokio task in which it is running
    /// and deallocate the linked library.
    pub(crate) async fn kill_deployment(
        &self,
        id: &DeploymentId,
    ) -> Result<DeploymentMeta, DeploymentApiError> {
        match self.deployments.write().await.remove(id) {
            Some(deployment) => {
                let mut meta = deployment.meta().await;

                // If the deployment is in the 'deployed' state, kill the Tokio
                // task in which it is deployed and deallocate the linked
                // library when the runtime gets around to it.

                let mut lock = deployment.state.write().await;
                if let DeploymentState::Deployed(DeployedState { so, handle, .. }) = lock.take() {
                    handle.abort();

                    tokio::spawn(async move {
                        so.close().unwrap();
                    });

                    match handle.await {
                        Err(err) if err.is_cancelled() => {}
                        other => other
                            .map_err(|e| DeploymentApiError::Internal(e.to_string()))?
                            .map_err(|e| DeploymentApiError::Internal(e.to_string()))?,
                    };
                }

                let _ = self.router.remove(&meta.host);

                meta.state = DeploymentStateMeta::Deleted;

                Ok(meta)
            }
            None => Err(DeploymentApiError::NotFound(String::new())),
        }
    }

    pub(crate) async fn num_active(&self) -> usize {
        let deployments = self
            .deployments
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        stream::unfold(deployments, |mut deployments| async move {
            Some((deployments.pop()?.deployment_active().await, deployments))
        })
        .filter(|is_active| future::ready(*is_active))
        .count()
        .await
    }

    /// Main way to interface with the deployment manager.
    /// Will take a crate through the whole lifecycle.
    pub(crate) async fn deploy(
        &self,
        crate_file: Data<'_>,
        project: ProjectName,
    ) -> Result<DeploymentMeta, DeploymentApiError> {
        // Assumes that only `::Deployed` deployments are blocking a thread.
        if self.num_active().await >= MAX_DEPLOYS {
            return Err(DeploymentApiError::Unavailable(
                "this instance has reached its maximum number of supported deployments".to_string(),
            ));
        };

        let crate_bytes = crate_file
            .open(ByteUnit::max_value())
            .into_bytes()
            .await
            .map_err(|_| {
                DeploymentApiError::BadRequest("could not read crate file into bytes".to_string())
            })?
            .to_vec();

        let deployment = Arc::new(Deployment::from_bytes(&self.fqdn, project, crate_bytes));

        let info = deployment.meta().await;

        self.deployments
            .write()
            .await
            .insert(info.id, deployment.clone());

        self.job_queue.push(deployment).await;

        Ok(info)
    }
}

/// Call on the operating system to identify an available port on which a
/// deployment may then be hosted.
fn identify_free_port() -> u16 {
    let ip = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    TcpListener::bind(ip).unwrap().local_addr().unwrap().port()
}

/// Finite-state machine representing the various stages of the build
/// process.
enum DeploymentState {
    /// Deployment waiting to be built.
    Queued(QueuedState),
    /// Built deployment that is ready and waiting to be loaded.
    Built(BuiltState),
    /// Deployment is loaded into the server application as a
    /// dynamically-linked library (`.so` file). The [`libloading`] crate has
    /// been used to achieve this and to obtain this particular deployment's
    /// implementation of the [`shuttle_service::Service`] trait.
    Loaded(Loader),
    /// Deployment that is actively running inside a Tokio task and listening
    /// for connections on some port indicated in [`DeployedState`].
    Deployed(DeployedState),
    /// A state entered when something unexpected occurs during the deployment
    /// process.
    Error(anyhow::Error),
    /// A state indicating that the user has intentionally terminated this
    /// deployment
    #[allow(dead_code)]
    Deleted,
}

impl DeploymentState {
    fn take(&mut self) -> Self {
        std::mem::replace(self, DeploymentState::Deleted)
    }

    fn queued(crate_bytes: Vec<u8>) -> Self {
        Self::Queued(QueuedState { crate_bytes })
    }

    fn built(build: Build) -> Self {
        Self::Built(BuiltState { build })
    }

    fn loaded(loader: Loader) -> Self {
        Self::Loaded(loader)
    }

    fn deployed(so: Library, port: Port, handle: ServeHandle) -> Self {
        Self::Deployed(DeployedState { so, port, handle })
    }

    fn meta(&self) -> DeploymentStateMeta {
        match self {
            DeploymentState::Queued(_) => DeploymentStateMeta::Queued,
            DeploymentState::Built(_) => DeploymentStateMeta::Built,
            DeploymentState::Loaded(_) => DeploymentStateMeta::Loaded,
            DeploymentState::Deployed(_) => DeploymentStateMeta::Deployed,
            DeploymentState::Error(e) => DeploymentStateMeta::Error(format!("{:#?}", e)),
            DeploymentState::Deleted => DeploymentStateMeta::Deleted,
        }
    }
}

struct QueuedState {
    crate_bytes: Vec<u8>,
}

struct BuiltState {
    build: Build,
}

#[allow(dead_code)]
struct DeployedState {
    so: Library,
    port: Port,
    handle: ServeHandle,
}
