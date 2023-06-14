use std::{collections::HashMap, net::Ipv4Addr, sync::Arc, time::Duration};

use anyhow::Context;
use shuttle_common::claims::{ClaimLayer, ClaimService, InjectPropagation, InjectPropagationLayer};
use shuttle_proto::runtime::{
    runtime_client::{self, RuntimeClient},
    Ping, StopRequest, SubscribeLogsRequest,
};
use tokio::sync::Mutex;
use tonic::transport::{Channel, Endpoint};
use tower::ServiceBuilder;
use tracing::trace;
use ulid::Ulid;

use crate::{deployment::deploy_layer, project::service::RUNTIME_API_PORT};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

type Runtimes =
    Arc<tokio::sync::Mutex<HashMap<Ulid, RuntimeClient<ClaimService<InjectPropagation<Channel>>>>>>;

/// Manager that can start up mutliple runtimes. This is needed so that two runtimes can be up when a new deployment is made:
/// One runtime for the new deployment being loaded; another for the currently active deployment
#[derive(Clone)]
pub struct RuntimeManager {
    runtimes: Runtimes,
    log_sender: crossbeam_channel::Sender<deploy_layer::Log>,
}

impl RuntimeManager {
    pub fn new(log_sender: crossbeam_channel::Sender<deploy_layer::Log>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            runtimes: Default::default(),
            log_sender,
        }))
    }

    pub async fn runtime_client(
        &mut self,
        id: Ulid,
        target_ip: Ipv4Addr,
    ) -> anyhow::Result<RuntimeClient<ClaimService<InjectPropagation<Channel>>>> {
        trace!("making new client");
        let guard = self.runtimes.lock().await;

        if let Some(runtime_client) = guard.get(&id) {
            return Ok(runtime_client.clone());
        }

        // Connection to the docker container where the shuttle-runtime lives.
        let conn = Endpoint::new(format!("http://{target_ip}:{RUNTIME_API_PORT}"))
            .context("creating runtime client endpoint")?
            .connect_timeout(Duration::from_secs(5));

        let channel = conn.connect().await.context("connecting runtime client")?;
        let channel = ServiceBuilder::new()
            .layer(ClaimLayer)
            .layer(InjectPropagationLayer)
            .service(channel);
        let runtime_client = runtime_client::RuntimeClient::new(channel);
        let sender = self.log_sender.clone();
        let mut stream = runtime_client
            .clone()
            .subscribe_logs(tonic::Request::new(SubscribeLogsRequest {}))
            .await
            .context("subscribing to runtime logs stream")?
            .into_inner();

        tokio::spawn(async move {
            while let Ok(Some(log)) = stream.message().await {
                if let Ok(mut log) = deploy_layer::Log::try_from(log) {
                    log.id = id;
                    sender.send(log).expect("to send log to persistence");
                }
            }
        });

        self.runtimes
            .lock()
            .await
            .insert(id, runtime_client.clone());

        Ok(runtime_client)
    }

    /// Send a kill / stop signal for a deployment to its running runtime
    pub async fn kill(&mut self, id: &Ulid) -> bool {
        let value = self.runtimes.lock().await.remove(id);

        if let Some(mut runtime_client) = value {
            trace!(%id, "sending stop signal for deployment");

            let stop_request = tonic::Request::new(StopRequest {});
            let response = runtime_client.stop(stop_request).await.unwrap();

            trace!(?response, "stop deployment response");

            let result = response.into_inner().success;
            result
        } else {
            trace!("no client running");
            true
        }
    }

    pub async fn is_healthy(&self, id: &Ulid) -> bool {
        let mut guard = self.runtimes.lock().await;

        if let Some(runtime_client) = guard.get_mut(id) {
            trace!(%id, "sending ping to the runtime");

            let ping = tonic::Request::new(Ping {});
            let response = runtime_client.health_check(ping).await;
            match response {
                Ok(inner) => {
                    trace!("runtime responded with pong");
                    true
                }
                Err(status) => {
                    trace!(?status, "health check failed");
                    false
                }
            }
        } else {
            trace!("no client running");
            false
        }
    }
}
