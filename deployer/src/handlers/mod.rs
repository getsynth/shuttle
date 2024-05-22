use std::str::FromStr;

use anyhow::anyhow;
use async_trait::async_trait;
use axum::body::Bytes;
use axum::extract::{
    ws::{self, WebSocket},
    FromRequest,
};
use axum::extract::{DefaultBodyLimit, Extension, Query};
use axum::handler::Handler;
use axum::middleware::{self, from_extractor};
use axum::routing::{delete, get, post, Router};
use axum::Json;
use chrono::{SecondsFormat, Utc};
use hyper::{Request, StatusCode, Uri};
use serde::{de::DeserializeOwned, Deserialize};
use shuttle_service::builder::clean_crate;
use tonic::Code;
use tracing::{error, field, info, info_span, instrument, trace, warn};
use uuid::Uuid;

use shuttle_backends::{
    auth::{AdminSecretLayer, AuthPublicKey, JwtAuthenticationLayer, ScopedLayer},
    axum::CustomErrorPath,
    metrics::{Metrics, TraceLayer},
    request_span,
};
use shuttle_common::{
    claims::{Claim, Scope},
    models::deployment::{DeploymentRequest, CREATE_SERVICE_BODY_LIMIT, GIT_STRINGS_MAX_LENGTH},
    LogItem,
};
use shuttle_proto::logger::LogsRequest;

use crate::{
    deployment::{DeploymentManager, Queued},
    persistence::{resource::ResourceManager, Deployment, Persistence, State},
};
pub use {self::error::Error, self::error::Result, self::local::set_jwt_bearer};

mod error;
mod local;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PaginationDetails {
    /// Page to fetch, starting from 0.
    pub page: Option<u32>,
    /// Number of results per page.
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
struct LogsQuery {
    head: Option<u32>,
    tail: Option<u32>,
}
#[derive(Clone)]
pub struct RouterBuilder {
    router: Router,
    // might be used for tracing instruments?
    _project_name: String,
    auth_uri: Uri,
}

impl RouterBuilder {
    pub fn new(
        persistence: Persistence,
        deployment_manager: DeploymentManager,
        project_name: String,
        auth_uri: Uri,
    ) -> Self {
        let router = Router::new()
            .route(
                "/projects/:project_name/services",
                get(get_services.layer(ScopedLayer::new(vec![Scope::Service]))),
            )
            .route(
                "/projects/:project_name/services/:service_name",
                get(get_service.layer(ScopedLayer::new(vec![Scope::Service])))
                    .post(
                        create_service
                            .layer(DefaultBodyLimit::max(CREATE_SERVICE_BODY_LIMIT))
                            .layer(ScopedLayer::new(vec![Scope::ServiceCreate])),
                    )
                    .delete(stop_service.layer(ScopedLayer::new(vec![Scope::ServiceCreate]))),
            )
            .route(
                "/projects/:project_name/services/:service_name/resources",
                get(get_service_resources).layer(ScopedLayer::new(vec![Scope::Resources])),
            )
            .route(
                "/projects/:project_name/services/:service_name/resources/:resource_type",
                delete(delete_service_resource)
                    .layer(ScopedLayer::new(vec![Scope::ResourcesWrite])),
            )
            .route(
                "/projects/:project_name/deployments",
                get(get_deployments).layer(ScopedLayer::new(vec![Scope::Service])),
            )
            .route(
                "/projects/:project_name/deployments/:deployment_id",
                get(get_deployment.layer(ScopedLayer::new(vec![Scope::Deployment])))
                    .delete(delete_deployment.layer(ScopedLayer::new(vec![Scope::DeploymentPush])))
                    // Deprecated.
                    // Deployer now always starts the last running deployment on start up / wake up.
                    // This is kept for compatibility.
                    .put(|| async move {}),
            )
            .route(
                "/projects/:project_name/ws/deployments/:deployment_id/logs",
                get(get_logs_subscribe.layer(ScopedLayer::new(vec![Scope::Logs]))),
            )
            .route(
                "/projects/:project_name/deployments/:deployment_id/logs",
                get(get_logs.layer(ScopedLayer::new(vec![Scope::Logs]))),
            )
            .route(
                "/projects/:project_name/clean",
                post(clean_project.layer(ScopedLayer::new(vec![Scope::DeploymentPush]))),
            )
            .layer(Extension(persistence))
            .layer(Extension(deployment_manager))
            .layer(JwtAuthenticationLayer::new(AuthPublicKey::new(
                auth_uri.clone(),
            )));

        Self {
            router,
            _project_name: project_name,
            auth_uri,
        }
    }

    pub fn with_admin_secret_layer(mut self, admin_secret: String) -> Self {
        self.router = self.router.layer(AdminSecretLayer::new(admin_secret));

        self
    }

    /// Sets an admin JWT bearer token on every request for use when running deployer locally.
    pub fn with_local_admin_layer(mut self) -> Self {
        warn!("Building deployer router with auth bypassed, this should only be used for local development.");
        self.router = self
            .router
            .layer(middleware::from_fn(set_jwt_bearer))
            .layer(Extension(self.auth_uri.clone()));

        self
    }

    pub fn into_router(self) -> Router {
        self.router
            .route("/projects/:project_name/status", get(|| async { "Ok" }))
            .route_layer(from_extractor::<Metrics>())
            .layer(
                TraceLayer::new(|request| {
                    request_span!(
                        request,
                        account.user_id = field::Empty,
                        request.params.project_name = field::Empty,
                        request.params.service_name = field::Empty,
                        request.params.deployment_id = field::Empty,
                    )
                })
                .with_propagation()
                .build(),
            )
    }
}

#[instrument(skip_all)]
pub async fn get_services(
    Extension(persistence): Extension<Persistence>,
) -> Result<Json<Vec<shuttle_common::models::service::Response>>> {
    let services = persistence
        .get_all_services()
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(services))
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name, shuttle.service.name = %service_name))]
pub async fn get_service(
    Extension(persistence): Extension<Persistence>,
    CustomErrorPath((project_name, service_name)): CustomErrorPath<(String, String)>,
) -> Result<Json<shuttle_common::models::service::Summary>> {
    let service = persistence
        .get_service_by_name(&service_name)
        .await?
        .ok_or_else(|| Error::NotFound("service not found".to_string()))?;

    let deployment = persistence
        .get_active_deployment(&service.id)
        .await?
        .map(Into::into);
    let response = shuttle_common::models::service::Summary {
        name: service.name,
        deployment,
        uri: "".to_owned(),
    };

    Ok(Json(response))
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name, shuttle.service.name = %service_name))]
pub async fn get_service_resources(
    Extension(mut persistence): Extension<Persistence>,
    Extension(claim): Extension<Claim>,
    CustomErrorPath((project_name, service_name)): CustomErrorPath<(String, String)>,
) -> Result<Json<Vec<shuttle_common::resource::Response>>> {
    let service = persistence
        .get_service_by_name(&service_name)
        .await?
        .ok_or_else(|| Error::NotFound("service not found".to_string()))?;

    let resources = persistence
        .get_resources(&service.id, claim)
        .await?
        .resources
        .into_iter()
        .map(shuttle_common::resource::Response::try_from)
        // We ignore and trace the errors for resources with corrupted data, returning just the
        // valid resources.
        // TODO: investigate how the resource data can get corrupted.
        .filter_map(|resource| {
            resource
                .map_err(|err| {
                    error!(
                        error = err.as_ref() as &dyn std::error::Error,
                        "failed to parse resource data"
                    );
                })
                .ok()
        })
        .collect();

    Ok(Json(resources))
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name, shuttle.service.name = %service_name, %resource_type))]
pub async fn delete_service_resource(
    Extension(mut persistence): Extension<Persistence>,
    Extension(claim): Extension<Claim>,
    CustomErrorPath((project_name, service_name, resource_type)): CustomErrorPath<(
        String,
        String,
        String,
    )>,
) -> Result<Json<()>> {
    let service = persistence
        .get_service_by_name(&service_name)
        .await?
        .ok_or_else(|| Error::NotFound("service not found".to_string()))?;

    let r#type =
        shuttle_common::resource::Type::from_str(resource_type.as_str()).map_err(|err| {
            error::Error::Convert {
                from: "str".to_string(),
                to: "shuttle_common::resource::Type".to_string(),
                message: format!("Not a valid resource type representation: {}", err).to_string(),
            }
        })?;

    let get_resource_response = persistence
        .get_resource(&service.id, r#type, claim.clone())
        .await?;

    if get_resource_response.resource.is_none() {
        return Err(Error::NotFound("resource not found".to_string()));
    }

    let delete_resource_response = persistence
        .delete_resource(project_name, &service.id, r#type, claim)
        .await?;

    if !delete_resource_response.success {
        return Err(anyhow!("Unable to delete resource from resource recorder").into());
    }

    Ok(Json(()))
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name, shuttle.service.name = %service_name))]
pub async fn create_service(
    Extension(persistence): Extension<Persistence>,
    Extension(deployment_manager): Extension<DeploymentManager>,
    Extension(claim): Extension<Claim>,
    CustomErrorPath((project_name, service_name)): CustomErrorPath<(String, String)>,
    Rmp(deployment_req): Rmp<DeploymentRequest>,
) -> Result<Json<shuttle_common::models::deployment::Response>> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let span = info_span!(
        "Starting deployment",
        deployment_id = %id,
    );

    let service = persistence.get_or_create_service(&service_name).await?;
    let pid = persistence.project_id();

    span.in_scope(|| {
        info!("Deployer version: {}", crate::VERSION);
        info!("Deployment ID: {}", id);
        info!("Service ID: {}", service.id);
        info!("Service name: {}", service.name);
        info!("Project ID: {}", pid);
        info!("Project name: {}", project_name);
        info!("Date: {}", now.to_rfc3339_opts(SecondsFormat::Secs, true));
    });

    let deployment = Deployment {
        id,
        service_id: service.id,
        state: State::Queued,
        last_update: now,
        address: None,
        is_next: false,
        git_commit_id: deployment_req
            .git_commit_id
            .map(|s| s.chars().take(GIT_STRINGS_MAX_LENGTH).collect()),
        git_commit_msg: deployment_req
            .git_commit_msg
            .map(|s| s.chars().take(GIT_STRINGS_MAX_LENGTH).collect()),
        git_branch: deployment_req
            .git_branch
            .map(|s| s.chars().take(GIT_STRINGS_MAX_LENGTH).collect()),
        git_dirty: deployment_req.git_dirty,
    };

    persistence.insert_deployment(&deployment).await?;
    let queued = Queued {
        id: deployment.id,
        service_name: service.name,
        service_id: deployment.service_id,
        project_id: pid,
        data: deployment_req.data,
        will_run_tests: !deployment_req.no_test,
        tracing_context: Default::default(),
        claim,
    };

    deployment_manager.queue_push(queued).await;

    Ok(Json(deployment.into()))
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name, shuttle.service.name = %service_name))]
pub async fn stop_service(
    Extension(persistence): Extension<Persistence>,
    Extension(deployment_manager): Extension<DeploymentManager>,
    CustomErrorPath((project_name, service_name)): CustomErrorPath<(String, String)>,
) -> Result<Json<shuttle_common::models::service::Summary>> {
    let service = persistence
        .get_service_by_name(&service_name)
        .await?
        .ok_or_else(|| Error::NotFound("service not found".to_string()))?;

    let running_deployment = persistence.get_active_deployment(&service.id).await?;
    let Some(ref deployment) = running_deployment else {
        return Err(Error::NotFound("no running deployment found".to_string()));
    };
    deployment_manager.kill(deployment.id).await;

    let response = shuttle_common::models::service::Summary {
        name: service.name,
        deployment: running_deployment.map(Into::into),
        uri: "".to_owned(),
    };

    Ok(Json(response))
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name, page, limit))]
pub async fn get_deployments(
    Extension(persistence): Extension<Persistence>,
    CustomErrorPath(project_name): CustomErrorPath<String>,
    Query(PaginationDetails { page, limit }): Query<PaginationDetails>,
) -> Result<Json<Vec<shuttle_common::models::deployment::Response>>> {
    let Some(service) = persistence.get_service_by_name(&project_name).await? else {
        return Ok(Json(vec![]));
    };

    let limit = limit.unwrap_or(u32::MAX);
    let page = page.unwrap_or(0);
    let deployments = persistence
        .get_deployments(&service.id, page * limit, limit)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(deployments))
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name, %deployment_id))]
pub async fn get_deployment(
    Extension(persistence): Extension<Persistence>,
    CustomErrorPath((project_name, deployment_id)): CustomErrorPath<(String, Uuid)>,
) -> Result<Json<shuttle_common::models::deployment::Response>> {
    if let Some(deployment) = persistence.get_deployment(&deployment_id).await? {
        Ok(Json(deployment.into()))
    } else {
        Err(Error::NotFound("deployment not found".to_string()))
    }
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name, %deployment_id))]
pub async fn delete_deployment(
    Extension(deployment_manager): Extension<DeploymentManager>,
    Extension(persistence): Extension<Persistence>,
    CustomErrorPath((project_name, deployment_id)): CustomErrorPath<(String, Uuid)>,
) -> Result<Json<shuttle_common::models::deployment::Response>> {
    if let Some(deployment) = persistence.get_deployment(&deployment_id).await? {
        deployment_manager.kill(deployment.id).await;

        Ok(Json(deployment.into()))
    } else {
        Err(Error::NotFound("deployment not found".to_string()))
    }
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name, %deployment_id))]
pub async fn get_logs(
    Extension(deployment_manager): Extension<DeploymentManager>,
    Extension(claim): Extension<Claim>,
    CustomErrorPath((project_name, deployment_id)): CustomErrorPath<(String, Uuid)>,
    Query(LogsQuery { head, tail }): Query<LogsQuery>,
) -> Result<Json<Vec<LogItem>>> {
    let mut logs_request: tonic::Request<LogsRequest> = tonic::Request::new(LogsRequest {
        deployment_id: deployment_id.to_string(),
        head,
        tail,
    });

    logs_request.extensions_mut().insert(claim);

    let mut client = deployment_manager.logs_fetcher().clone();

    match client.get_logs(logs_request).await {
        Ok(logs) => Ok(Json(
            logs.into_inner()
                .log_items
                .into_iter()
                .map(|l| l.to_log_item_with_id(deployment_id))
                .collect(),
        )),
        Err(error) => {
            error!(
                error = &error as &dyn std::error::Error,
                "failed to retrieve logs for deployment"
            );
            match error.code() {
                Code::OutOfRange => {
                    Err(anyhow!("Too many log lines to be fetched in one request. You can redeploy your app to refresh the logs quota\n
                    or if you'd like more flexibility around logs fetching, please reach out in a help thread\n
                    on Shuttle's Discord server.").into())
                }
                _ => Err(anyhow!("failed to retrieve logs for deployment").into()),
            }
        }
    }
}

// don't instrument id to prevent it from showing up in deployment log
#[instrument(skip_all, fields(shuttle.project.name = %project_name))]
pub async fn get_logs_subscribe(
    Extension(deployment_manager): Extension<DeploymentManager>,
    Extension(claim): Extension<Claim>,
    CustomErrorPath((project_name, deployment_id)): CustomErrorPath<(String, Uuid)>,
    Query(LogsQuery { head, tail }): Query<LogsQuery>,
    ws_upgrade: ws::WebSocketUpgrade,
) -> axum::response::Response {
    ws_upgrade.on_upgrade(move |s| {
        logs_websocket_handler(s, deployment_manager, deployment_id, head, tail, claim)
    })
}

async fn logs_websocket_handler(
    mut s: WebSocket,
    deployment_manager: DeploymentManager,
    deployment_id: Uuid,
    head: Option<u32>,
    tail: Option<u32>,
    claim: Claim,
) {
    let mut logs_request: tonic::Request<LogsRequest> = tonic::Request::new(LogsRequest {
        deployment_id: deployment_id.to_string(),
        head,
        tail,
    });

    logs_request.extensions_mut().insert(claim);

    let mut client = deployment_manager.logs_fetcher().clone();
    let log_stream_response = client.get_logs_stream(logs_request).await;

    let mut stream = match log_stream_response {
        Ok(inner_response) => inner_response.into_inner(),
        Err(error) => {
            error!(
                error = &error as &dyn std::error::Error,
                "failed to get backlog of logs"
            );

            let _ = s
                .send(ws::Message::Text("failed to get logs".to_string()))
                .await;

            let _ = s.close().await;
            return;
        }
    };

    loop {
        match stream.message().await {
            Ok(None) => {
                trace!("The logs stream was closed gracefully.");
                let _ = s
                    .send(ws::Message::Text(
                        "the logs stream was closed gracefully.".to_string(),
                    ))
                    .await;
                break;
            }
            Ok(Some(proto_log)) => {
                let log = proto_log.to_log_item_with_id(deployment_id);
                trace!(?log, "received log from logger stream");
                if log.id == deployment_id {
                    let msg = serde_json::to_string(&log).expect("to convert log item to json");
                    let sent = s.send(ws::Message::Text(msg)).await;

                    // Client disconnected?
                    if sent.is_err() {
                        return;
                    }
                }
            }
            Err(error) => {
                trace!(?error, "the logs stream was closed by Shuttle");
                let _ = s
                    .send(ws::Message::Text(
                        "The logs stream was closed by Shuttle because of an internal error"
                            .to_string(),
                    ))
                    .await;
                break;
            }
        }
    }

    let _ = s.close().await;
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name))]
pub async fn clean_project(
    Extension(deployment_manager): Extension<DeploymentManager>,
    CustomErrorPath(project_name): CustomErrorPath<String>,
) -> Result<Json<String>> {
    clean_crate(
        deployment_manager
            .builds_path()
            .join(project_name)
            .as_path(),
    )
    .await?;

    Ok(Json("Cleaning done".into()))
}

pub struct Rmp<T>(T);

#[async_trait]
impl<S, B, T> FromRequest<S, B> for Rmp<T>
where
    S: Send + Sync,
    B: Send + 'static,
    Bytes: FromRequest<S, B>,
    T: DeserializeOwned,
{
    type Rejection = StatusCode;

    async fn from_request(
        req: Request<B>,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state).await.map_err(|_| {
            info!("failed to collect body bytes, is the body too large?");
            StatusCode::PAYLOAD_TOO_LARGE
        })?;

        let t = rmp_serde::from_slice::<T>(&bytes).map_err(|error| {
            error!(error = %error, "failed to deserialize request body");
            StatusCode::BAD_REQUEST
        })?;

        Ok(Self(t))
    }
}
