pub mod deploy_layer;
pub mod gateway_client;
mod queue;
mod run;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub use queue::Queued;
pub use run::{ActiveDeploymentsGetter, Built};
use tracing::{instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{
    persistence::{DeploymentUpdater, ResourceManager, SecretGetter, SecretRecorder, State},
    RuntimeManager,
};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinSet,
};
use uuid::Uuid;

use self::{deploy_layer::LogRecorder, gateway_client::BuildQueueClient};

const QUEUE_BUFFER_SIZE: usize = 100;
const RUN_BUFFER_SIZE: usize = 100;

pub struct DeploymentManagerBuilder<LR, SR, ADG, DU, SG, RM, QC> {
    build_log_recorder: Option<LR>,
    secret_recorder: Option<SR>,
    active_deployment_getter: Option<ADG>,
    artifacts_path: Option<PathBuf>,
    runtime_manager: Option<Arc<Mutex<RuntimeManager>>>,
    deployment_updater: Option<DU>,
    secret_getter: Option<SG>,
    resource_manager: Option<RM>,
    queue_client: Option<QC>,
}

impl<LR, SR, ADG, DU, SG, RM, QC> DeploymentManagerBuilder<LR, SR, ADG, DU, SG, RM, QC>
where
    LR: LogRecorder,
    SR: SecretRecorder,
    ADG: ActiveDeploymentsGetter,
    DU: DeploymentUpdater,
    SG: SecretGetter,
    RM: ResourceManager,
    QC: BuildQueueClient,
{
    pub fn build_log_recorder(mut self, build_log_recorder: LR) -> Self {
        self.build_log_recorder = Some(build_log_recorder);

        self
    }

    pub fn secret_recorder(mut self, secret_recorder: SR) -> Self {
        self.secret_recorder = Some(secret_recorder);

        self
    }

    pub fn active_deployment_getter(mut self, active_deployment_getter: ADG) -> Self {
        self.active_deployment_getter = Some(active_deployment_getter);

        self
    }

    pub fn artifacts_path(mut self, artifacts_path: PathBuf) -> Self {
        self.artifacts_path = Some(artifacts_path);

        self
    }

    pub fn queue_client(mut self, queue_client: QC) -> Self {
        self.queue_client = Some(queue_client);

        self
    }

    pub fn secret_getter(mut self, secret_getter: SG) -> Self {
        self.secret_getter = Some(secret_getter);

        self
    }

    pub fn resource_manager(mut self, resource_manager: RM) -> Self {
        self.resource_manager = Some(resource_manager);

        self
    }

    pub fn runtime(mut self, runtime_manager: Arc<Mutex<RuntimeManager>>) -> Self {
        self.runtime_manager = Some(runtime_manager);

        self
    }

    pub fn deployment_updater(mut self, deployment_updater: DU) -> Self {
        self.deployment_updater = Some(deployment_updater);

        self
    }

    /// Creates two Tokio tasks, one for building queued services, the other for
    /// executing/deploying built services. Two multi-producer, single consumer
    /// channels are also created which are for moving on-going service
    /// deployments between the aforementioned tasks.
    pub fn build(self) -> DeploymentManager {
        let build_log_recorder = self
            .build_log_recorder
            .expect("a build log recorder to be set");
        let secret_recorder = self.secret_recorder.expect("a secret recorder to be set");
        let active_deployment_getter = self
            .active_deployment_getter
            .expect("an active deployment getter to be set");
        let artifacts_path = self.artifacts_path.expect("artifacts path to be set");
        let queue_client = self.queue_client.expect("a queue client to be set");
        let runtime_manager = self.runtime_manager.expect("a runtime manager to be set");
        let deployment_updater = self
            .deployment_updater
            .expect("a deployment updater to be set");
        let secret_getter = self.secret_getter.expect("a secret getter to be set");
        let resource_manager = self.resource_manager.expect("a resource manager to be set");

        let (queue_send, queue_recv) = mpsc::channel(QUEUE_BUFFER_SIZE);
        let (run_send, run_recv) = mpsc::channel(RUN_BUFFER_SIZE);

        let builds_path = artifacts_path.join("shuttle-builds");

        let run_send_clone = run_send.clone();
        let mut set = JoinSet::new();

        // Build queue. Waits for incoming deployments and builds them.
        set.spawn(queue::task(
            queue_recv,
            run_send_clone,
            deployment_updater.clone(),
            build_log_recorder,
            secret_recorder,
            queue_client,
            builds_path.clone(),
        ));
        // Run queue. Waits for built deployments and runs them.
        set.spawn(run::task(
            run_recv,
            runtime_manager.clone(),
            deployment_updater,
            active_deployment_getter,
            secret_getter,
            resource_manager,
            builds_path.clone(),
        ));

        DeploymentManager {
            queue_send,
            run_send,
            runtime_manager,
            _join_set: Arc::new(Mutex::new(set)),
            builds_path,
        }
    }
}

#[derive(Clone)]
pub struct DeploymentManager {
    queue_send: QueueSender,
    run_send: RunSender,
    runtime_manager: Arc<Mutex<RuntimeManager>>,
    _join_set: Arc<Mutex<JoinSet<()>>>,
    builds_path: PathBuf,
}

/// ```no-test
/// queue channel   all deployments here are State::Queued until the get a slot from gateway
///       |
///       v
///  queue task     when taken from the channel by this task, deployments
///                 enter the State::Building state and upon being
///       |         built transition to the State::Built state
///       v
///  run channel    all deployments here are State::Built
///       |
///       v
///    run task     tasks enter the State::Running state and begin
///                 executing
/// ```
impl DeploymentManager {
    /// Create a new deployment manager. Manages one or more 'pipelines' for
    /// processing service building, loading, and deployment.
    pub fn builder<LR, SR, ADG, DU, SG, RM, QC>(
    ) -> DeploymentManagerBuilder<LR, SR, ADG, DU, SG, RM, QC> {
        DeploymentManagerBuilder {
            build_log_recorder: None,
            secret_recorder: None,
            active_deployment_getter: None,
            artifacts_path: None,
            runtime_manager: None,
            deployment_updater: None,
            secret_getter: None,
            resource_manager: None,
            queue_client: None,
        }
    }

    pub async fn queue_push(&self, mut queued: Queued) {
        let cx = Span::current().context();

        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut queued.tracing_context);
        });

        self.queue_send.send(queued).await.unwrap();
    }

    #[instrument(skip(self), fields(id = %built.id, state = %State::Built))]
    pub async fn run_push(&self, built: Built) {
        self.run_send.send(built).await.unwrap();
    }

    pub async fn kill(&self, id: Uuid) {
        self.runtime_manager.lock().await.kill(&id).await;
    }

    pub fn builds_path(&self) -> &Path {
        self.builds_path.as_path()
    }
}

type QueueSender = mpsc::Sender<queue::Queued>;
type QueueReceiver = mpsc::Receiver<queue::Queued>;

type RunSender = mpsc::Sender<run::Built>;
type RunReceiver = mpsc::Receiver<run::Built>;
