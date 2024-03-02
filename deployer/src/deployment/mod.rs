use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use shuttle_common::log::LogRecorder;
use shuttle_proto::{logger, provisioner};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinSet,
};
use tracing::{instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

pub mod gateway_client;
mod queue;
mod run;
pub mod state_change_layer;

use self::gateway_client::BuildQueueClient;
use crate::{
    persistence::{resource::ResourceManager, State},
    RuntimeManager,
};
pub use queue::Queued;
pub use run::{ActiveDeploymentsGetter, Built};

const QUEUE_BUFFER_SIZE: usize = 100;
const RUN_BUFFER_SIZE: usize = 100;

pub struct DeploymentManagerBuilder<LR, ADG, RM, QC> {
    build_log_recorder: Option<LR>,
    logs_fetcher: Option<logger::Client>,
    active_deployment_getter: Option<ADG>,
    artifacts_path: Option<PathBuf>,
    runtime_manager: Option<Arc<Mutex<RuntimeManager>>>,
    resource_manager: Option<RM>,
    queue_client: Option<QC>,
    provisioner_client: Option<provisioner::Client>,
}

impl<LR, ADG, RM, QC> DeploymentManagerBuilder<LR, ADG, RM, QC>
where
    LR: LogRecorder,
    ADG: ActiveDeploymentsGetter,
    RM: ResourceManager,
    QC: BuildQueueClient,
{
    pub fn build_log_recorder(mut self, build_log_recorder: LR) -> Self {
        self.build_log_recorder = Some(build_log_recorder);

        self
    }

    pub fn log_fetcher(mut self, logs_fetcher: logger::Client) -> Self {
        self.logs_fetcher = Some(logs_fetcher);

        self
    }

    pub fn provisioner_client(mut self, provisioner_client: provisioner::Client) -> Self {
        self.provisioner_client = Some(provisioner_client);

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

    pub fn resource_manager(mut self, resource_manager: RM) -> Self {
        self.resource_manager = Some(resource_manager);

        self
    }

    pub fn runtime(mut self, runtime_manager: Arc<Mutex<RuntimeManager>>) -> Self {
        self.runtime_manager = Some(runtime_manager);

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
        let active_deployment_getter = self
            .active_deployment_getter
            .expect("an active deployment getter to be set");
        let artifacts_path = self.artifacts_path.expect("artifacts path to be set");
        let queue_client = self.queue_client.expect("a queue client to be set");
        let runtime_manager = self.runtime_manager.expect("a runtime manager to be set");
        let resource_manager = self.resource_manager.expect("a resource manager to be set");
        let logs_fetcher = self.logs_fetcher.expect("a logs fetcher to be set");
        let provisioner_client = self
            .provisioner_client
            .expect("a provisioner client to be set");

        let (queue_send, queue_recv) = mpsc::channel(QUEUE_BUFFER_SIZE);
        let (run_send, run_recv) = mpsc::channel(RUN_BUFFER_SIZE);

        let builds_path = artifacts_path.join("shuttle-builds");

        let run_send_clone = run_send.clone();
        let mut set = JoinSet::new();

        // Build queue. Waits for incoming deployments and builds them.
        set.spawn(queue::task(
            queue_recv,
            run_send_clone,
            build_log_recorder,
            queue_client,
            builds_path.clone(),
        ));
        // Run queue. Waits for built deployments and runs them.
        set.spawn(run::task(
            run_recv,
            runtime_manager.clone(),
            active_deployment_getter,
            resource_manager,
            builds_path.clone(),
            provisioner_client,
        ));

        DeploymentManager {
            queue_send,
            run_send,
            runtime_manager,
            logs_fetcher,
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
    logs_fetcher: logger::Client,
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
    pub fn builder<LR, ADG, RM, QC>() -> DeploymentManagerBuilder<LR, ADG, RM, QC> {
        DeploymentManagerBuilder {
            build_log_recorder: None,
            logs_fetcher: None,
            active_deployment_getter: None,
            artifacts_path: None,
            runtime_manager: None,
            resource_manager: None,
            queue_client: None,
            provisioner_client: None,
        }
    }

    pub async fn queue_push(&self, mut queued: Queued) {
        let cx = Span::current().context();

        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut queued.tracing_context);
        });

        self.queue_send.send(queued).await.unwrap();
    }

    #[instrument(name = "Starting deployment", skip(self), fields(deployment_id = %built.id, state = %State::Built))]
    pub async fn run_push(&self, built: Built) {
        self.run_send.send(built).await.unwrap();
    }

    #[instrument(name = "Killing deployment", skip(self), fields(deployment_id = %id, state = %State::Stopped))]
    pub async fn kill(&self, id: Uuid) {
        self.runtime_manager.lock().await.kill(&id).await;
    }

    pub fn builds_path(&self) -> &Path {
        self.builds_path.as_path()
    }

    pub fn logs_fetcher(&self) -> &logger::Client {
        &self.logs_fetcher
    }
}

type QueueSender = mpsc::Sender<queue::Queued>;
type QueueReceiver = mpsc::Receiver<queue::Queued>;

type RunSender = mpsc::Sender<run::Built>;
type RunReceiver = mpsc::Receiver<run::Built>;
