use std::collections::HashMap;
use std::net::{SocketAddrV4, Ipv4Addr, TcpListener};
use tokio::sync::{RwLock, oneshot};
use core::default::Default;
use libloading::Library;
use std::io::Write;
use rocket::data::ByteUnit;
use rocket::response::Responder;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio;
use rocket::Data;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::build::Build;
use crate::BuildSystem;
use lib::{DeploymentId, DeploymentMeta, DeploymentStateMeta, Host, Port, ProjectConfig};

use unveil_service::Service;
use crate::router::Router;

// TODO: Determine error handling strategy - error types or just use `anyhow`?
#[derive(Debug, Clone, Serialize, Deserialize, Responder)]
pub enum DeploymentError {
    #[response(status = 500)]
    Internal(String),
    #[response(status = 404)]
    NotFound(String),
    #[response(status = 400)]
    BadRequest(String),
}

/// Inner struct of a deployment which holds the deployment itself
/// and the some metadata
pub(crate) struct Deployment {
    meta: Arc<RwLock<DeploymentMeta>>,
    state: RwLock<DeploymentState>,
}

impl Deployment {
    fn new(config: &ProjectConfig, crate_bytes: Vec<u8>) -> Self {
        Self {
            meta: Arc::new(RwLock::new(DeploymentMeta::new(&config))),
            state: RwLock::new(DeploymentState::queued(crate_bytes)),
        }
    }

    /// Gets a `clone`ed copy of the metadata.
    pub(crate) async fn meta(&self) -> DeploymentMeta {
        dbg!("trying to get meta");
        self.meta.read().await.clone()
    }

    /// Evaluates if the deployment can be advanced. If the deployment
    /// has reached a state where it can no longer `advance`, returns `false`.
    pub(crate) async fn deployment_finished(&self) -> bool {
        match *self.state.read().await {
            DeploymentState::QUEUED(_) | DeploymentState::BUILT(_) | DeploymentState::LOADED(_) => false,
            DeploymentState::DEPLOYED(_) | DeploymentState::ERROR => true,
        }
    }

    /// Tries to advance the deployment one stage. Does nothing if the deployment
    /// is in a terminal state.
    pub(crate) async fn advance(&self, context: &Context) {
        dbg!("waiting to get write on the state");
        {
            let meta = self.meta().await;
            let mut state = self.state.write().await;

            *state = match state.take() {
                DeploymentState::QUEUED(queued) => {
                    dbg!("deployment '{}' build starting...", &meta.id);
                    let console_writer = BuildOutputWriter::new(self.meta.clone());
                    match context.build_system
                        .build(&queued.crate_bytes, &meta.config, Box::new(console_writer))
                        .await
                    {
                        Ok(build) => DeploymentState::built(build),
                        Err(e) => {
                            dbg!("failed to build with error: {}", e);
                            DeploymentState::ERROR
                        }
                    }
                }
                DeploymentState::BUILT(built) => {
                    dbg!("deployment '{}' loading shared object and service...", &meta.id);

                    match load_service_from_so_file(&built.build.so_path) {
                        Ok((svc, so)) => DeploymentState::loaded(so, svc),
                        Err(e) => {
                            dbg!("failed to load with error: {}", e);
                            DeploymentState::ERROR
                        }
                    }
                }
                DeploymentState::LOADED(loaded) => {
                    let port = identify_free_port();

                    dbg!("deployment '{}' getting deployed on port {}...", &meta.id, port);

                    let deployed_future = match loaded.service.deploy() {
                        unveil_service::Deployment::Rocket(r) => {
                            let config = rocket::Config {
                                port,
                                log_level: rocket::config::LogLevel::Normal,
                                ..Default::default()
                            };

                            r.configure(config).launch()
                        }
                    };

                    let (kill_oneshot, kill_receiver) = oneshot::channel::<()>();

                    tokio::spawn(async move {
                        tokio::select! {
                            _ = kill_receiver => {}
                            _ = deployed_future => {}
                        }
                    });

                    // todo, when we add deletion logic, the deployment id
                    // returned from promotion should be deleted
                    context.router.promote(meta.host, meta.id).await;

                    DeploymentState::deployed(loaded.so, loaded.service, port, kill_oneshot)
                }
                deployed_or_error => deployed_or_error, /* nothing to do here */
            };
        }
        // ensures that the metadata state is inline with the actual
        // state. This can go when we have an API layer.
        self.update_meta_state().await
    }

    async fn update_meta_state(&self) {
        self.meta.write().await.state = self.state.read().await.meta()
    }

    async fn port(&self) -> Option<Port> {
        match &*self.state.read().await {
            DeploymentState::DEPLOYED(deployed) => Some(deployed.port),
            _ => None
        }
    }
}

// Provides a `Write` wrapper around the build logs
// Ie. the build output is written into our build logs using this wrapper
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
            // reduces the likelyhood of threads with overlapping lifetimes, the guard exists.
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

/// The top-level manager for deployments. Is responsible for their
/// creation and lifecycle.
pub(crate) struct DeploymentSystem {
    deployments: RwLock<Deployments>,
    job_queue: Arc<JobQueue>,
    router: Arc<Router>,
}

#[derive(Default)]
struct JobQueue {
    queue: Arc<Mutex<Vec<Arc<Deployment>>>>,
}

impl JobQueue {
    fn push(&self, deployment: Arc<Deployment>) {
        self.queue.lock().unwrap().push(deployment)
    }

    fn pop(&self) -> Option<Arc<Deployment>> {
        self.queue.lock().unwrap().pop()
    }

    /// Returns a JobQueue with the job processor already running
    async fn initialise(context: Context) -> Arc<Self> {
        let job_queue = Arc::new(JobQueue::default());

        let queue_ref = job_queue.clone();

        tokio::spawn(async move {
            Self::start_job_processor(
                context,
                queue_ref,
            ).await
        });

        job_queue
    }


    async fn start_job_processor(
        context: Context,
        queue: Arc<JobQueue>,
    ) {
        dbg!("job processor started");
        loop {
            if let Some(deployment) = queue.pop() {
                let id = deployment.meta().await.id;

                dbg!("started deployment job for id: '{}'", id);

                while !deployment.deployment_finished().await {
                    deployment.advance(&context).await
                }

                dbg!("ended deployment job for id: '{}'", id);
            } else {
                tokio::time::sleep(Duration::from_millis(100)).await
            }
        }
    }
}

/// Convenience stuct used to store a bunch of stuff needed
/// for the job processor
pub(crate) struct Context {
    router: Arc<Router>,
    build_system: Box<dyn BuildSystem>,
}

impl DeploymentSystem {
    pub(crate) async fn new(build_system: Box<dyn BuildSystem>) -> Self {
        let router: Arc<Router> = Default::default();
        let context = Context {
            router: router.clone(),
            build_system,
        };

        Self {
            deployments: Default::default(),
            job_queue: JobQueue::initialise(context).await,
            router,
        }
    }

    /// Returns the port for a given host.
    /// If the host does not exist, returns `None`
    pub(crate) async fn port_for_host(&self, host: &Host) -> Option<Port> {
        let id_for_host = self.router.id_for_host(host).await?;
        self.deployments.read()
            .await
            .get(&id_for_host)?
            .port()
            .await
    }

    /// Retrieves a clone of the deployment information
    pub(crate) async fn get_deployment(&self, id: &DeploymentId) -> Result<DeploymentMeta, DeploymentError> {
        match self.deployments.read().await.get(&id) {
            Some(deployment) => Ok(deployment.meta().await),
            None => Err(DeploymentError::NotFound(format!("could not find deployment for id '{}'", &id)))
        }
    }

    /// Main way to interface with the deployment manager.
    /// Will take a crate through the whole lifecycle.
    pub(crate) async fn deploy(&self,
                               crate_file: Data<'_>,
                               project_config: &ProjectConfig) -> Result<DeploymentMeta, DeploymentError> {
        let crate_bytes = crate_file
            .open(ByteUnit::max_value()).into_bytes()
            .await
            .map_err(|_| DeploymentError::BadRequest("could not read crate file into bytes".to_string()))?
            .to_vec();

        let deployment = Arc::new(Deployment::new(&project_config, crate_bytes));
        let info = deployment.meta().await;

        self.deployments
            .write()
            .await
            .insert(info.id.clone(), deployment.clone());

        self.add_to_job_queue(deployment);

        Ok(info)
    }

    fn add_to_job_queue(&self, deployment: Arc<Deployment>) {
        self.job_queue.push(deployment)
    }
}

const ENTRYPOINT_SYMBOL_NAME: &'static [u8] = b"_create_service\0";

type CreateService = unsafe extern fn() -> *mut dyn Service;

/// Dynamically load from a `.so` file a value of a type implementing the
/// [`Service`] trait. Relies on the `.so` library having an ``extern "C"`
/// function called [`ENTRYPOINT_SYMBOL_NAME`], likely automatically generated
/// using the [`service::declare_service`] macro.
fn load_service_from_so_file(so_path: &Path) -> anyhow::Result<(Box<dyn Service>, Library)> {
    unsafe {
        let lib = libloading::Library::new(so_path)?;

        let entrypoint: libloading::Symbol<CreateService> = lib.get(ENTRYPOINT_SYMBOL_NAME)?;
        let raw = entrypoint();

        Ok((Box::from_raw(raw), lib))
    }
}

fn identify_free_port() -> u16 {
    let ip = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    TcpListener::bind(ip).unwrap().local_addr().unwrap().port()
}

/// Finite-state machine representing the various stages of the build
/// process.
enum DeploymentState {
    QUEUED(QueuedState),
    BUILT(BuiltState),
    LOADED(LoadedState),
    DEPLOYED(DeployedState),
    ERROR,
}

impl DeploymentState {
    fn take(&mut self) -> Self {
        std::mem::replace(self, DeploymentState::ERROR)
    }

    fn queued(crate_bytes: Vec<u8>) -> Self {
        Self::QUEUED(
            QueuedState {
                crate_bytes
            }
        )
    }

    fn built(build: Build) -> Self {
        Self::BUILT(
            BuiltState {
                build
            }
        )
    }

    fn loaded(so: Library, service: Box<dyn Service>) -> Self {
        Self::LOADED(
            LoadedState {
                service,
                so,
            }
        )
    }

    fn deployed(so: Library, service: Box<dyn Service>, port: Port, kill_oneshot: oneshot::Sender<()>) -> Self {
        Self::DEPLOYED(
            DeployedState {
                service,
                so,
                port,
                kill_oneshot,
            }
        )
    }

    fn meta(&self) -> DeploymentStateMeta {
        match self {
            DeploymentState::QUEUED(_) => DeploymentStateMeta::QUEUED,
            DeploymentState::BUILT(_) => DeploymentStateMeta::BUILT,
            DeploymentState::LOADED(_) => DeploymentStateMeta::LOADED,
            DeploymentState::DEPLOYED(_) => DeploymentStateMeta::DEPLOYED,
            DeploymentState::ERROR => DeploymentStateMeta::ERROR
        }
    }
}

struct QueuedState {
    crate_bytes: Vec<u8>,
}

struct BuiltState {
    build: Build,
}

struct LoadedState {
    service: Box<dyn Service>,
    so: Library,
}

struct DeployedState {
    service: Box<dyn Service>,
    so: Library,
    port: Port,
    kill_oneshot: oneshot::Sender<()>,
}
