use std::io::Cursor;
use std::net::SocketAddr;
use std::ops::Sub;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Extension, Path, Query, State};
use axum::handler::Handler;
use axum::http::Request;
use axum::middleware::from_extractor;
use axum::response::Response;
use axum::routing::{any, delete, get, post};
use axum::{Json as AxumJson, Router};
use fqdn::FQDN;
use futures::Future;
use http::{StatusCode, Uri};
use instant_acme::{AccountCredentials, ChallengeType};
use serde::{Deserialize, Serialize};
use shuttle_common::backends::auth::{AuthPublicKey, JwtAuthenticationLayer, ScopedLayer};
use shuttle_common::backends::cache::CacheManager;
use shuttle_common::backends::metrics::{Metrics, TraceLayer};
use shuttle_common::claims::{Scope, EXP_MINUTES};
use shuttle_common::limits::ClaimExt;
use shuttle_common::models::error::axum::CustomErrorPath;
use shuttle_common::models::error::ErrorKind;
use shuttle_common::models::{
    admin::ProjectResponse,
    project::{self, ProjectName},
    stats,
};
use shuttle_common::{deployment, request_span, VersionInfo};
use shuttle_proto::provisioner::provisioner_client::ProvisionerClient;
use shuttle_proto::provisioner::Ping;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, MutexGuard};
use tower::ServiceBuilder;
use tracing::{field, instrument, trace};
use ttl_cache::TtlCache;
use ulid::Ulid;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::IntoParams;
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;
use x509_parser::nom::AsBytes;
use x509_parser::parse_x509_certificate;
use x509_parser::pem::parse_x509_pem;
use x509_parser::time::ASN1Time;

use crate::acme::{AcmeClient, CustomDomain};
use crate::auth::{ScopedUser, User};
use crate::project::{ContainerInspectResponseExt, Project, ProjectCreating};
use crate::service::GatewayService;
use crate::task::{self, BoxedTask, TaskResult};
use crate::tls::{GatewayCertResolver, RENEWAL_VALIDITY_THRESHOLD_IN_DAYS};
use crate::worker::WORKER_QUEUE_SIZE;
use crate::{Error, AUTH_CLIENT};

use super::auth_layer::ShuttleAuthLayer;
use super::project_caller::ProjectCaller;

pub const SVC_DEGRADED_THRESHOLD: usize = 128;
pub const SHUTTLE_GATEWAY_VARIANT: &str = "shuttle-gateway";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    status: ComponentStatus,
}

#[derive(Debug, Clone, Copy, Deserialize, IntoParams)]
pub struct PaginationDetails {
    /// Page to fetch, starting from 0.
    pub page: Option<u32>,
    /// Number of results per page.
    pub limit: Option<u32>,
}

impl StatusResponse {
    pub fn healthy() -> Self {
        Self {
            status: ComponentStatus::Healthy,
        }
    }

    pub fn degraded() -> Self {
        Self {
            status: ComponentStatus::Degraded,
        }
    }

    pub fn unhealthy() -> Self {
        Self {
            status: ComponentStatus::Unhealthy,
        }
    }
}

#[instrument(skip(service))]
#[utoipa::path(
    get,
    path = "/projects/{project_name}",
    responses(
        (status = 200, description = "Successfully got a specific project information.", body = shuttle_common::models::project::Response),
        (status = 500, description = "Server internal error.")
    ),
    params(
        ("project_name" = String, Path, description = "The name of the project."),
    )
)]
async fn get_project(
    State(RouterState { service, .. }): State<RouterState>,
    ScopedUser { scope, .. }: ScopedUser,
) -> Result<AxumJson<project::Response>, Error> {
    let project = service.find_project(&scope).await?;
    let idle_minutes = project.state.idle_minutes();

    let response = project::Response {
        id: project.project_id.to_uppercase(),
        name: scope.to_string(),
        state: project.state.into(),
        idle_minutes,
    };

    Ok(AxumJson(response))
}

#[instrument(skip(service))]
#[utoipa::path(
    get,
    path = "/projects/name/{project_name}",
    responses(
        (status = 200, description = "True if project name is taken. False if free.", body = bool),
        (status = 400, description = "Invalid project name.", body = String),
        (status = 500, description = "Server internal error.")
    ),
    params(
        ("project_name" = String, Path, description = "The project name to check."),
    )
)]
async fn check_project_name(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath(project_name): CustomErrorPath<ProjectName>,
) -> Result<AxumJson<bool>, Error> {
    service
        .project_name_exists(&project_name)
        .await
        .map(AxumJson)
}

#[utoipa::path(
    get,
    path = "/projects",
    responses(
        (status = 200, description = "Successfully got the projects list.", body = [shuttle_common::models::project::Response]),
        (status = 500, description = "Server internal error.")
    ),
    params(
        PaginationDetails
    )
)]
async fn get_projects_list(
    State(RouterState { service, .. }): State<RouterState>,
    User { name, .. }: User,
    Query(PaginationDetails { page, limit }): Query<PaginationDetails>,
) -> Result<AxumJson<Vec<project::Response>>, Error> {
    let limit = limit.unwrap_or(u32::MAX);
    let page = page.unwrap_or(0);
    let projects = service
        // The `offset` is page size * amount of pages
        .iter_user_projects_detailed(&name, limit * page, limit)
        .await?
        .map(|project| project::Response {
            id: project.0.to_uppercase(),
            name: project.1.to_string(),
            idle_minutes: project.2.idle_minutes(),
            state: project.2.into(),
        })
        .collect();

    Ok(AxumJson(projects))
}

#[instrument(skip_all, fields(%project_name))]
#[utoipa::path(
    post,
    path = "/projects/{project_name}",
    responses(
        (status = 200, description = "Successfully created a specific project.", body = shuttle_common::models::project::Response),
        (status = 500, description = "Server internal error.")
    ),
    params(
        ("project_name" = String, Path, description = "The name of the project."),
    )
)]
async fn create_project(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
    User { name, claim, .. }: User,
    CustomErrorPath(project_name): CustomErrorPath<ProjectName>,
    AxumJson(config): AxumJson<project::Config>,
) -> Result<AxumJson<project::Response>, Error> {
    let is_cch_project = project_name.is_cch_project();

    // Check that the user is within their project limits.
    let can_create_project = claim.can_create_project(
        service
            .get_project_count(&name)
            .await?
            .saturating_sub(is_cch_project as u32),
    );

    if is_cch_project {
        let current_container_count = service.count_ready_projects().await?;

        if current_container_count >= service.container_limit() {
            return Err(Error::from_kind(ErrorKind::ContainerLimit));
        }
    }

    let project = service
        .create_project(
            project_name.clone(),
            name.clone(),
            claim.is_admin(),
            can_create_project,
            if is_cch_project {
                5
            } else {
                config.idle_minutes
            },
        )
        .await?;
    let idle_minutes = project.state.idle_minutes();

    service
        .new_task()
        .project(project_name.clone())
        .and_then(task::run_until_done())
        .and_then(task::start_idle_deploys())
        .send(&sender)
        .await?;

    let response = project::Response {
        id: project.project_id.to_string().to_uppercase(),
        name: project_name.to_string(),
        state: project.state.into(),
        idle_minutes,
    };

    Ok(AxumJson(response))
}

#[instrument(skip_all, fields(%project_name))]
#[utoipa::path(
    delete,
    path = "/projects/{project_name}",
    responses(
        (status = 200, description = "Successfully destroyed a specific project.", body = shuttle_common::models::project::Response),
        (status = 500, description = "Server internal error.")
    ),
    params(
        ("project_name" = String, Path, description = "The name of the project."),
    )
)]
async fn destroy_project(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
    ScopedUser {
        scope: project_name,
        ..
    }: ScopedUser,
) -> Result<AxumJson<project::Response>, Error> {
    let project = service.find_project(&project_name).await?;
    let idle_minutes = project.state.idle_minutes();

    let mut response = project::Response {
        id: project.project_id.to_uppercase(),
        name: project_name.to_string(),
        state: project.state.into(),
        idle_minutes,
    };

    if response.state == shuttle_common::models::project::State::Destroyed {
        return Ok(AxumJson(response));
    }

    // if project exists and isn't `Destroyed`, send destroy task
    service
        .new_task()
        .project(project_name)
        .and_then(task::destroy())
        .send(&sender)
        .await?;

    response.state = shuttle_common::models::project::State::Destroying;

    Ok(AxumJson(response))
}

#[derive(Deserialize, IntoParams)]
struct DeleteProjectParams {
    // Was added in v0.30.0
    // We have not needed it since 0.35.0, but have to keep in for any old CLI users
    #[allow(dead_code)]
    dry_run: Option<bool>,
}

#[instrument(skip_all, fields(project_name = %scoped_user.scope))]
#[utoipa::path(
    delete,
    path = "/projects/{project_name}/delete",
    responses(
        (status = 200, description = "Successfully deleted a project, unless dry run.", body = shuttle_common::models::project::Response),
        (status = 403, description = "Project cannot be deleted now."),
        (status = 500, description = "Server internal error."),
    ),
    params(
        ("project_name" = String, Path, description = "The name of the project."),
    )
)]
async fn delete_project(
    State(state): State<RouterState>,
    scoped_user: ScopedUser,
    Query(DeleteProjectParams { dry_run }): Query<DeleteProjectParams>,
    req: Request<Body>,
) -> Result<AxumJson<String>, Error> {
    // Don't do the dry run that might come from older CLIs
    if dry_run.is_some_and(|d| d) {
        return Ok(AxumJson("dry run is no longer supported".to_owned()));
    }

    let project_name = scoped_user.scope.clone();
    let project = state.service.find_project(&project_name).await?;
    let project_id =
        Ulid::from_string(&project.project_id).expect("stored project id to be a valid ULID");

    // Try to startup destroyed or errored projects
    let project_deletable = project.state.is_ready() || project.state.is_stopped();
    if !(project_deletable) {
        let handle = state
            .service
            .new_task()
            .project(project_name.clone())
            .and_then(task::restart(project_id))
            .send(&state.sender)
            .await?;

        // Wait for the project to be ready
        handle.await;

        let new_state = state.service.find_project(&project_name).await?;

        if !new_state.state.is_ready() {
            return Err(Error::from_kind(ErrorKind::ProjectCorrupted));
        }
    }

    let service = state.service.clone();
    let sender = state.sender.clone();

    let project_caller =
        ProjectCaller::new(state.clone(), scoped_user.clone(), req.headers()).await?;

    // check that a deployment is not running
    let mut deployments = project_caller.get_deployment_list().await?;
    deployments.sort_by_key(|d| d.last_update);

    // Make sure no deployment is in the building pipeline
    let has_bad_state = deployments.iter().any(|d| {
        !matches!(
            d.state,
            deployment::State::Running
                | deployment::State::Completed
                | deployment::State::Crashed
                | deployment::State::Stopped
        )
    });

    if has_bad_state {
        return Err(Error::from_kind(ErrorKind::ProjectHasBuildingDeployment));
    }

    let running_deployments = deployments
        .into_iter()
        .filter(|d| d.state == deployment::State::Running);

    for running_deployment in running_deployments {
        let res = project_caller
            .stop_deployment(&running_deployment.id)
            .await?;

        if res.status() != StatusCode::OK {
            return Err(Error::from_kind(ErrorKind::ProjectHasRunningDeployment));
        }
    }

    // check if any resources exist
    let resources = project_caller.get_resources().await?;
    let mut delete_fails = Vec::new();

    for resource in resources {
        let resource_type = resource.r#type.to_string();
        let res = project_caller.delete_resource(&resource_type).await?;

        if res.status() != StatusCode::OK {
            delete_fails.push(resource_type)
        }
    }

    if !delete_fails.is_empty() {
        return Err(Error::from_kind(ErrorKind::ProjectHasResources(
            delete_fails,
        )));
    }

    let task = service
        .new_task()
        .project(project_name.clone())
        .and_then(task::delete_project())
        .send(&sender)
        .await?;
    task.await;

    service.delete_project(&project_name).await?;

    Ok(AxumJson("project successfully deleted".to_owned()))
}

#[instrument(skip_all, fields(scope = %scoped_user.scope))]
async fn route_project(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
    scoped_user: ScopedUser,
    req: Request<Body>,
) -> Result<Response<Body>, Error> {
    let project_name = scoped_user.scope;
    let is_cch_project = project_name.is_cch_project();

    // Don't start cch projects if we will be going over the container limit
    if is_cch_project {
        let current_container_count = service.count_ready_projects().await?;

        if current_container_count >= service.container_limit() {
            return Err(Error::from_kind(ErrorKind::ContainerLimit));
        }
    }

    let project = service.find_or_start_project(&project_name, sender).await?;
    service
        .route(&project.state, &project_name, &scoped_user.user.name, req)
        .await
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Get the gateway operational status."),
        (status = 500, description = "Server internal error.")
    )
)]
async fn get_status(
    State(RouterState {
        sender, service, ..
    }): State<RouterState>,
) -> Response<Body> {
    let mut statuses = Vec::new();
    // Compute gateway status.
    if sender.is_closed() || sender.capacity() == 0 {
        statuses.push((SHUTTLE_GATEWAY_VARIANT, StatusResponse::unhealthy()));
    } else if sender.capacity() < WORKER_QUEUE_SIZE - SVC_DEGRADED_THRESHOLD {
        statuses.push((SHUTTLE_GATEWAY_VARIANT, StatusResponse::degraded()));
    } else {
        statuses.push((SHUTTLE_GATEWAY_VARIANT, StatusResponse::healthy()));
    };

    // Compute provisioner status.
    let provisioner_status = if let Ok(channel) = service.provisioner_host().connect().await {
        let channel = ServiceBuilder::new().service(channel);
        let mut provisioner_client = ProvisionerClient::new(channel);
        if provisioner_client.health_check(Ping {}).await.is_ok() {
            StatusResponse::healthy()
        } else {
            StatusResponse::unhealthy()
        }
    } else {
        StatusResponse::unhealthy()
    };

    statuses.push(("shuttle-provisioner", provisioner_status));

    // Compute auth status.
    let auth_status = {
        let response = AUTH_CLIENT.get(service.auth_uri().clone()).await;
        match response {
            Ok(response) if response.status() == 200 => StatusResponse::healthy(),
            Ok(_) | Err(_) => StatusResponse::unhealthy(),
        }
    };

    statuses.push(("shuttle-auth", auth_status));

    let body = serde_json::to_vec(&statuses).expect("could not make a json out of the statuses");
    Response::builder()
        .body(body.into())
        .expect("could not make a response with the status check response")
}

#[instrument(skip_all)]
#[utoipa::path(
    post,
    path = "/stats/load",
    responses(
        (status = 200, description = "Successfully fetched the build queue load.", body = shuttle_common::models::stats::LoadResponse),
        (status = 500, description = "Server internal error.")
    )
)]
async fn post_load(
    State(RouterState { running_builds, .. }): State<RouterState>,
    AxumJson(build): AxumJson<stats::LoadRequest>,
) -> Result<AxumJson<stats::LoadResponse>, Error> {
    let mut running_builds = running_builds.lock().await;

    trace!(id = %build.id, "checking build queue");
    let mut load = calculate_capacity(&mut running_builds);

    if load.has_capacity
        && running_builds
            .insert(build.id, (), Duration::from_secs(60 * EXP_MINUTES as u64))
            .is_none()
    {
        // Only increase when an item was not already in the queue
        load.builds_count += 1;
    }

    Ok(AxumJson(load))
}

#[instrument(skip_all)]
#[utoipa::path(
    delete,
    path = "/stats/load",
    responses(
        (status = 200, description = "Successfully removed the build with the ID specified in the load request from the build queue.", body = shuttle_common::models::stats::LoadResponse),
        (status = 500, description = "Server internal error.")
    )
)]
async fn delete_load(
    State(RouterState { running_builds, .. }): State<RouterState>,
    AxumJson(build): AxumJson<stats::LoadRequest>,
) -> Result<AxumJson<stats::LoadResponse>, Error> {
    let mut running_builds = running_builds.lock().await;
    running_builds.remove(&build.id);

    trace!(id = %build.id, "removing from build queue");
    let load = calculate_capacity(&mut running_builds);

    Ok(AxumJson(load))
}

#[instrument(skip_all)]
#[utoipa::path(
    get,
    path = "/admin/stats/load",
    responses(
        (status = 200, description = "Successfully gets the build queue load as an admin.", body = shuttle_common::models::stats::LoadResponse),
        (status = 500, description = "Server internal error.")
    )
)]
async fn get_load_admin(
    State(RouterState { running_builds, .. }): State<RouterState>,
) -> Result<AxumJson<stats::LoadResponse>, Error> {
    let mut running_builds = running_builds.lock().await;

    let load = calculate_capacity(&mut running_builds);

    Ok(AxumJson(load))
}

#[instrument(skip_all)]
#[utoipa::path(
    delete,
    path = "/admin/stats/load",
    responses(
        (status = 200, description = "Successfully clears the build queue.", body = shuttle_common::models::stats::LoadResponse),
        (status = 500, description = "Server internal error.")
    )
)]
async fn delete_load_admin(
    State(RouterState { running_builds, .. }): State<RouterState>,
) -> Result<AxumJson<stats::LoadResponse>, Error> {
    let mut running_builds = running_builds.lock().await;
    running_builds.clear();

    let load = calculate_capacity(&mut running_builds);

    Ok(AxumJson(load))
}

fn calculate_capacity(running_builds: &mut MutexGuard<TtlCache<Uuid, ()>>) -> stats::LoadResponse {
    let active = running_builds.iter().count();
    let capacity = running_builds.capacity();
    let has_capacity = active < capacity;

    stats::LoadResponse {
        builds_count: active,
        has_capacity,
    }
}

#[instrument(skip_all)]
#[utoipa::path(
    post,
    path = "/admin/revive",
    responses(
        (status = 200, description = "Successfully revived stopped or errored projects."),
        (status = 500, description = "Server internal error.")
    )
)]
async fn revive_projects(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
) -> Result<(), Error> {
    crate::project::exec::revive(service, sender)
        .await
        .map_err(|_| Error::from_kind(ErrorKind::Internal))
}

#[instrument(skip_all)]
#[utoipa::path(
    post,
    path = "/admin/destroy",
    responses(
        (status = 200, description = "Successfully destroyed the projects."),
        (status = 500, description = "Server internal error.")
    )
)]
async fn destroy_projects(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
) -> Result<(), Error> {
    crate::project::exec::destroy(service, sender)
        .await
        .map_err(|_| Error::from_kind(ErrorKind::Internal))
}

#[instrument(skip_all, fields(%email, ?acme_server))]
#[utoipa::path(
    post,
    path = "/admin/acme/{email}",
    responses(
        (status = 200, description = "Created an acme account.", content_type = "application/json", body = String),
        (status = 500, description = "Server internal error.")
    ),
    params(
        ("email" = String, Path, description = "An email the acme account binds to."),
    ),

)]
async fn create_acme_account(
    Extension(acme_client): Extension<AcmeClient>,
    Path(email): Path<String>,
    AxumJson(acme_server): AxumJson<Option<String>>,
) -> Result<AxumJson<serde_json::Value>, Error> {
    let res = acme_client.create_account(&email, acme_server).await?;

    Ok(AxumJson(res))
}

#[instrument(skip_all, fields(%project_name, %fqdn))]
#[utoipa::path(
    post,
    path = "/admin/acme/request/{project_name}/{fqdn}",
    responses(
        (status = 200, description = "Successfully requested a custom domain for the the project."),
        (status = 500, description = "Server internal error.")
    ),
    params(
        ("project_name" = String, Path, description = "The project name associated to the requested custom domain."),
        ("fqdn" = String, Path, description = "The fqdn that represents the requested custom domain."),
    )
)]
async fn request_custom_domain_acme_certificate(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
    Extension(acme_client): Extension<AcmeClient>,
    Extension(resolver): Extension<Arc<GatewayCertResolver>>,
    CustomErrorPath((project_name, fqdn)): CustomErrorPath<(ProjectName, String)>,
    AxumJson(credentials): AxumJson<AccountCredentials<'_>>,
) -> Result<String, Error> {
    let fqdn: FQDN = fqdn
        .parse()
        .map_err(|_err| Error::from(ErrorKind::InvalidCustomDomain))?;

    let (certs, private_key) = service
        .create_custom_domain_certificate(&fqdn, &acme_client, &project_name, credentials)
        .await?;

    let project = service.find_project(&project_name).await?;
    let project_id = project
        .state
        .container()
        .unwrap()
        .project_id()
        .map_err(|_| Error::custom(ErrorKind::Internal, "Missing project_id from the container"))?;

    let container = project.state.container().unwrap();
    let idle_minutes = container.idle_minutes();

    // Destroy and recreate the project with the new domain.
    service
        .new_task()
        .project(project_name.clone())
        .and_then(task::destroy())
        .and_then(task::run_until_done())
        .and_then(task::run({
            let fqdn = fqdn.to_string();
            move |ctx| {
                let fqdn = fqdn.clone();
                async move {
                    let creating = ProjectCreating::new_with_random_initial_key(
                        ctx.project_name,
                        project_id,
                        idle_minutes,
                    )
                    .with_fqdn(fqdn);
                    TaskResult::Done(Project::Creating(creating))
                }
            }
        }))
        .and_then(task::run_until_done())
        .and_then(task::start_idle_deploys())
        .send(&sender)
        .await?;

    let mut buf = Vec::new();
    buf.extend(certs.as_bytes());
    buf.extend(private_key.as_bytes());
    resolver
        .serve_pem(&fqdn.to_string(), Cursor::new(buf))
        .await?;
    Ok(format!(
        r#""New certificate created for {} project.""#,
        project_name
    ))
}

#[instrument(skip_all, fields(%project_name, %fqdn))]
#[utoipa::path(
    post,
    path = "/admin/acme/renew/{project_name}/{fqdn}",
    responses(
        (status = 200, description = "Successfully renewed the project TLS certificate for the appointed custom domain fqdn."),
        (status = 500, description = "Server internal error.")
    ),
    params(
        ("project_name" = String, Path, description = "The project name associated to the requested custom domain."),
        ("fqdn" = String, Path, description = "The fqdn that represents the requested custom domain."),
    )
)]
async fn renew_custom_domain_acme_certificate(
    State(RouterState { service, .. }): State<RouterState>,
    Extension(acme_client): Extension<AcmeClient>,
    Extension(resolver): Extension<Arc<GatewayCertResolver>>,
    CustomErrorPath((project_name, fqdn)): CustomErrorPath<(ProjectName, String)>,
    AxumJson(credentials): AxumJson<AccountCredentials<'_>>,
) -> Result<String, Error> {
    let fqdn: FQDN = fqdn
        .parse()
        .map_err(|_err| Error::from(ErrorKind::InvalidCustomDomain))?;
    // Try retrieve the current certificate if any.
    match service.project_details_for_custom_domain(&fqdn).await {
        Ok(CustomDomain {
            mut certificate,
            private_key,
            ..
        }) => {
            certificate.push('\n');
            certificate.push('\n');
            certificate.push_str(private_key.as_str());
            let (_, pem) = parse_x509_pem(certificate.as_bytes()).map_err(|err| {
                Error::custom(
                    ErrorKind::Internal,
                    format!("Error while parsing the pem certificate for {project_name}: {err}"),
                )
            })?;

            let (_, x509_cert_chain) =
                parse_x509_certificate(pem.contents.as_bytes()).map_err(|err| {
                    Error::custom(
                        ErrorKind::Internal,
                        format!(
                            "Error while parsing the certificate chain for {project_name}: {err}"
                        ),
                    )
                })?;

            let diff = x509_cert_chain
                .validity()
                .not_after
                .sub(ASN1Time::now())
                .unwrap_or_default();

            // Renew only when the difference is `None` (meaning certificate expired) or we're within the last 30 days of validity.
            if diff.whole_days() <= RENEWAL_VALIDITY_THRESHOLD_IN_DAYS {
                return match acme_client
                    .create_certificate(&fqdn.to_string(), ChallengeType::Http01, credentials)
                    .await
                {
                    // If successfully created, save the certificate in memory to be
                    // served in the future.
                    Ok((certs, private_key)) => {
                        service
                            .create_custom_domain(&project_name, &fqdn, &certs, &private_key)
                            .await?;

                        let mut buf = Vec::new();
                        buf.extend(certs.as_bytes());
                        buf.extend(private_key.as_bytes());
                        resolver
                            .serve_pem(&fqdn.to_string(), Cursor::new(buf))
                            .await?;
                        Ok(format!(
                            r#""Certificate renewed for {} project.""#,
                            project_name
                        ))
                    }
                    Err(err) => Err(err.into()),
                };
            } else {
                Ok(format!(
                    r#""Certificate renewal skipped, {} project certificate still valid for {} days.""#,
                    project_name, diff
                ))
            }
        }
        Err(err) => Err(err),
    }
}

#[instrument(skip_all)]
#[utoipa::path(
    post,
    path = "/admin/acme/gateway/renew",
    responses(
        (status = 200, description = "Successfully renewed the gateway TLS certificate."),
        (status = 500, description = "Server internal error.")
    )
)]
async fn renew_gateway_acme_certificate(
    State(RouterState { service, .. }): State<RouterState>,
    Extension(acme_client): Extension<AcmeClient>,
    Extension(resolver): Extension<Arc<GatewayCertResolver>>,
    AxumJson(credentials): AxumJson<AccountCredentials<'_>>,
) -> Result<String, Error> {
    service
        .renew_certificate(&acme_client, resolver, credentials)
        .await;
    Ok(r#""Renewed the gateway certificate.""#.to_string())
}

#[utoipa::path(
    post,
    path = "/admin/projects",
    responses(
        (status = 200, description = "Successfully fetched the projects list.", body = shuttle_common::models::project::AdminResponse),
        (status = 500, description = "Server internal error.")
    )
)]
async fn get_projects(
    State(RouterState { service, .. }): State<RouterState>,
) -> Result<AxumJson<Vec<ProjectResponse>>, Error> {
    let projects = service
        .iter_projects_detailed()
        .await?
        .map(Into::into)
        .collect();

    Ok(AxumJson(projects))
}

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "Gateway API Key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Bearer"))),
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_acme_account,
        request_custom_domain_acme_certificate,
        renew_custom_domain_acme_certificate,
        renew_gateway_acme_certificate,
        get_status,
        get_projects_list,
        get_project,
        destroy_project,
        create_project,
        post_load,
        delete_load,
        get_projects,
        revive_projects,
        destroy_projects,
        get_load_admin,
        delete_load_admin
    ),
    modifiers(&SecurityAddon),
    components(schemas(
        shuttle_common::models::project::Response,
        shuttle_common::models::stats::LoadResponse,
        shuttle_common::models::admin::ProjectResponse,
        shuttle_common::models::stats::LoadResponse,
        shuttle_common::models::project::State
    ))
)]
pub struct ApiDoc;

#[derive(Clone)]
pub(crate) struct RouterState {
    pub service: Arc<GatewayService>,
    pub sender: Sender<BoxedTask>,
    pub running_builds: Arc<Mutex<TtlCache<Uuid, ()>>>,
}

pub struct ApiBuilder {
    router: Router<RouterState>,
    service: Option<Arc<GatewayService>>,
    sender: Option<Sender<BoxedTask>>,
    bind: Option<SocketAddr>,
}

impl Default for ApiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiBuilder {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            service: None,
            sender: None,
            bind: None,
        }
    }

    pub fn with_acme(mut self, acme: AcmeClient, resolver: Arc<GatewayCertResolver>) -> Self {
        self.router = self
            .router
            .route(
                "/admin/acme/:email",
                post(create_acme_account.layer(ScopedLayer::new(vec![Scope::AcmeCreate]))),
            )
            .route(
                "/admin/acme/request/:project_name/:fqdn",
                post(
                    request_custom_domain_acme_certificate
                        .layer(ScopedLayer::new(vec![Scope::CustomDomainCreate])),
                ),
            )
            .route(
                "/admin/acme/renew/:project_name/:fqdn",
                post(
                    renew_custom_domain_acme_certificate
                        .layer(ScopedLayer::new(vec![Scope::CustomDomainCertificateRenew])),
                ),
            )
            .route(
                "/admin/acme/gateway/renew",
                post(
                    renew_gateway_acme_certificate
                        .layer(ScopedLayer::new(vec![Scope::GatewayCertificateRenew])),
                ),
            )
            .layer(Extension(acme))
            .layer(Extension(resolver));
        self
    }

    pub fn with_service(mut self, service: Arc<GatewayService>) -> Self {
        self.service = Some(service);
        self
    }

    pub fn with_sender(mut self, sender: Sender<BoxedTask>) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn binding_to(mut self, addr: SocketAddr) -> Self {
        self.bind = Some(addr);
        self
    }

    pub fn with_default_traces(mut self) -> Self {
        self.router = self.router.route_layer(from_extractor::<Metrics>()).layer(
            TraceLayer::new(|request| {
                request_span!(
                    request,
                    account.name = field::Empty,
                    request.params.project_name = field::Empty,
                    request.params.account_name = field::Empty
                )
            })
            .with_propagation()
            .build(),
        );
        self
    }

    pub fn with_default_routes(mut self) -> Self {
        let admin_routes = Router::new()
            .route("/projects", get(get_projects))
            .route("/revive", post(revive_projects))
            .route("/destroy", post(destroy_projects))
            .route("/stats/load", get(get_load_admin).delete(delete_load_admin))
            // TODO: The `/swagger-ui` responds with a 303 See Other response which is followed in
            // browsers but leads to 404 Not Found. This must be investigated.
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .layer(ScopedLayer::new(vec![Scope::Admin]));

        const CARGO_SHUTTLE_VERSION: &str = env!("CARGO_PKG_VERSION");

        self.router = self
            .router
            .route("/", get(get_status))
            .route(
                "/versions",
                get(|| async {
                    axum::Json(VersionInfo {
                        gateway: env!("CARGO_PKG_VERSION").parse().unwrap(),
                        // For now, these use the same version as gateway (we release versions in lockstep).
                        // Only one version is officially compatible, but more are in reality.
                        cargo_shuttle: env!("CARGO_PKG_VERSION").parse().unwrap(),
                        deployer: env!("CARGO_PKG_VERSION").parse().unwrap(),
                        runtime: CARGO_SHUTTLE_VERSION.parse().unwrap(),
                    })
                }),
            )
            .route(
                "/version/cargo-shuttle",
                get(|| async { CARGO_SHUTTLE_VERSION }),
            )
            .route(
                "/projects",
                get(get_projects_list.layer(ScopedLayer::new(vec![Scope::Project]))),
            )
            .route(
                "/projects/:project_name",
                get(get_project.layer(ScopedLayer::new(vec![Scope::Project])))
                    .delete(destroy_project.layer(ScopedLayer::new(vec![Scope::ProjectWrite])))
                    .post(create_project.layer(ScopedLayer::new(vec![Scope::ProjectWrite]))),
            )
            .route(
                "/projects/:project_name/delete",
                delete(delete_project.layer(ScopedLayer::new(vec![Scope::ProjectWrite]))),
            )
            .route("/projects/name/:project_name", get(check_project_name))
            .route("/projects/:project_name/*any", any(route_project))
            .route("/stats/load", post(post_load).delete(delete_load))
            .nest("/admin", admin_routes);

        self
    }

    pub fn with_auth_service(mut self, auth_uri: Uri) -> Self {
        let auth_public_key = AuthPublicKey::new(auth_uri.clone());

        let jwt_cache_manager = CacheManager::new(1000);

        self.router = self
            .router
            .layer(JwtAuthenticationLayer::new(auth_public_key))
            .layer(ShuttleAuthLayer::new(
                auth_uri,
                Arc::new(Box::new(jwt_cache_manager)),
            ));

        self
    }

    pub fn into_router(self) -> Router {
        let service = self.service.expect("a GatewayService is required");
        let sender = self.sender.expect("a task Sender is required");

        // Allow about 4 cores per build, but use at most 75% (* 3 / 4) of all cores and at least 1 core
        let concurrent_builds: usize = (num_cpus::get() * 3 / 4 / 4).max(1);

        let running_builds = Arc::new(Mutex::new(TtlCache::new(concurrent_builds)));

        self.router.with_state(RouterState {
            service,
            sender,
            running_builds,
        })
    }

    pub fn serve(self) -> impl Future<Output = Result<(), hyper::Error>> {
        let bind = self.bind.expect("a socket address to bind to is required");
        let router = self.into_router();
        axum::Server::bind(&bind).serve(router.into_make_service())
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::headers::Authorization;
    use axum::http::Request;
    use futures::TryFutureExt;
    use http::Method;
    use hyper::body::to_bytes;
    use hyper::StatusCode;
    use serde_json::Value;
    use shuttle_common::constants::limits::{MAX_PROJECTS_DEFAULT, MAX_PROJECTS_EXTRA};
    use test_context::test_context;
    use tokio::sync::mpsc::channel;
    use tokio::sync::oneshot;
    use tokio::time::sleep;
    use tower::Service;

    use super::*;
    use crate::project::ProjectError;
    use crate::service::GatewayService;
    use crate::tests::{RequestBuilderExt, TestGateway, TestProject, World};

    #[tokio::test]
    async fn api_create_get_delete_projects() -> anyhow::Result<()> {
        let world = World::new().await;
        let service = Arc::new(GatewayService::init(world.args(), world.pool(), "".into()).await);

        let (sender, mut receiver) = channel::<BoxedTask>(256);
        tokio::spawn(async move {
            while receiver.recv().await.is_some() {
                // do not do any work with inbound requests
            }
        });

        let mut router = ApiBuilder::new()
            .with_service(Arc::clone(&service))
            .with_sender(sender)
            .with_default_routes()
            .with_auth_service(world.context().auth_uri)
            .into_router();

        let neo_key = world.create_user("neo");

        let create_project = |project: &str| {
            Request::builder()
                .method("POST")
                .uri(format!("/projects/{project}"))
                .header("Content-Type", "application/json")
                .body("{\"idle_minutes\": 3}".into())
                .unwrap()
        };

        let stop_project = |project: &str| {
            Request::builder()
                .method("DELETE")
                .uri(format!("/projects/{project}"))
                .body(Body::empty())
                .unwrap()
        };

        router
            .call(create_project("matrix"))
            .map_ok(|resp| assert_eq!(resp.status(), StatusCode::UNAUTHORIZED))
            .await
            .unwrap();

        let authorization = Authorization::bearer(&neo_key).unwrap();

        router
            .call(create_project("matrix").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        router
            .call(create_project("matrix").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            })
            .await
            .unwrap();

        let get_project = |project| {
            Request::builder()
                .method("GET")
                .uri(format!("/projects/{project}"))
                .body(Body::empty())
                .unwrap()
        };

        router
            .call(get_project("matrix"))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            })
            .await
            .unwrap();

        router
            .call(get_project("matrix").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        router
            .call(stop_project("matrix").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        router
            .call(create_project("reloaded").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        let trinity_key = world.create_user("trinity");

        let authorization = Authorization::bearer(&trinity_key).unwrap();

        router
            .call(get_project("reloaded").with_header(&authorization))
            .map_ok(|resp| assert_eq!(resp.status(), StatusCode::NOT_FOUND))
            .await
            .unwrap();

        router
            .call(stop_project("reloaded").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            })
            .await
            .unwrap();

        let get_load = || {
            Request::builder()
                .method("GET")
                .uri("/admin/stats/load")
                .body(Body::empty())
                .unwrap()
        };

        // Non-admin user cannot access admin routes
        router
            .call(get_load().with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::FORBIDDEN);
            })
            .await
            .unwrap();

        // Create new admin user
        let admin_neo_key = world.create_user("admin-neo");
        world.set_super_user("admin-neo");

        let authorization = Authorization::bearer(&admin_neo_key).unwrap();

        // Admin user can access admin routes
        router
            .call(get_load().with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        // TODO: setting the user to admin here doesn't update the cached token, so the
        // commands will still fail. We need to add functionality for this or modify the test.
        // world.set_super_user("trinity");

        // router
        //     .call(get_project("reloaded").with_header(&authorization))
        //     .map_ok(|resp| assert_eq!(resp.status(), StatusCode::OK))
        //     .await
        //     .unwrap();

        // router
        //     .call(delete_project("reloaded").with_header(&authorization))
        //     .map_ok(|resp| {
        //         assert_eq!(resp.status(), StatusCode::OK);
        //     })
        //     .await
        //     .unwrap();

        // // delete returns 404 for project that doesn't exist
        // router
        //     .call(delete_project("resurrections").with_header(&authorization))
        //     .map_ok(|resp| {
        //         assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        //     })
        //     .await
        //     .unwrap();

        Ok(())
    }

    #[tokio::test]
    async fn api_create_project_limits() -> anyhow::Result<()> {
        let world = World::new().await;
        let service = Arc::new(GatewayService::init(world.args(), world.pool(), "".into()).await);

        let (sender, mut receiver) = channel::<BoxedTask>(256);
        tokio::spawn(async move {
            while receiver.recv().await.is_some() {
                // do not do any work with inbound requests
            }
        });

        let mut router = ApiBuilder::new()
            .with_service(Arc::clone(&service))
            .with_sender(sender)
            .with_default_routes()
            .with_auth_service(world.context().auth_uri)
            .into_router();

        let neo_key = world.create_user("neo");

        let create_project = |project: &str| {
            Request::builder()
                .method("POST")
                .uri(format!("/projects/{project}"))
                .header("Content-Type", "application/json")
                .body("{\"idle_minutes\": 3}".into())
                .unwrap()
        };

        let authorization = Authorization::bearer(&neo_key).unwrap();

        // Creating three projects for a basic user succeeds.
        for i in 0..MAX_PROJECTS_DEFAULT {
            router
                .call(create_project(format!("matrix-{i}").as_str()).with_header(&authorization))
                .map_ok(|resp| {
                    assert_eq!(resp.status(), StatusCode::OK);
                })
                .await
                .unwrap();
        }

        // Creating one more project hits the project limit.
        router
            .call(create_project("resurrections").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::FORBIDDEN);
            })
            .await
            .unwrap();

        // Create a new admin user. We can't simply make the previous user an admin, since their token
        // will live in the auth cache without the admin scope.
        let trinity_key = world.create_user("trinity");
        world.set_super_user("trinity");
        let authorization = Authorization::bearer(&trinity_key).unwrap();

        // Creating more than the basic and pro limit of projects for an admin user succeeds.
        for i in 0..MAX_PROJECTS_EXTRA + 1 {
            router
                .call(create_project(format!("reloaded-{i}").as_str()).with_header(&authorization))
                .map_ok(|resp| {
                    assert_eq!(resp.status(), StatusCode::OK);
                })
                .await
                .unwrap();
        }

        Ok(())
    }

    #[test_context(TestGateway)]
    #[tokio::test]
    async fn api_create_project_above_container_limit(gateway: &mut TestGateway) {
        let _ = gateway.create_project("matrix").await;
        let cch_code = gateway.try_create_project("cch23-project").await;

        assert_eq!(cch_code, StatusCode::SERVICE_UNAVAILABLE);

        let normal_code = gateway.try_create_project("project").await;

        assert_eq!(
            normal_code,
            StatusCode::OK,
            "it should be possible to still create normal projects"
        );
    }

    #[test_context(TestGateway)]
    #[tokio::test]
    async fn start_idle_project_when_above_container_limit(gateway: &mut TestGateway) {
        let mut cch_idle_project = gateway.create_project("cch23-project").await;

        // Run four health checks to get the project to go into idle mode (cch projects always default to 5 min of idle time)
        cch_idle_project.run_health_check().await;
        cch_idle_project.run_health_check().await;
        cch_idle_project.run_health_check().await;
        cch_idle_project.run_health_check().await;

        cch_idle_project
            .wait_for_state(project::State::Stopped)
            .await;

        let mut normal_idle_project = gateway.create_project("project").await;

        // Run two health checks to get the project to go into idle mode
        normal_idle_project.run_health_check().await;
        normal_idle_project.run_health_check().await;
        normal_idle_project.run_health_check().await;
        normal_idle_project.run_health_check().await;

        normal_idle_project
            .wait_for_state(project::State::Stopped)
            .await;

        let _project_two = gateway.create_project("matrix").await;

        // Now try to start the idle projects
        let cch_code = cch_idle_project
            .router_call(Method::GET, "/services/cch23-project")
            .await;

        assert_eq!(cch_code, StatusCode::SERVICE_UNAVAILABLE);

        let normal_code = normal_idle_project
            .router_call(Method::GET, "/services/project")
            .await;

        assert_eq!(
            normal_code,
            StatusCode::NOT_FOUND,
            "should not be able to find a service since nothing was deployed"
        );
    }

    #[test_context(TestProject)]
    #[tokio::test]
    async fn api_delete_project_that_is_ready(project: &mut TestProject) {
        assert_eq!(
            project.router_call(Method::DELETE, "/delete").await,
            StatusCode::OK
        );
    }

    #[test_context(TestProject)]
    #[tokio::test]
    async fn api_delete_project_that_is_stopped(project: &mut TestProject) {
        // Run two health checks to get the project to go into idle mode
        project.run_health_check().await;
        project.run_health_check().await;

        project.wait_for_state(project::State::Stopped).await;

        assert_eq!(
            project.router_call(Method::DELETE, "/delete").await,
            StatusCode::OK
        );
    }

    #[test_context(TestProject)]
    #[tokio::test]
    async fn api_delete_project_that_is_destroyed(project: &mut TestProject) {
        project.destroy_project().await;

        assert_eq!(
            project.router_call(Method::DELETE, "/delete").await,
            StatusCode::OK
        );
    }

    #[test_context(TestProject)]
    #[tokio::test]
    async fn api_delete_project_that_has_resources(project: &mut TestProject) {
        project.deploy("../examples/rocket/secrets").await;
        project.stop_service().await;

        assert_eq!(
            project.router_call(Method::DELETE, "/delete").await,
            StatusCode::OK
        );
    }

    #[test_context(TestProject)]
    #[tokio::test]
    async fn api_delete_project_that_has_resources_but_fails_to_remove_them(
        project: &mut TestProject,
    ) {
        project.deploy("../examples/axum/metadata").await;
        project.stop_service().await;

        assert_eq!(
            project.router_call(Method::DELETE, "/delete").await,
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test_context(TestProject)]
    #[tokio::test]
    async fn api_delete_project_that_has_running_deployment(project: &mut TestProject) {
        project.deploy("../examples/axum/hello-world").await;

        assert_eq!(
            project.router_call(Method::DELETE, "/delete").await,
            StatusCode::OK
        );
    }

    #[test_context(TestProject)]
    #[tokio::test]
    async fn api_delete_project_that_is_building(project: &mut TestProject) {
        project.just_deploy("../examples/axum/hello-world").await;

        // Wait a bit to it to progress in the queue
        sleep(Duration::from_secs(2)).await;

        assert_eq!(
            project.router_call(Method::DELETE, "/delete").await,
            StatusCode::BAD_REQUEST
        );
    }

    #[test_context(TestProject)]
    #[tokio::test]
    async fn api_delete_project_that_is_errored(project: &mut TestProject) {
        project
            .update_state(Project::Errored(ProjectError::internal(
                "Mr. Anderson is here",
            )))
            .await;

        assert_eq!(
            project.router_call(Method::DELETE, "/delete").await,
            StatusCode::OK
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn status() {
        let world = World::new().await;
        let service = Arc::new(GatewayService::init(world.args(), world.pool(), "".into()).await);

        let (sender, mut receiver) = channel::<BoxedTask>(1);
        let (ctl_send, ctl_recv) = oneshot::channel();
        let (done_send, done_recv) = oneshot::channel();
        let worker = tokio::spawn(async move {
            let mut done_send = Some(done_send);
            // do not process until instructed
            ctl_recv.await.unwrap();

            while receiver.recv().await.is_some() {
                done_send.take().unwrap().send(()).unwrap();
                // do nothing
            }
        });

        let mut router = ApiBuilder::new()
            .with_service(Arc::clone(&service))
            .with_sender(sender)
            .with_default_routes()
            .with_auth_service(world.context().auth_uri)
            .into_router();

        let get_status = || {
            Request::builder()
                .method("GET")
                .uri("/")
                .body(Body::empty())
                .unwrap()
        };

        let resp = router.call(get_status()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let matrix: ProjectName = "matrix".parse().unwrap();

        let neo_key = world.create_user("neo");
        let authorization = Authorization::bearer(&neo_key).unwrap();

        let create_project = Request::builder()
            .method("POST")
            .uri(format!("/projects/{matrix}"))
            .header("Content-Type", "application/json")
            .body("{\"idle_minutes\": 3}".into())
            .unwrap()
            .with_header(&authorization);

        router.call(create_project).await.unwrap();

        let resp = router.call(get_status()).await.unwrap();
        let body = to_bytes(resp.into_body()).await.unwrap();

        // The status check response will be a JSON array of objects.
        let resp: Value = serde_json::from_slice(&body).unwrap();

        // The gateway health status will always be the first element in the array.
        assert_eq!(resp[0][1]["status"], "unhealthy".to_string());

        ctl_send.send(()).unwrap();
        done_recv.await.unwrap();

        let resp = router.call(get_status()).await.unwrap();
        let body = to_bytes(resp.into_body()).await.unwrap();

        let resp: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp[0][1]["status"], "degraded".to_string());

        worker.abort();
        let _ = worker.await;

        let resp = router.call(get_status()).await.unwrap();
        let body = to_bytes(resp.into_body()).await.unwrap();

        let resp: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp[0][1]["status"], "unhealthy".to_string());
    }
}
