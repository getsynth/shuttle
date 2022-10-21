use std::fmt::Debug;
use std::sync::Arc;

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::{debug, info};

use crate::project::Project;
use crate::service::GatewayService;
use crate::{AccountName, Context, EndState, Error, ProjectName, Refresh, Service, State};

#[must_use]
#[derive(Debug, Clone)]
pub struct Work<W = Project> {
    pub project_name: ProjectName,
    pub account_name: AccountName,
    pub work: W,
}

#[async_trait]
impl<W> Refresh for Work<W>
where
    W: Refresh + Send,
{
    type Error = W::Error;

    async fn refresh<'c, C: Context<'c>>(self, ctx: &C) -> Result<Self, Self::Error> {
        Ok(Self {
            project_name: self.project_name,
            account_name: self.account_name,
            work: self.work.refresh(ctx).await?,
        })
    }
}

#[async_trait]
impl<'c, W> State<'c> for Work<W>
where
    W: State<'c>,
{
    type Next = Work<W::Next>;

    type Error = W::Error;

    async fn next<C: Context<'c>>(self, ctx: &C) -> Result<Self::Next, Self::Error> {
        Ok(Work::<W::Next> {
            project_name: self.project_name,
            account_name: self.account_name,
            work: self.work.next(ctx).await?,
        })
    }
}

impl<'c, W> EndState<'c> for Work<W>
where
    W: EndState<'c>,
{
    type ErrorVariant = W::ErrorVariant;

    fn is_done(&self) -> bool {
        self.work.is_done()
    }

    fn into_result(self) -> Result<Self, Self::ErrorVariant> {
        Ok(Self {
            project_name: self.project_name,
            account_name: self.account_name,
            work: self.work.into_result()?,
        })
    }
}

pub struct Worker<Svc = Arc<GatewayService>, W = Work> {
    service: Svc,
    send: Option<Sender<W>>,
    recv: Receiver<W>,
}

impl<Svc, W> Worker<Svc, W>
where
    W: Send,
{
    pub fn new(service: Svc) -> Self {
        let (send, recv) = channel(32);
        Self {
            service,
            send: Some(send),
            recv,
        }
    }

    /// Returns a [Sender] to push work to this worker.
    ///
    /// # Panics
    /// If this worker has already started.
    pub fn sender(&self) -> Sender<W> {
        Sender::clone(self.send.as_ref().unwrap())
    }
}

impl<Svc, W> Worker<Svc, W>
where
    Svc: for<'c> Service<'c, State = W, Error = Error>,
    W: Debug + Send + for<'c> EndState<'c>,
{
    /// Starts the worker, waiting and processing elements from the
    /// queue until the last sending end for the channel is dropped,
    /// at which point this future resolves.
    ///
    /// # Panics
    /// If this worker has already started.
    pub async fn start(mut self) -> Result<Self, Error> {
        // Drop the self-sender owned by this worker to prevent a
        // deadlock if all the other senders have already been dropped
        // at this point.
        let _ = self.send.take().unwrap();
        debug!("starting worker");

        while let Some(work) = self.recv.recv().await {
            debug!(?work, "received work");
            do_work(work, &self.service).await;
        }

        Ok(self)
    }
}

pub async fn do_work<
    'c,
    E: std::fmt::Display,
    S: Service<'c, State = W, Error = E>,
    W: EndState<'c> + Debug,
>(
    mut work: W,
    service: &'c S,
) {
    loop {
        work = {
            let context = service.context();

            // Safety: EndState's transitions are Infallible
            work.next(&context).await.unwrap()
        };

        match service.update(&work).await {
            Ok(_) => {}
            Err(err) => info!("failed to update a state: {}\nstate: {:?}", err, work),
        };

        if work.is_done() {
            break;
        } else {
            debug!(?work, "work not done yet");
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::convert::Infallible;

    use anyhow::anyhow;
    use tokio::sync::Mutex;

    use super::*;
    use crate::tests::{World, WorldContext};

    pub struct DummyService<S> {
        world: World,
        state: Mutex<Option<S>>,
    }

    impl DummyService<()> {
        pub async fn new<S>() -> DummyService<S> {
            let world = World::new().await;
            DummyService {
                world,
                state: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl<'c, S> Service<'c> for DummyService<S>
    where
        S: EndState<'c> + Sync,
    {
        type Context = WorldContext<'c>;

        type State = S;

        type Error = Error;

        fn context(&'c self) -> Self::Context {
            self.world.context()
        }

        async fn update(&self, state: &Self::State) -> Result<(), Self::Error> {
            let mut lock = self.state.lock().await;
            *lock = Some(Self::State::clone(state));
            Ok(())
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct FiniteState {
        count: usize,
        max_count: usize,
    }

    #[async_trait]
    impl<'c> State<'c> for FiniteState {
        type Next = Self;

        type Error = Infallible;

        async fn next<C: Context<'c>>(mut self, _ctx: &C) -> Result<Self::Next, Self::Error> {
            if self.count < self.max_count {
                self.count += 1;
            }
            Ok(self)
        }
    }

    impl<'c> EndState<'c> for FiniteState {
        type ErrorVariant = anyhow::Error;

        fn is_done(&self) -> bool {
            self.count == self.max_count
        }

        fn into_result(self) -> Result<Self, Self::ErrorVariant> {
            if self.count > self.max_count {
                Err(anyhow!(
                    "count is over max_count: {} > {}",
                    self.count,
                    self.max_count
                ))
            } else {
                Ok(self)
            }
        }
    }

    #[tokio::test]
    async fn worker_queue_and_proceed_until_done() {
        let svc = DummyService::new::<FiniteState>().await;

        let worker = Worker::new(svc);

        {
            let sender = worker.sender();

            let state = FiniteState {
                count: 0,
                max_count: 42,
            };

            sender.send(state).await.unwrap();
        }

        let Worker {
            service: DummyService { state, .. },
            ..
        } = worker.start().await.unwrap();

        assert_eq!(
            *state.lock().await,
            Some(FiniteState {
                count: 42,
                max_count: 42
            })
        );
    }
}
