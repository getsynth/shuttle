use std::time::Duration;

use anyhow::{Context, Result};
use headers::{Authorization, HeaderMapExt};
use percent_encoding::utf8_percent_encode;
use reqwest::header::HeaderMap;
use reqwest::Response;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::{Deserialize, Serialize};
use shuttle_common::certificate::{
    AddCertificateRequest, CertificateListResponse, CertificateResponse, DeleteCertificateRequest,
};
use shuttle_common::log::{LogsRange, LogsResponseBeta};
use shuttle_common::models::deployment::{
    DeploymentListResponseBeta, DeploymentRequest, DeploymentRequestBeta, UploadArchiveResponseBeta,
};
use shuttle_common::models::project::{ProjectCreateRequestBeta, ProjectListResponseBeta};
use shuttle_common::models::{deployment, project, service, team, user};
use shuttle_common::resource::{
    ProvisionResourceRequestBeta, ResourceListResponseBeta, ResourceResponseBeta,
};
use shuttle_common::{resource, LogItem, VersionInfo};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

#[cfg(feature = "tracing")]
mod middleware;
#[cfg(feature = "tracing")]
use crate::middleware::LoggingMiddleware;
#[cfg(feature = "tracing")]
use tracing::{debug, error};

mod util;
use util::ToJson;

#[derive(Clone)]
pub struct ShuttleApiClient {
    pub client: ClientWithMiddleware,
    pub api_url: String,
    pub api_key: Option<String>,
}

impl ShuttleApiClient {
    pub fn new(api_url: String, api_key: Option<String>, headers: Option<HeaderMap>) -> Self {
        let mut builder = reqwest::Client::builder();
        if let Some(h) = headers {
            builder = builder.default_headers(h);
        }
        let client = builder.timeout(Duration::from_secs(60)).build().unwrap();

        let builder = reqwest_middleware::ClientBuilder::new(client);
        #[cfg(feature = "tracing")]
        let builder = builder.with(LoggingMiddleware);
        let client = builder.build();

        Self {
            client,
            api_url,
            api_key,
        }
    }

    pub fn set_auth_bearer(&self, builder: RequestBuilder) -> RequestBuilder {
        if let Some(ref api_key) = self.api_key {
            builder.bearer_auth(api_key)
        } else {
            builder
        }
    }

    pub async fn get_api_versions(&self) -> Result<VersionInfo> {
        let url = format!("{}/versions", self.api_url);

        self.client
            .get(url)
            .send()
            .await?
            .json()
            .await
            .context("parsing API version info")
    }

    pub async fn get_device_auth_ws(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        self.ws_get("/device-auth/ws".to_owned())
            .await
            .with_context(|| "failed to connect to auth endpoint")
    }

    pub async fn check_project_name(&self, project_name: &str) -> Result<bool> {
        let url = format!("{}/projects/name/{project_name}", self.api_url);

        self.client
            .get(url)
            .send()
            .await
            .context("failed to check project name availability")?
            .to_json()
            .await
            .context("parsing name check response")
    }

    pub async fn check_project_name_beta(&self, project_name: &str) -> Result<bool> {
        let url = format!("{}/projects/{project_name}/name", self.api_url);

        self.client
            .get(url)
            .send()
            .await
            .context("failed to check project name availability")?
            .to_json()
            .await
            .context("parsing name check response")
    }

    pub async fn get_current_user_beta(&self) -> Result<user::UserResponse> {
        self.get_json("/users/me".to_owned()).await
    }

    pub async fn deploy(
        &self,
        project: &str,
        deployment_req: DeploymentRequest,
    ) -> Result<deployment::Response> {
        let path = format!("/projects/{project}/services/{project}");
        let deployment_req = rmp_serde::to_vec(&deployment_req)
            .context("serialize DeploymentRequest as a MessagePack byte vector")?;

        let url = format!("{}{}", self.api_url, path);
        let mut builder = self.client.post(url);
        builder = self.set_auth_bearer(builder);

        builder
            .header("Transfer-Encoding", "chunked")
            .body(deployment_req)
            .send()
            .await
            .context("failed to send deployment to the Shuttle server")?
            .to_json()
            .await
    }

    pub async fn deploy_beta(
        &self,
        project: &str,
        deployment_req: DeploymentRequestBeta,
    ) -> Result<deployment::DeploymentResponseBeta> {
        let path = format!("/projects/{project}/deployments");
        self.post_json(path, Some(deployment_req)).await
    }

    pub async fn upload_archive_beta(
        &self,
        project: &str,
        data: Vec<u8>,
    ) -> Result<UploadArchiveResponseBeta> {
        let path = format!("/projects/{project}/archives");

        let url = format!("{}{}", self.api_url, path);
        let mut builder = self.client.post(url);
        builder = self.set_auth_bearer(builder);

        builder
            .body(data)
            .send()
            .await
            .context("failed to upload archive")?
            .to_json()
            .await
    }

    pub async fn redeploy_beta(
        &self,
        project: &str,
        deployment_id: &str,
    ) -> Result<deployment::DeploymentResponseBeta> {
        let path = format!("/projects/{project}/deployments/{deployment_id}/redeploy");

        self.post_json(path, Option::<()>::None).await
    }

    pub async fn stop_service(&self, project: &str) -> Result<service::Summary> {
        let path = format!("/projects/{project}/services/{project}");

        self.delete_json(path).await
    }

    pub async fn stop_service_beta(&self, project: &str) -> Result<String> {
        let path = format!("/projects/{project}/deployments");

        self.delete_json(path).await
    }

    pub async fn get_service(&self, project: &str) -> Result<service::Summary> {
        let path = format!("/projects/{project}/services/{project}");

        self.get_json(path).await
    }

    pub async fn get_service_resources(&self, project: &str) -> Result<Vec<resource::Response>> {
        self.get_json(format!("/projects/{project}/services/{project}/resources"))
            .await
    }
    pub async fn get_service_resources_beta(
        &self,
        project: &str,
    ) -> Result<ResourceListResponseBeta> {
        self.get_json(format!("/projects/{project}/resources"))
            .await
    }

    pub async fn dump_service_resource(
        &self,
        project: &str,
        resource_type: &resource::Type,
    ) -> Result<Vec<u8>> {
        let r#type = resource_type.to_string();
        let r#type = utf8_percent_encode(&r#type, percent_encoding::NON_ALPHANUMERIC).to_owned();

        let res = self
            .get(
                format!(
                    "/projects/{project}/services/{project}/resources/{}/dump",
                    r#type
                ),
                Option::<()>::None,
            )
            .await?;

        let bytes = res.bytes().await?;

        Ok(bytes.to_vec())
    }

    pub async fn delete_service_resource(
        &self,
        project: &str,
        resource_type: &resource::Type,
    ) -> Result<()> {
        let r#type = resource_type.to_string();
        let r#type = utf8_percent_encode(&r#type, percent_encoding::NON_ALPHANUMERIC).to_owned();

        self.delete_json(format!(
            "/projects/{project}/services/{project}/resources/{}",
            r#type
        ))
        .await
    }
    pub async fn delete_service_resource_beta(
        &self,
        project: &str,
        resource_type: &resource::Type,
    ) -> Result<String> {
        let r#type = resource_type.to_string();
        let r#type = utf8_percent_encode(&r#type, percent_encoding::NON_ALPHANUMERIC).to_owned();

        self.delete_json(format!("/projects/{project}/resources/{}", r#type))
            .await
    }
    pub async fn provision_resource_beta(
        &self,
        project: &str,
        req: ProvisionResourceRequestBeta,
    ) -> Result<ResourceResponseBeta> {
        self.post_json(format!("/projects/{project}/resources"), Some(req))
            .await
    }
    pub async fn get_secrets_beta(&self, project: &str) -> Result<ResourceResponseBeta> {
        self.get_json(format!("/projects/{project}/resources/secrets"))
            .await
    }

    pub async fn list_certificates_beta(&self, project: &str) -> Result<CertificateListResponse> {
        self.get_json(format!("/projects/{project}/certificates"))
            .await
    }
    pub async fn add_certificate_beta(
        &self,
        project: &str,
        subject: String,
    ) -> Result<CertificateResponse> {
        self.post_json(
            format!("/projects/{project}/certificates"),
            Some(AddCertificateRequest { subject }),
        )
        .await
    }
    pub async fn delete_certificate_beta(&self, project: &str, subject: String) -> Result<String> {
        self.delete_json_with_body(
            format!("/projects/{project}/certificates"),
            DeleteCertificateRequest { subject },
        )
        .await
    }

    pub async fn create_project(
        &self,
        project: &str,
        config: &project::Config,
    ) -> Result<project::Response> {
        self.post_json(format!("/projects/{project}"), Some(config))
            .await
            .context("failed to make create project request")
    }
    pub async fn create_project_beta(&self, name: &str) -> Result<project::ProjectResponseBeta> {
        self.post_json(
            "/projects",
            Some(ProjectCreateRequestBeta {
                name: name.to_string(),
            }),
        )
        .await
    }

    pub async fn clean_project(&self, project: &str) -> Result<String> {
        let path = format!("/projects/{project}/clean");

        self.post_json(path, Option::<String>::None)
            .await
            .context("failed to get clean output")
    }

    pub async fn get_project(&self, project: &str) -> Result<project::Response> {
        self.get_json(format!("/projects/{project}")).await
    }
    pub async fn get_project_beta(&self, project: &str) -> Result<project::ProjectResponseBeta> {
        self.get_json(format!("/projects/{project}")).await
    }

    pub async fn get_projects_list(&self) -> Result<Vec<project::Response>> {
        self.get_json("/projects".to_owned()).await
    }
    pub async fn get_projects_list_beta(&self) -> Result<ProjectListResponseBeta> {
        self.get_json("/projects".to_owned()).await
    }

    pub async fn update_project_beta(
        &self,
        project: &str,
        req: project::ProjectUpdateRequestBeta,
    ) -> Result<project::ProjectResponseBeta> {
        self.put_json(format!("/projects/{project}"), Some(req))
            .await
    }

    pub async fn stop_project(&self, project: &str) -> Result<project::Response> {
        let path = format!("/projects/{project}");

        self.delete_json(path).await
    }

    pub async fn delete_project(&self, project: &str) -> Result<String> {
        let path = format!("/projects/{project}/delete");
        let url = format!("{}{}", self.api_url, path);
        let mut builder = self.client.delete(url);
        builder = self.set_auth_bearer(builder);
        // project delete on alpha can take a while
        builder = builder.timeout(Duration::from_secs(60 * 5));

        builder
            .send()
            .await
            .context("failed to make delete request")?
            .to_json()
            .await
    }
    pub async fn delete_project_beta(&self, project: &str) -> Result<String> {
        self.delete_json(format!("/projects/{project}")).await
    }

    pub async fn get_teams_list(&self) -> Result<Vec<team::Response>> {
        self.get_json("/teams".to_string()).await
    }

    pub async fn get_team_projects_list(&self, team_id: &str) -> Result<Vec<project::Response>> {
        self.get_json(format!("/teams/{team_id}/projects")).await
    }
    pub async fn get_team_projects_list_beta(
        &self,
        team_id: &str,
    ) -> Result<ProjectListResponseBeta> {
        self.get_json(format!("/teams/{team_id}/projects")).await
    }

    pub async fn get_logs(
        &self,
        project: &str,
        deployment_id: &str,
        range: LogsRange,
    ) -> Result<Vec<LogItem>> {
        let mut path = format!("/projects/{project}/deployments/{deployment_id}/logs");
        Self::add_range_query(range, &mut path);

        self.get_json(path).await
    }
    pub async fn get_deployment_logs_beta(
        &self,
        project: &str,
        deployment_id: &str,
    ) -> Result<LogsResponseBeta> {
        let path = format!("/projects/{project}/deployments/{deployment_id}/logs");

        self.get_json(path).await
    }
    pub async fn get_project_logs_beta(&self, project: &str) -> Result<LogsResponseBeta> {
        let path = format!("/projects/{project}/logs");

        self.get_json(path).await
    }

    pub async fn get_logs_ws(
        &self,
        project: &str,
        deployment_id: &str,
        range: LogsRange,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let mut path = format!("/projects/{project}/ws/deployments/{deployment_id}/logs");
        Self::add_range_query(range, &mut path);

        self.ws_get(path).await
    }

    fn add_range_query(range: LogsRange, path: &mut String) {
        match range {
            LogsRange::Head(n) => {
                path.push_str("?head=");
                path.push_str(&n.to_string())
            }
            LogsRange::Tail(n) => {
                path.push_str("?tail=");
                path.push_str(&n.to_string())
            }
            _ => {}
        };
    }

    pub async fn get_deployments(
        &self,
        project: &str,
        page: u32,
        limit: u32,
    ) -> Result<Vec<deployment::Response>> {
        let path = format!(
            "/projects/{project}/deployments?page={}&limit={}",
            page.saturating_sub(1),
            limit,
        );

        self.get_json(path).await
    }
    pub async fn get_deployments_beta(
        &self,
        project: &str,
        page: i32,
        per_page: i32,
    ) -> Result<DeploymentListResponseBeta> {
        let path = format!(
            "/projects/{project}/deployments?page={}&per_page={}",
            page.saturating_sub(1).max(0),
            per_page.max(1),
        );

        self.get_json(path).await
    }
    pub async fn get_current_deployment_beta(
        &self,
        project: &str,
    ) -> Result<Option<deployment::DeploymentResponseBeta>> {
        let path = format!("/projects/{project}/deployments/current");

        self.get_json(path).await
    }

    pub async fn get_deployment_beta(
        &self,
        project: &str,
        deployment_id: &str,
    ) -> Result<deployment::DeploymentResponseBeta> {
        let path = format!("/projects/{project}/deployments/{deployment_id}");

        self.get_json(path).await
    }

    pub async fn get_deployment_details(
        &self,
        project: &str,
        deployment_id: &Uuid,
    ) -> Result<deployment::Response> {
        let path = format!("/projects/{project}/deployments/{deployment_id}");

        self.get_json(path).await
    }

    pub async fn reset_api_key(&self) -> Result<Response> {
        self.put("/users/reset-api-key", Option::<()>::None).await
    }

    pub async fn ws_get(&self, path: String) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let ws_url = self.api_url.clone().replace("http", "ws");
        let url = format!("{ws_url}{path}");
        let mut req = url.into_client_request()?;

        #[cfg(feature = "tracing")]
        debug!("WS Request: {} {}", req.method(), req.uri());

        if let Some(ref api_key) = self.api_key {
            let auth_header = Authorization::bearer(api_key.as_ref())?;
            req.headers_mut().typed_insert(auth_header);
        }

        let (stream, _) = connect_async(req).await.with_context(|| {
            #[cfg(feature = "tracing")]
            error!("failed to connect to websocket");
            "could not connect to websocket"
        })?;

        Ok(stream)
    }

    pub async fn get<T: Serialize>(
        &self,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.api_url, path.as_ref());

        let mut builder = self.client.get(url);
        builder = self.set_auth_bearer(builder);

        if let Some(body) = body {
            let body = serde_json::to_string(&body)?;
            #[cfg(feature = "tracing")]
            debug!("Outgoing body: {}", body);
            builder = builder.body(body);
            builder = builder.header("Content-Type", "application/json");
        }

        Ok(builder.send().await?)
    }

    pub async fn get_json<R>(&self, path: impl AsRef<str>) -> Result<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        self.get(path, Option::<()>::None).await?.to_json().await
    }

    pub async fn get_json_with_body<R, T: Serialize>(
        &self,
        path: impl AsRef<str>,
        body: T,
    ) -> Result<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        self.get(path, Some(body)).await?.to_json().await
    }

    pub async fn post<T: Serialize>(
        &self,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.api_url, path.as_ref());

        let mut builder = self.client.post(url);
        builder = self.set_auth_bearer(builder);

        if let Some(body) = body {
            let body = serde_json::to_string(&body)?;
            #[cfg(feature = "tracing")]
            debug!("Outgoing body: {}", body);
            builder = builder.body(body);
            builder = builder.header("Content-Type", "application/json");
        }

        Ok(builder.send().await?)
    }

    pub async fn post_json<T: Serialize, R>(
        &self,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        self.post(path, body).await?.to_json().await
    }

    pub async fn put<T: Serialize>(
        &self,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.api_url, path.as_ref());

        let mut builder = self.client.put(url);
        builder = self.set_auth_bearer(builder);

        if let Some(body) = body {
            let body = serde_json::to_string(&body)?;
            #[cfg(feature = "tracing")]
            debug!("Outgoing body: {}", body);
            builder = builder.body(body);
            builder = builder.header("Content-Type", "application/json");
        }

        Ok(builder.send().await?)
    }

    pub async fn put_json<T: Serialize, R>(
        &self,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        self.put(path, body).await?.to_json().await
    }

    pub async fn delete<T: Serialize>(
        &self,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.api_url, path.as_ref());

        let mut builder = self.client.delete(url);
        builder = self.set_auth_bearer(builder);

        if let Some(body) = body {
            let body = serde_json::to_string(&body)?;
            #[cfg(feature = "tracing")]
            debug!("Outgoing body: {}", body);
            builder = builder.body(body);
            builder = builder.header("Content-Type", "application/json");
        }

        Ok(builder.send().await?)
    }

    pub async fn delete_json<R>(&self, path: impl AsRef<str>) -> Result<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        self.delete(path, Option::<()>::None).await?.to_json().await
    }

    pub async fn delete_json_with_body<R, T: Serialize>(
        &self,
        path: impl AsRef<str>,
        body: T,
    ) -> Result<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        self.delete(path, Some(body)).await?.to_json().await
    }
}
