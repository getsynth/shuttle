use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bollard::{Docker, API_DEFAULT_VERSION};
use chrono::Utc;
use dal::{Dal, DalError, Deployment, Service};
use deployment::{RunnableDeployment, USER_SERVICE_DEFAULT_PORT};
use derive_builder::Builder;
use error::{Error, Result};
use futures::TryFutureExt;
use http::Uri;
use project::docker::{ContainerSettings, ServiceDockerContext};
use project::service::state::a_creating::ServiceCreating;
use project::service::state::f_running::ServiceRunning;
use project::service::state::StateVariant;
use project::service::ServiceState;
use project::task::{BoxedTask, Task, TaskBuilder};
use runtime_manager::RuntimeManager;
use shuttle_common::backends::auth::VerifyClaim;
use shuttle_common::claims::Claim;
use shuttle_common::{
    backends::{auth::JwtAuthenticationLayer, tracing::ExtractPropagationLayer},
    claims::Scope,
};
use shuttle_proto::auth::AuthPublicKey;
use shuttle_proto::deployer::{
    deployer_server::{Deployer, DeployerServer},
    DeployRequest, DeployResponse,
};
use shuttle_proto::deployer::{
    DestroyDeploymentRequest, DestroyDeploymentResponse, ProjectChange, ProjectEvent,
    SubscribeProjectsRequest, UnsubscribeProjectsRequest, UnsubscribeProjectsResponse,
};
use tokio::sync::mpsc::{self, Sender, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tonic::{transport::Server, Response, Result as TonicResult};
use tracing::{error, info, instrument};
use ulid::Ulid;

use crate::project::worker::{TaskRouter, Worker};
use crate::project::{error::Error as ProjectError, task};

pub mod args;
pub mod dal;
pub mod deployment;
pub mod error;
pub mod project;
pub mod runtime_manager;

const RUN_BUFFER_SIZE: usize = 100;

#[derive(Default)]
pub struct GitMetadata {
    git_commit_hash: Option<String>,
    git_branch: Option<String>,
    git_dirty: Option<bool>,
    git_commit_message: Option<String>,
}

impl GitMetadata {
    pub fn new(
        git_branch: Option<String>,
        git_commit_hash: Option<String>,
        git_commit_message: Option<String>,
        git_dirty: Option<bool>,
    ) -> Self {
        GitMetadata {
            git_commit_hash,
            git_branch,
            git_dirty,
            git_commit_message,
        }
    }
}

#[derive(Builder, Clone)]
pub struct DeployerServiceConfig {
    bind_address: SocketAddr,
    docker_host: PathBuf,
    provisioner_uri: Uri,
    auth_uri: Uri,
    network_name: String,
    prefix: String,
}

pub struct DeployerService<D: Dal + Send + Sync + 'static> {
    runtime_manager: RuntimeManager,
    docker: Docker,
    dal: D,
    task_router: TaskRouter<BoxedTask>,
    deployment_state_machine_channel:
        tokio::sync::mpsc::Sender<Box<dyn Task<(), Output = (), Error = project::error::Error>>>,
    runtime_start_channel: Sender<RunnableDeployment>,

    // These channel are intended to work with one subscriber. They can be reinitialized as well.
    // They are wrapped in Mutex first because we want them with interior muttability (see unsubscribe method).
    // They are wrapped in Arc because they get sent on all deployment tasks.
    project_events_channel_tx: Arc<Mutex<Option<UnboundedSender<ProjectEvent>>>>,
    project_events_channel_rx: Arc<Mutex<Option<UnboundedReceiver<ProjectEvent>>>>,

    config: DeployerServiceConfig,
}

impl<D: Dal + Send + Sync + 'static> DeployerService<D> {
    pub async fn new(dal: D, config: DeployerServiceConfig) -> Self {
        let runtime_manager = RuntimeManager::default();

        // We create the worker who handles creation of workers per service.
        // We're sending through this channel the work that needs to be taken
        // care of for a service.
        let worker = Worker::new();
        let deployment_state_machine_channel = worker.sender();
        tokio::spawn(
            worker
                .start()
                .map_ok(|_| info!("worker terminated successfully"))
                .map_err(|err| error!("worker error: {}", err)),
        );
        let (runtime_start_channel, run_recv) = mpsc::channel(RUN_BUFFER_SIZE);
        tokio::spawn(deployment::task(
            dal.clone(),
            run_recv,
            runtime_manager.clone(),
        ));
        let (tx, rx) = mpsc::unbounded_channel::<ProjectEvent>();

        Self {
            docker: Docker::connect_with_unix(
                config
                    .docker_host
                    .to_str()
                    .expect("docker host path to be a valid filesystem path"),
                60,
                API_DEFAULT_VERSION,
            )
            .expect("to initialize docker connection the installed docker daemon"),
            runtime_manager,
            dal,
            task_router: TaskRouter::default(),
            deployment_state_machine_channel,
            runtime_start_channel,
            project_events_channel_tx: Arc::new(Mutex::new(Some(tx))),
            project_events_channel_rx: Arc::new(Mutex::new(Some(rx))),
            config,
        }
    }

    pub async fn start(self) -> Result<()> {
        // The deployments which are in the `Running` state are considered runnable and they are started again. Running the
        // deployments happens after their associated services' sandboxes are healthy and we start them.
        let runnable_deployments = self.dal.running_deployments().await?;
        info!(count = %runnable_deployments.len(), "enqueuing runnable deployments");
        for existing_deployment in runnable_deployments {
            // We want to restart the corresponding deployment service container.
            let state = self
                .dal
                .service(&existing_deployment.service_id)
                .await?
                .state;

            // Clean the previous docker container if any.
            let runnable_deployment = RunnableDeployment {
                deployment_id: existing_deployment.id,
                service_name: existing_deployment.service_name,
                service_id: existing_deployment.service_id,
                tracing_context: Default::default(),
                claim: None,
                target_ip: state.target_ip(self.config.network_name.as_str()).ok(),
                is_next: existing_deployment.is_next,
            };
            let image_name = state
                .image()
                .map_err(|err| Error::Internal(err.to_string()))?;

            self.instate_service(
                runnable_deployment,
                existing_deployment.project_id,
                GitMetadata::default(),
                image_name.clone(),
                existing_deployment.idle_minutes,
                false,
            )
            .await?;
        }

        let mut server_builder = Server::builder()
            .http2_keepalive_interval(Some(Duration::from_secs(60)))
            .layer(JwtAuthenticationLayer::new(AuthPublicKey::new(
                shuttle_proto::auth::client(&self.config.auth_uri)
                    .await
                    .expect("auth service should be reachable"),
            )))
            .layer(ExtractPropagationLayer);
        let bind_address = self.config.bind_address;
        let svc = DeployerServer::new(self);
        let router = server_builder.add_service(svc);

        router
            .serve(bind_address)
            .await
            .expect("to serve on address");
        Ok(())
    }

    /// Ensures this service is created and the runtime loaded & started. Important to note that this method
    /// can be called when starting the deployer, to pick up from persistence the existing deployments and
    /// reinstate them if they are on the running code path, but also when deploying a brand new deployment,
    /// storing it in the persistence.
    pub async fn instate_service(
        &self,
        runnable_deployment: RunnableDeployment,
        project_id: Ulid,
        git_metadata: GitMetadata,
        image_name: String,
        idle_minutes: u64,
        force: bool,
    ) -> Result<()> {
        // The creating step might be required, initing now.
        let creating = ServiceState::Creating(ServiceCreating::new(
            runnable_deployment.service_id,
            runnable_deployment.deployment_id,
            project_id,
            image_name.clone(),
            idle_minutes,
        ));

        // If the service already lives in the persistence with a previous state.
        if let Some(state) = self
            .dal
            .service_state(&runnable_deployment.service_id)
            .await
            .map_err(Error::Dal)?
        {
            // But the container is not on the running path and the instating is with force.
            if (state.is_destroyed() || state.is_stopped() || state.is_completed()) && force {
                // Update the state to creating.
                self.dal
                    .update_service_state(runnable_deployment.service_id, creating)
                    .await
                    .map_err(Error::Dal)?;
            }
        } else {
            // Insert the service.
            let service = Service {
                id: runnable_deployment.service_id,
                name: runnable_deployment.service_name.clone(),
                state_variant: creating.to_string(),
                state: creating,
                last_update: Utc::now(),
                project_id,
            };
            self.dal
                .insert_service_if_absent(service)
                .await
                .map_err(Error::Dal)?;

            // Insert the new deployment.
            let deployment = Deployment {
                id: runnable_deployment.deployment_id,
                service_id: runnable_deployment.service_id,
                last_update: Utc::now(),
                is_next: runnable_deployment.is_next,
                git_branch: git_metadata.git_branch,
                git_commit_hash: git_metadata.git_commit_hash,
                git_commit_message: git_metadata.git_commit_message,
                git_dirty: git_metadata.git_dirty,
            };
            self.dal.insert_deployment(deployment).await?;
        }

        // We want to refresh the service.
        let service_id = runnable_deployment.service_id;
        let cs = ContainerSettings::builder()
            .image(image_name)
            .provisioner_host(self.config.provisioner_uri.to_string())
            .auth_uri(self.config.auth_uri.to_string())
            .network_name(self.config.network_name.to_string())
            .runnable_deployment(runnable_deployment)
            .runtime_start_channel(self.runtime_start_channel.clone())
            .prefix(self.config.prefix.to_string())
            .build()
            .await;

        TaskBuilder::new(self.dal.clone())
            .task_router(self.task_router.clone())
            .service_id(service_id)
            .service_docker_context(ServiceDockerContext::new_with_container_settings(
                self.docker.clone(),
                cs,
                self.runtime_manager.clone(),
                self.project_events_channel_tx.clone(),
            ))
            .and_then(task::refresh())
            .and_then(task::run_until_done())
            .send(&self.deployment_state_machine_channel)
            .await
            .expect("to get a handle of the created task")
            .await;

        Ok(())
    }
}

#[async_trait]
impl<D: Dal + Sync + 'static> Deployer for DeployerService<D> {
    #[instrument(skip(self, request))]
    async fn deploy(
        &self,
        request: tonic::Request<DeployRequest>,
    ) -> TonicResult<tonic::Response<DeployResponse>, tonic::Status> {
        // Authorize the request.
        request.verify(Scope::DeploymentWrite)?;

        let claim = request.extensions().get::<Claim>().cloned();
        let request = request.into_inner();
        let req_deployment = request.deployment.ok_or(tonic::Status::invalid_argument(
            "missing deploymet information in the rpc call",
        ))?;
        let service_id = Ulid::from_string(req_deployment.service_id.as_str())
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        let project_id = Ulid::from_string(req_deployment.project_id.as_str())
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;
        // Create the deployment.
        let deployment_id = Ulid::new();
        let is_next = req_deployment.is_next;
        let service_name: String = req_deployment.service_name.clone();
        let image_name = req_deployment.image_name.clone();
        let idle_minutes = u64::from(req_deployment.idle_minutes);
        let runnable_deployment = RunnableDeployment {
            deployment_id,
            service_name,
            service_id,
            tracing_context: Default::default(),
            claim,
            target_ip: None,
            is_next,
        };
        let git_metadata = GitMetadata::new(
            req_deployment.git_branch,
            req_deployment.git_commit_hash,
            req_deployment.git_commit_message,
            req_deployment.git_dirty,
        );

        // Instate the service.
        self.instate_service(
            runnable_deployment,
            project_id,
            git_metadata,
            image_name,
            idle_minutes,
            false,
        )
        .await
        .map_err(|err| {
            tonic::Status::internal(format!("failed to instate the service: {}", err))
        })?;

        Ok(Response::new(DeployResponse {
            deployment_id: deployment_id.to_string(),
        }))
    }

    #[instrument(skip_all)]
    async fn destroy_deployment(
        &self,
        request: tonic::Request<DestroyDeploymentRequest>,
    ) -> TonicResult<tonic::Response<DestroyDeploymentResponse>, tonic::Status> {
        // Authorize the request.
        request.verify(Scope::DeploymentWrite)?;
        let request = request.into_inner();

        // Do a cleanup in terms of previous invalid deployments.
        let deployment_id = Ulid::from_string(&request.deployment_id)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;
        let deployment = self
            .dal
            .deployment(&deployment_id)
            .await
            .map_err(|err| match err {
                DalError::DeploymentNotFound => tonic::Status::not_found(err.to_string()),
                _ => tonic::Status::internal(err.to_string()),
            })?;
        let service = self
            .dal
            .service(&deployment.service_id)
            .await
            .map_err(|err| match err {
                DalError::ServiceNotFound => tonic::Status::not_found(err.to_string()),
                _ => tonic::Status::internal(err.to_string()),
            })?;

        if service.state_variant != ServiceRunning::name() {
            return Err(tonic::Status::cancelled(
                "deployment is not running".to_string(),
            ));
        }

        // Destroying the deployment and waiting on finishing up
        let dal = self.dal.clone();
        let task_router = self.task_router.clone();
        let docker = self.docker.clone();
        let runtime_manager = self.runtime_manager.clone();
        let sender = self.deployment_state_machine_channel.clone();

        TaskBuilder::new(dal)
            .task_router(task_router)
            .service_id(deployment.service_id)
            .service_docker_context(ServiceDockerContext::new(
                docker,
                runtime_manager,
                self.project_events_channel_tx.clone(),
            ))
            .and_then(task::destroy())
            .and_then(task::run_until_done())
            .send(&sender)
            .await
            .map_err(|err| match err {
                ProjectError::TaskInternal => tonic::Status::internal(err.to_string()),
                ProjectError::Service(err) => tonic::Status::internal(err.to_string()),
                ProjectError::ServiceUnavailable => tonic::Status::unavailable(err.to_string()),
                ProjectError::Dal(dal_err) => match dal_err {
                    DalError::Sqlx(_) | DalError::Decode(_) => tonic::Status::internal(
                        "querying the database while destroying the project failed",
                    ),
                    DalError::ServiceNotFound | DalError::DeploymentNotFound => {
                        tonic::Status::not_found(dal_err.to_string())
                    }
                },
            })?
            .await;

        Ok(Response::new(DestroyDeploymentResponse {}))
    }

    type SubscribeProjectsStream =
        tokio_stream::wrappers::ReceiverStream<TonicResult<ProjectEvent, tonic::Status>>;

    #[instrument(skip_all)]
    async fn subscribe_projects(
        &self,
        _request: tonic::Request<SubscribeProjectsRequest>,
    ) -> TonicResult<tonic::Response<Self::SubscribeProjectsStream>, tonic::Status> {
        // We can not authorize yet to easily these requests given they need machine-to-machine authorization.
        // TODO: add support for this in the auth component.
        // request.verify(Scope::Admin)?;
        let events_rx = self.project_events_channel_rx.lock().await.take();

        if let Some(mut events_rx) = events_rx {
            let (tx, rx) = mpsc::channel(1);
            let dal = self.dal.clone();
            let network_name = self.config.network_name.clone();
            // Move the events items into stream to be returned
            tokio::spawn(async move {
                let services = dal.services().await.unwrap_or_else(|err| {
                    error!("{err}");
                    Vec::new()
                });
                for service in services {
                    let socket_addr = if service.state.is_running() {
                        let target_ip = service
                            .state
                            .target_ip(network_name.as_str())
                            .expect("to be attached to the network");
                        Some(format!("{}:{}", target_ip, USER_SERVICE_DEFAULT_PORT))
                    } else {
                        None
                    };

                    tx.send(Ok(ProjectEvent {
                        service_id: service.id.to_string(),
                        project_id: service.project_id.to_string(),
                        change: Some(ProjectChange {
                            state_variant: service.state_variant,
                            socket_addr,
                        }),
                    }))
                    .await
                    .unwrap_or_else(|err| error!("errored when sending project event: {err}"));
                }

                while let Some(event) = events_rx.recv().await {
                    tx.send(Ok(event))
                        .await
                        .unwrap_or_else(|err| error!("errored when sending project event: {err}"));
                }
            });

            Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
                rx,
            )))
        } else {
            Err(tonic::Status::cancelled(
                "the events have already been subscribed to. You must unsubscribe first to be able to subscribe again.",
            ))
        }
    }

    #[instrument(skip_all)]
    async fn unsubscribe_projects(
        &self,
        _request: tonic::Request<UnsubscribeProjectsRequest>,
    ) -> TonicResult<tonic::Response<UnsubscribeProjectsResponse>, tonic::Status> {
        // We can not authorize yet to easily these requests given they need machine-to-machine authorization.
        // TODO: add support for this in the auth component.
        // request.verify(Scope::Admin)?;
        let mut guard_rx = self.project_events_channel_rx.lock().await;
        let mut guard_tx = self.project_events_channel_tx.lock().await;
        let (tx, rx) = mpsc::unbounded_channel::<ProjectEvent>();
        guard_rx.replace(rx);
        guard_tx.replace(tx);
        Ok(Response::new(UnsubscribeProjectsResponse {}))
    }
}
