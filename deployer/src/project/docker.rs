use async_trait::async_trait;
use bollard::{errors::Error as DockerError, service::ContainerInspectResponse, Docker};
use shuttle_common::models::project::IDLE_MINUTES;
use ulid::Ulid;

use crate::safe_unwrap;

use super::service::state::errored::ServiceErrored;
use super::service::state::machine::Refresh;

pub struct ContainerSettingsBuilder {
    prefix: Option<String>,
    image: Option<String>,
    provisioner: Option<String>,
    auth_uri: Option<String>,
    network_name: Option<String>,
}

impl Default for ContainerSettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerSettingsBuilder {
    pub fn new() -> Self {
        Self {
            prefix: None,
            image: None,
            provisioner: None,
            auth_uri: None,
            network_name: None,
        }
    }

    pub fn prefix<S: ToString>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    pub fn image<S: ToString>(mut self, image: S) -> Self {
        self.image = Some(image.to_string());
        self
    }

    pub fn provisioner_host<S: ToString>(mut self, host: S) -> Self {
        self.provisioner = Some(host.to_string());
        self
    }

    pub fn auth_uri<S: ToString>(mut self, auth_uri: S) -> Self {
        self.auth_uri = Some(auth_uri.to_string());
        self
    }

    pub fn network_name<S: ToString>(mut self, name: S) -> Self {
        self.network_name = Some(name.to_string());
        self
    }

    pub async fn build(mut self) -> ContainerSettings {
        let prefix = self.prefix.take().unwrap();
        let image = self.image.take().unwrap();
        let provisioner_host = self.provisioner.take().unwrap();
        let auth_uri = self.auth_uri.take().unwrap();
        let network_name = self.network_name.take().unwrap();

        ContainerSettings {
            prefix,
            image,
            provisioner_host,
            auth_uri,
            network_name,
        }
    }
}

#[derive(Clone)]
pub struct ContainerSettings {
    pub prefix: String,
    pub image: String,
    pub provisioner_host: String,
    pub auth_uri: String,
    pub network_name: String,
}

impl ContainerSettings {
    pub fn builder() -> ContainerSettingsBuilder {
        ContainerSettingsBuilder::new()
    }
}

#[derive(Clone)]
pub struct ServiceDockerContext {
    docker: Docker,
    settings: ContainerSettings,
}

impl ServiceDockerContext {
    pub fn new(docker: Docker, cs: ContainerSettings) -> Self {
        Self {
            docker,
            settings: cs,
        }
    }
}

impl DockerContext for ServiceDockerContext {
    fn docker(&self) -> &Docker {
        &self.docker
    }

    fn container_settings(&self) -> &ContainerSettings {
        &self.settings
    }
}

pub trait DockerContext: Send + Sync {
    fn docker(&self) -> &Docker;

    fn container_settings(&self) -> &ContainerSettings;
}

#[async_trait]
impl<Ctx> Refresh<Ctx> for ContainerInspectResponse
where
    Ctx: DockerContext,
{
    type Error = DockerError;
    async fn refresh(self, ctx: &Ctx) -> Result<Self, Self::Error> {
        ctx.docker()
            .inspect_container(self.id.as_ref().unwrap(), None)
            .await
    }
}

pub trait ContainerInspectResponseExt {
    fn container(&self) -> &ContainerInspectResponse;

    fn service_id(&self) -> Result<Ulid, ServiceErrored> {
        let container = self.container();

        Ulid::from_string(safe_unwrap!(container
            .config
            .labels
            .get("shuttle.service_id")))
        .map_err(|err| ServiceErrored::internal(err.to_string()))
    }

    fn idle_minutes(&self) -> u64 {
        let container = self.container();

        if let Some(config) = &container.config {
            if let Some(labels) = &config.labels {
                if let Some(idle_minutes) = labels.get("shuttle.idle_minutes") {
                    return idle_minutes.parse::<u64>().unwrap_or(IDLE_MINUTES);
                }
            }
        }

        IDLE_MINUTES
    }

    fn find_arg_and_then<'s, F, O>(&'s self, find: &str, and_then: F) -> Result<O, ServiceErrored>
    where
        F: FnOnce(&'s str) -> O,
        O: 's,
    {
        let mut args = self.args()?.iter();
        let out = if args.any(|arg| arg.as_str() == find) {
            args.next().map(|s| and_then(s.as_str()))
        } else {
            None
        };
        out.ok_or_else(|| ServiceErrored::internal(format!("no such argument: {find}")))
    }

    fn args(&self) -> Result<&Vec<String>, ServiceErrored> {
        let container = self.container();
        Ok(safe_unwrap!(container.args))
    }
}

impl ContainerInspectResponseExt for ContainerInspectResponse {
    fn container(&self) -> &ContainerInspectResponse {
        self
    }
}
