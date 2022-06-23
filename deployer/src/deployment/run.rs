use tracing::{debug, error, info, instrument};

use super::{KillReceiver, KillSender, RunReceiver, State};
use crate::error::Result;

pub async fn task(mut recv: RunReceiver, kill_send: KillSender) {
    info!("Run task started");

    while let Some(built) = recv.recv().await {
        let name = built.name.clone();

        info!("Built deployment at the front of run queue: {}", name);

        let kill_recv = kill_send.subscribe();

        tokio::spawn(async move {
            if let Err(e) = built.handle(kill_recv).await {
                error!("Error during running of deployment '{}' - {e}", name);
            }
        });
    }
}

#[derive(Debug)]
pub struct Built {
    pub name: String,
}

impl Built {
    #[instrument(skip(self), fields(name = self.name.as_str(), state = %State::Running))]
    async fn handle(self, mut kill_recv: KillReceiver) -> Result<()> {
        // Load service into memory:
        // TODO
        let mut execute_future = Box::pin(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }); // placeholder

        // Execute loaded service:

        loop {
            tokio::select! {
                Ok(name) = kill_recv.recv() => {
                    if name == self.name {
                        debug!("Service {name} killed");
                        break;
                    }
                }
                _ = &mut execute_future => {}
            }
        }

        Ok(())
    }
}
