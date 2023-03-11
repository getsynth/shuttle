use std::{convert::TryInto, path::PathBuf, sync::Arc};

use anyhow::Context;
use shuttle_proto::runtime::{
    self, runtime_client::RuntimeClient, StopRequest, SubscribeLogsRequest,
};
use tokio::{process, sync::Mutex};
use tonic::transport::Channel;
use tracing::{info, instrument, trace};
use uuid::Uuid;

use crate::deployment::deploy_layer;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[derive(Clone)]
pub struct RuntimeManager {
    legacy: Option<RuntimeClient<Channel>>,
    legacy_process: Option<Arc<std::sync::Mutex<process::Child>>>,
    next: Option<RuntimeClient<Channel>>,
    next_process: Option<Arc<std::sync::Mutex<process::Child>>>,
    artifacts_path: PathBuf,
    provisioner_address: String,
    log_sender: crossbeam_channel::Sender<deploy_layer::Log>,
}

impl RuntimeManager {
    pub fn new(
        artifacts_path: PathBuf,
        provisioner_address: String,
        log_sender: crossbeam_channel::Sender<deploy_layer::Log>,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            legacy: None,
            legacy_process: None,
            next: None,
            next_process: None,
            artifacts_path,
            provisioner_address,
            log_sender,
        }))
    }

    pub async fn get_runtime_client(
        &mut self,
        is_next: bool,
    ) -> anyhow::Result<RuntimeClient<Channel>> {
        if is_next {
            Self::get_runtime_client_helper(
                &mut self.next,
                &mut self.next_process,
                is_next,
                self.artifacts_path.clone(),
                &self.provisioner_address,
                self.log_sender.clone(),
            )
            .await
        } else {
            Self::get_runtime_client_helper(
                &mut self.legacy,
                &mut self.legacy_process,
                is_next,
                self.artifacts_path.clone(),
                &self.provisioner_address,
                self.log_sender.clone(),
            )
            .await
        }
    }

    /// Send a kill / stop signal for a deployment to any runtimes currently running
    pub async fn kill(&mut self, id: &Uuid) -> bool {
        let success_legacy = if let Some(legacy_client) = &mut self.legacy {
            trace!(%id, "sending stop signal to legacy for deployment");

            let stop_request = tonic::Request::new(StopRequest {});
            let response = legacy_client.stop(stop_request).await.unwrap();

            response.into_inner().success
        } else {
            trace!("no legacy client running");
            true
        };

        let success_next = if let Some(next_client) = &mut self.next {
            trace!(%id, "sending stop signal to next for deployment");

            let stop_request = tonic::Request::new(StopRequest {});
            let response = next_client.stop(stop_request).await.unwrap();

            response.into_inner().success
        } else {
            trace!("no next client running");
            true
        };

        success_legacy && success_next
    }

    #[instrument(skip(runtime_option, process_option, log_sender))]
    async fn get_runtime_client_helper(
        runtime_option: &mut Option<RuntimeClient<Channel>>,
        process_option: &mut Option<Arc<std::sync::Mutex<process::Child>>>,
        is_next: bool,
        artifacts_path: PathBuf,
        provisioner_address: &str,
        log_sender: crossbeam_channel::Sender<deploy_layer::Log>,
    ) -> anyhow::Result<RuntimeClient<Channel>> {
        if let Some(runtime_client) = runtime_option {
            trace!("returning previous client");
            Ok(runtime_client.clone())
        } else {
            trace!("making new client");

            let port = portpicker::pick_unused_port().context("failed to find available port")?;

            let get_runtime_executable = || {
                if cfg!(debug_assertions) {
                    // If we're running deployer natively, install shuttle-runtime using the
                    // version of runtime from the calling repo.
                    let path = std::fs::canonicalize(format!("{MANIFEST_DIR}/../runtime"));

                    // The path will not be valid if we are in a deployer container, in which
                    // case we don't try to install and use the one installed in deploy.sh.
                    if let Ok(path) = path {
                        std::process::Command::new("cargo")
                            .arg("install")
                            .arg("shuttle-runtime")
                            .arg("--path")
                            .arg(path)
                            .output()
                            .expect("failed to install the local version of shuttle-runtime");
                    }
                }

                // If we're in a deployer built with the containerfile, the runtime will have
                // been installed in deploy.sh.
                home::cargo_home()
                    .expect("failed to find path to cargo home")
                    .join("bin/shuttle-runtime")
            };

            let (process, runtime_client) = runtime::start(
                is_next,
                runtime::StorageManagerType::Artifacts(artifacts_path),
                provisioner_address,
                port,
                get_runtime_executable,
            )
            .await
            .context("failed to start shuttle runtime")?;

            let sender = log_sender;
            let mut stream = runtime_client
                .clone()
                .subscribe_logs(tonic::Request::new(SubscribeLogsRequest {}))
                .await
                .context("subscribing to runtime logs stream")?
                .into_inner();

            tokio::spawn(async move {
                while let Ok(Some(log)) = stream.message().await {
                    if let Ok(log) = log.try_into() {
                        sender.send(log).expect("to send log to persistence");
                    }
                }
            });

            *runtime_option = Some(runtime_client.clone());
            *process_option = Some(Arc::new(std::sync::Mutex::new(process)));

            // Safe to unwrap as it was just set
            Ok(runtime_client)
        }
    }
}

impl Drop for RuntimeManager {
    fn drop(&mut self) {
        info!("runtime manager shutting down");

        if let Some(ref process) = self.legacy_process.take() {
            let _ = process.lock().unwrap().start_kill();
        }

        if let Some(ref process) = self.next_process.take() {
            let _ = process.lock().unwrap().start_kill();
        }
    }
}
