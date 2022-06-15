mod info;
mod queue;
mod run;
mod states;

pub use info::DeploymentInfo;
pub use states::DeploymentState;

pub use queue::Queued;
pub use run::Built;

use crate::persistence::Persistence;

use tokio::sync::mpsc;

const QUEUE_BUFFER_SIZE: usize = 100;
const RUN_BUFFER_SIZE: usize = 100;

#[derive(Clone)]
pub struct DeploymentManager {
    queue_send: QueueSender,
    run_send: RunSender,
}

impl DeploymentManager {
    /// Create a new deployment manager. Creates two Tokio tasks, one for
    /// building queued services, the other for executing/deploying built
    /// services. Two multi-producer, single consumer channels are also created
    /// which are moving service deployments between the aforementioned tasks.
    ///
    /// ```
    /// queue channel   all deployments here are DeploymentState::Queued
    ///       |
    ///       v
    ///  queue task     when taken from the channel by this task, deployments
    ///                 enter the DeploymentState::Building state and upon being
    ///       |         built transition to the DeploymentState::Built state
    ///       v
    ///  run channel    all deployments here are DeploymentState::Built
    ///       |
    ///       v
    ///    run task     tasks enter the DeploymentState::Running state and begin
    ///                 executing
    /// ```
    pub fn new(persistence: Persistence) -> Self {
        let (queue_send, queue_recv) = mpsc::channel(QUEUE_BUFFER_SIZE);
        let (run_send, run_recv) = mpsc::channel(RUN_BUFFER_SIZE);

        tokio::spawn(async move { queue::task(queue_recv, persistence).await });
        tokio::spawn(async move { run::task(run_recv).await });

        DeploymentManager {
            queue_send,
            run_send,
        }
    }

    pub async fn queue_push(&self, queued: Queued) {
        self.queue_send.send(queued).await.unwrap();
    }

    pub async fn run_push(&self, built: Built) {
        self.run_send.send(built).await.unwrap();
    }
}

type QueueSender = mpsc::Sender<queue::Queued>;
type QueueReceiver = mpsc::Receiver<queue::Queued>;

type RunSender = mpsc::Sender<run::Built>;
type RunReceiver = mpsc::Receiver<run::Built>;
