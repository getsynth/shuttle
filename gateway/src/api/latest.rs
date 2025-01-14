use std::io::Cursor;
use std::net::SocketAddr;
use std::ops::Sub;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Extension, Path, Query, State};
use axum::handler::Handler;
use axum::http::Request;
use axum::middleware::{self, from_extractor};
use axum::response::Response;
use axum::routing::{any, delete, get, post};
use axum::{Json as AxumJson, Router};
use fqdn::FQDN;
use futures::Future;
use http::header::AUTHORIZATION;
use http::{request, HeaderValue, Method, StatusCode, Uri};
use instant_acme::{AccountCredentials, ChallengeType};
use serde::{Deserialize, Serialize};
use shuttle_backends::auth::{AuthPublicKey, JwtAuthenticationLayer, ScopedLayer};
use shuttle_backends::axum::CustomErrorPath;
use shuttle_backends::cache::CacheManager;
use shuttle_backends::client::permit::Team;
use shuttle_backends::metrics::{Metrics, TraceLayer};
use shuttle_backends::project_name::ProjectName;
use shuttle_backends::request_span;
use shuttle_backends::ClaimExt;
use shuttle_common::claims::{Claim, Scope, EXP_MINUTES};
use shuttle_common::models::error::{
    ApiError, Deprecated, InvalidCustomDomain, InvalidTeamName, ProjectCorrupted,
    ProjectHasBuildingDeployment, ProjectHasResources, ProjectHasRunningDeployment,
};
use shuttle_common::models::{admin::ProjectResponse, project, stats};
use shuttle_common::models::{service, team};
use shuttle_common::{deployment, VersionInfo};
use shuttle_proto::provisioner::provisioner_client::ProvisionerClient;
use shuttle_proto::provisioner::Ping;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, MutexGuard};
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{debug, field, info, instrument, trace, warn, Span};
use ttl_cache::TtlCache;
use ulid::Ulid;
use uuid::Uuid;
use x509_parser::nom::AsBytes;
use x509_parser::parse_x509_certificate;
use x509_parser::pem::parse_x509_pem;
use x509_parser::time::ASN1Time;

use crate::acme::{AccountWrapper, AcmeClient, CustomDomain};
use crate::api::tracing::project_name_tracing_layer;
use crate::auth::ScopedUser;
use crate::service::{ContainerSettings, GatewayService};
use crate::task::{self, BoxedTask};
use crate::tls::{GatewayCertResolver, RENEWAL_VALIDITY_THRESHOLD_IN_DAYS};
use crate::worker::WORKER_QUEUE_SIZE;
use crate::{DockerContext, AUTH_CLIENT};

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

#[derive(Debug, Clone, Copy, Deserialize)]
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
async fn get_project(
    State(RouterState { service, .. }): State<RouterState>,
    ScopedUser { scope, claim }: ScopedUser,
) -> Result<AxumJson<project::Response>, ApiError> {
    let project = service.find_project_by_name(&scope).await?;
    let idle_minutes = project.state.idle_minutes();
    let owner = service
        .permit_client
        .get_project_owner(&claim.sub, &project.id)
        .await?
        .into();
    let is_admin = service
        .permit_client
        .allowed(&claim.sub, &project.id, "manage")
        .await?;

    let response = project::Response {
        id: project.id.to_uppercase(),
        name: scope.to_string(),
        state: project.state.into(),
        idle_minutes,
        owner,
        is_admin,
    };

    Ok(AxumJson(response))
}

#[instrument(skip(service))]
async fn check_project_name(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath(project_name): CustomErrorPath<ProjectName>,
) -> Result<AxumJson<bool>, ApiError> {
    let res = service.project_name_exists(&project_name).await?;

    Ok(AxumJson(res))
}
async fn get_projects_list(
    State(RouterState { service, .. }): State<RouterState>,
    Claim { sub, .. }: Claim,
) -> Result<AxumJson<Vec<project::Response>>, ApiError> {
    let project_ids = service.permit_client.get_personal_projects(&sub).await?;
    let mut projects = Vec::with_capacity(project_ids.len());
    for proj_id in project_ids {
        // Ignore projects on the beta platform
        if proj_id.starts_with("proj_") {
            continue;
        }
        let project = service.find_project_by_id(&proj_id).await?;
        let idle_minutes = project.state.idle_minutes();
        let owner = service
            .permit_client
            .get_project_owner(&sub, &proj_id)
            .await?
            .into();
        let is_admin = service
            .permit_client
            .allowed(&sub, &proj_id, "manage")
            .await?;

        let response = project::Response {
            id: project.id,
            name: project.name,
            state: project.state.into(),
            idle_minutes,
            owner,
            is_admin,
        };
        projects.push(response);
    }
    // sort by descending id
    projects.sort_by(|p1, p2| p2.id.cmp(&p1.id));

    Ok(AxumJson(projects))
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name))]
async fn create_project(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
    claim: Claim,
    CustomErrorPath(project_name): CustomErrorPath<ProjectName>,
    AxumJson(config): AxumJson<project::Config>,
) -> Result<AxumJson<project::Response>, ApiError> {
    let is_cch_project = project_name.is_cch_project();

    // Check that the user is within their project limits.
    let can_create_project = claim.can_create_project(
        service
            .get_project_count(&claim.sub)
            .await?
            .saturating_sub(is_cch_project as u32),
    );

    if !claim.is_admin() {
        service.has_capacity(is_cch_project, &claim.tier).await?;
    }

    let project = service
        .create_project(
            project_name.clone(),
            &claim.sub,
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
        id: project.id.to_string().to_uppercase(),
        name: project_name.to_string(),
        state: project.state.into(),
        idle_minutes,
        owner: project::Owner::User(claim.sub),
        is_admin: true,
    };

    Ok(AxumJson(response))
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name))]
async fn destroy_project(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
    ScopedUser {
        scope: project_name,
        claim,
        ..
    }: ScopedUser,
) -> Result<AxumJson<project::Response>, ApiError> {
    let project = service.find_project_by_name(&project_name).await?;
    let idle_minutes = project.state.idle_minutes();
    let owner = service
        .permit_client
        .get_project_owner(&claim.sub, &project.id)
        .await?
        .into();
    let is_admin = service
        .permit_client
        .allowed(&claim.sub, &project.id, "manage")
        .await?;

    let mut response = project::Response {
        id: project.id.to_uppercase(),
        name: project_name.to_string(),
        state: project.state.into(),
        idle_minutes,
        owner,
        is_admin,
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

#[derive(Deserialize)]
struct DeleteProjectParams {
    // Was added in v0.30.0
    // We have not needed it since 0.34.1, but have to keep in for any old CLI users
    #[allow(dead_code)]
    dry_run: Option<bool>,
}

#[instrument(skip_all, fields(shuttle.project.name = %scoped_user.scope))]
async fn delete_project(
    State(state): State<RouterState>,
    scoped_user: ScopedUser,
    Query(DeleteProjectParams { dry_run }): Query<DeleteProjectParams>,
    req: Request<Body>,
) -> Result<AxumJson<String>, ApiError> {
    // Don't do the dry run that might come from older CLIs
    if dry_run.is_some_and(|d| d) {
        return Ok(AxumJson("dry run is no longer supported".to_owned()));
    }

    let project_name = scoped_user.scope.clone();
    let project = state.service.find_project_by_name(&project_name).await?;

    let project_id = Ulid::from_string(&project.id).expect("stored project id to be a valid ULID");

    // We restart the project before deletion everytime
    let handle = state
        .service
        .new_task()
        .project(project_name.clone())
        .and_then(task::destroy()) // This destroy might only recover the project from an errored state
        .and_then(task::run_until_destroyed())
        .and_then(task::restart(project_id))
        .and_then(task::run_until_ready())
        .and_then(task::destroy())
        .and_then(task::run_until_destroyed())
        .and_then(task::restart(project_id))
        .and_then(task::run_until_ready())
        .send(&state.sender)
        .await?;

    // Wait for the project to be ready
    handle.await;

    let new_state = state.service.find_project_by_name(&project_name).await?;

    if !new_state.state.is_ready() {
        warn!(state = ?new_state.state, "failed to restart project");
        return Err(ProjectCorrupted.into());
    }

    let service = state.service.clone();
    let sender = state.sender.clone();

    let project_caller =
        ProjectCaller::new(state.clone(), scoped_user.clone(), req.headers()).await?;

    trace!("getting deployments");
    // check that a deployment is not running
    let mut deployments = project_caller.get_deployment_list().await?;
    debug!(?deployments, "got deployments");
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
        warn!("has bad state");
        return Err(ProjectHasBuildingDeployment.into());
    }

    let running_deployments = deployments
        .into_iter()
        .filter(|d| d.state == deployment::State::Running);

    for running_deployment in running_deployments {
        info!(%running_deployment, "stopping running deployment");
        let res = project_caller
            .stop_deployment(&running_deployment.id)
            .await?;

        if res.status() != StatusCode::OK {
            return Err(ProjectHasRunningDeployment.into());
        }
    }

    trace!("getting resources");
    // check if any resources exist
    let resources = project_caller.get_resources().await?;
    let mut delete_fails = Vec::new();

    for resource in resources {
        info!(?resource, "deleting resource");
        let resource_type = resource.r#type.to_string();
        let res = project_caller.delete_resource(&resource_type).await?;

        if res.status() != StatusCode::OK {
            delete_fails.push(resource_type)
        }
    }

    if !delete_fails.is_empty() {
        return Err(ProjectHasResources(delete_fails).into());
    }

    trace!("deleting container");
    let task = service
        .new_task()
        .project(project_name.clone())
        .and_then(task::delete_project())
        .send(&sender)
        .await?;
    task.await;

    trace!("removing project from state");
    service.delete_project(&project_name).await?;

    Ok(AxumJson("project successfully deleted".to_owned()))
}

#[derive(Deserialize)]
struct DeployQuery {
    force: Option<bool>,
}

#[instrument(skip_all, fields(shuttle.project.name = %scoped_user.scope))]
async fn override_create_service(
    state: State<RouterState>,
    scoped_user: ScopedUser,
    Query(query): Query<DeployQuery>,
    req: Request<Body>,
) -> Result<Response<Body>, ApiError> {
    if query.force.is_some_and(|t| t) {
        route_project(state, scoped_user, req).await
    } else {
        Err(Deprecated(
            "Creating new deployments on the shuttle.rs platform has been deprecated.".to_string(),
        ))?
    }
}

#[instrument(skip_all, fields(shuttle.project.name = %scoped_user.scope))]
async fn override_get_delete_service(
    state: State<RouterState>,
    scoped_user: ScopedUser,
    req: Request<Body>,
) -> Result<Response<Body>, ApiError> {
    let project_name = scoped_user.scope.to_string();
    let service = state.service.clone();
    let ctx = state.service.context().clone();
    let ContainerSettings { fqdn: public, .. } = ctx.container_settings();
    let mut res = route_project(state, scoped_user, req).await?;

    // inject the (most relevant) URI that this project is being served on
    let uri = service
        .find_custom_domain_for_project(&project_name)
        .await
        .unwrap_or_default() // use project name if domain lookup fails
        .map(|c| format!("https://{}", c.fqdn))
        .unwrap_or_else(|| format!("https://{project_name}.{public}"));
    let body = hyper::body::to_bytes(res.body_mut()).await.unwrap();
    let mut json: service::Summary =
        serde_json::from_slice(body.as_bytes()).expect("valid service response from deployer");
    json.uri = uri;

    let bytes = serde_json::to_vec(&json).unwrap();
    let len = res
        .headers_mut()
        .entry("content-length")
        .or_insert(0.into());
    *len = bytes.len().into();
    *res.body_mut() = bytes.into();

    Ok(res)
}

#[instrument(skip_all, fields(shuttle.project.name = %scoped_user.scope))]
async fn route_project(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
    scoped_user: ScopedUser,
    req: Request<Body>,
) -> Result<Response<Body>, ApiError> {
    let project_name = scoped_user.scope;
    let is_cch_project = project_name.is_cch_project();

    if !scoped_user.claim.is_admin() {
        service
            .has_capacity(is_cch_project, &scoped_user.claim.tier)
            .await?;
    }

    let project = service
        .find_or_start_project(&project_name, sender)
        .await?
        .0;

    let res = service
        .route(&project.state, &project_name, &scoped_user.claim.sub, req)
        .await?;

    Ok(res)
}

#[instrument(skip_all)]
async fn get_teams(
    State(RouterState { service, .. }): State<RouterState>,
    Claim { sub, .. }: Claim,
) -> Result<AxumJson<Vec<team::Response>>, ApiError> {
    let teams = service.permit_client.get_teams(&sub).await?;

    Ok(AxumJson(teams))
}

#[instrument(skip_all)]
async fn get_team(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath(team_id): CustomErrorPath<String>,
    Claim { sub, .. }: Claim,
) -> Result<AxumJson<team::Response>, ApiError> {
    let team = service.permit_client.get_team(&sub, &team_id).await?;

    Ok(AxumJson(team))
}

#[instrument(skip_all, fields(shuttle.team.name = %team_name, shuttle.team.id = field::Empty))]
async fn create_team(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath(team_name): CustomErrorPath<String>,
    Claim { sub, .. }: Claim,
) -> Result<AxumJson<team::Response>, ApiError> {
    if team_name.chars().count() > 30 {
        return Err(InvalidTeamName.into());
    }

    let team = Team {
        id: format!("team_{}", Ulid::new()),
        display_name: team_name.clone(),
    };

    service.permit_client.create_team(&sub, &team).await?;

    Span::current().record("shuttle.team.id", &team.id);

    Ok(AxumJson(team::Response {
        id: team.id,
        display_name: team.display_name,
        is_admin: true,
    }))
}

#[instrument(skip_all, fields(shuttle.team.id = %team_id))]
async fn get_team_projects(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath(team_id): CustomErrorPath<String>,
    Claim { sub, .. }: Claim,
) -> Result<AxumJson<Vec<project::Response>>, ApiError> {
    let project_ids = service
        .permit_client
        .get_team_projects(&sub, &team_id)
        .await?;
    let mut projects = Vec::with_capacity(project_ids.len());
    for proj_id in project_ids {
        // Ignore projects on the beta platform
        if proj_id.starts_with("proj_") {
            continue;
        }
        let project = service.find_project_by_id(&proj_id).await?;
        let idle_minutes = project.state.idle_minutes();
        let owner = service
            .permit_client
            .get_project_owner(&sub, &proj_id)
            .await?
            .into();
        let is_admin = service
            .permit_client
            .allowed(&sub, &proj_id, "manage")
            .await?;

        projects.push(project::Response {
            id: project.id,
            name: project.name,
            state: project.state.into(),
            idle_minutes,
            owner,
            is_admin,
        });
    }

    Ok(AxumJson(projects))
}

#[instrument(skip_all, fields(shuttle.team.id = %team_id))]
async fn delete_team(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath(team_id): CustomErrorPath<String>,
    Claim { sub, .. }: Claim,
) -> Result<String, ApiError> {
    service.permit_client.delete_team(&sub, &team_id).await?;

    Ok("Team deleted".to_string())
}

#[instrument(skip_all, fields(shuttle.team.id = %team_id, shuttle.project.id = %project_id))]
async fn transfer_project_to_team(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath((team_id, project_id)): CustomErrorPath<(String, String)>,
    Claim { sub, .. }: Claim,
) -> Result<String, ApiError> {
    service
        .permit_client
        .transfer_project_to_team(&sub, &project_id, &team_id)
        .await?;

    Ok("Project transfered".to_string())
}

#[instrument(skip_all, fields(shuttle.team.id = %team_id, shuttle.project.id = %project_id))]
async fn transfer_project_from_team(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath((team_id, project_id)): CustomErrorPath<(String, String)>,
    Claim { sub, .. }: Claim,
) -> Result<String, ApiError> {
    service
        .permit_client
        .transfer_project_from_team(&sub, &project_id, &team_id)
        .await?;

    Ok("Project transfered".to_string())
}

#[instrument(skip_all, fields(shuttle.team.id = %team_id))]
async fn get_team_members(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath(team_id): CustomErrorPath<String>,
    Claim { sub, .. }: Claim,
) -> Result<AxumJson<Vec<team::MemberResponse>>, ApiError> {
    let members = service
        .permit_client
        .get_team_members(&sub, &team_id)
        .await?;

    Ok(AxumJson(members))
}

#[instrument(skip_all, fields(shuttle.team.id = %team_id))]
async fn add_member_to_team(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath((team_id, user_id)): CustomErrorPath<(String, String)>,
    Claim { sub, .. }: Claim,
) -> Result<String, ApiError> {
    service
        .permit_client
        .add_team_member(&sub, &team_id, &user_id)
        .await?;

    Ok("Member added".to_string())
}

#[instrument(skip_all, fields(shuttle.team.id = %team_id))]
async fn remove_member_from_team(
    State(RouterState { service, .. }): State<RouterState>,
    CustomErrorPath((team_id, user_id)): CustomErrorPath<(String, String)>,
    Claim { sub, .. }: Claim,
) -> Result<String, ApiError> {
    service
        .permit_client
        .remove_team_member(&sub, &team_id, &user_id)
        .await?;

    Ok("Member removed".to_string())
}

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
    let provisioner_status = if let Ok(channel) = service.provisioner_uri().connect().await {
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
async fn post_load(
    State(RouterState { running_builds, .. }): State<RouterState>,
    AxumJson(build): AxumJson<stats::LoadRequest>,
) -> Result<AxumJson<stats::LoadResponse>, ApiError> {
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
async fn delete_load(
    State(RouterState { running_builds, .. }): State<RouterState>,
    AxumJson(build): AxumJson<stats::LoadRequest>,
) -> Result<AxumJson<stats::LoadResponse>, ApiError> {
    let mut running_builds = running_builds.lock().await;
    running_builds.remove(&build.id);

    trace!(id = %build.id, "removing from build queue");
    let load = calculate_capacity(&mut running_builds);

    Ok(AxumJson(load))
}

#[instrument(skip_all)]
async fn get_load_admin(
    State(RouterState { running_builds, .. }): State<RouterState>,
) -> Result<AxumJson<stats::LoadResponse>, ApiError> {
    let mut running_builds = running_builds.lock().await;

    let load = calculate_capacity(&mut running_builds);

    Ok(AxumJson(load))
}

#[instrument(skip_all)]
async fn delete_load_admin(
    State(RouterState { running_builds, .. }): State<RouterState>,
) -> Result<AxumJson<stats::LoadResponse>, ApiError> {
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
async fn revive_projects(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
) -> Result<(), ApiError> {
    crate::project::exec::revive(service, sender).await?;

    Ok(())
}

#[instrument(skip_all)]
async fn idle_cch_projects(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
) -> Result<(), ApiError> {
    crate::project::exec::idle_cch(service, sender).await?;

    Ok(())
}

#[instrument(skip_all)]
async fn destroy_projects(
    State(RouterState {
        service, sender, ..
    }): State<RouterState>,
) -> Result<(), ApiError> {
    crate::project::exec::destroy(service, sender).await?;

    Ok(())
}

#[instrument(skip_all, fields(%email, ?acme_server))]
async fn create_acme_account(
    Extension(acme_client): Extension<AcmeClient>,
    Path(email): Path<String>,
    AxumJson(acme_server): AxumJson<Option<String>>,
) -> Result<AxumJson<serde_json::Value>, ApiError> {
    let res = acme_client.create_account(&email, acme_server).await?;

    Ok(AxumJson(res))
}

#[instrument(skip_all, fields(shuttle.project.name = %project_name, %fqdn))]
async fn request_custom_domain_acme_certificate(
    State(RouterState { service, .. }): State<RouterState>,
    Extension(acme_client): Extension<AcmeClient>,
    Extension(resolver): Extension<Arc<GatewayCertResolver>>,
    CustomErrorPath((project_name, fqdn)): CustomErrorPath<(ProjectName, String)>,
    AxumJson(credentials): AxumJson<AccountCredentials<'_>>,
) -> Result<String, ApiError> {
    let fqdn: FQDN = fqdn.parse().map_err(|_| InvalidCustomDomain)?;

    let (certs, private_key) = service
        .create_custom_domain_certificate(&fqdn, &acme_client, &project_name, credentials)
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

#[instrument(skip_all, fields(shuttle.project.name = %project_name, %fqdn))]
async fn renew_custom_domain_acme_certificate(
    State(RouterState { service, .. }): State<RouterState>,
    Extension(acme_client): Extension<AcmeClient>,
    Extension(resolver): Extension<Arc<GatewayCertResolver>>,
    CustomErrorPath((project_name, fqdn)): CustomErrorPath<(ProjectName, String)>,
    AxumJson(credentials): AxumJson<AccountCredentials<'_>>,
) -> Result<String, ApiError> {
    let fqdn: FQDN = fqdn.parse().map_err(|_| InvalidCustomDomain)?;
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
                ApiError::internal(&format!(
                    "Error while parsing the pem certificate for {project_name}: {err}"
                ))
            })?;

            let (_, x509_cert_chain) =
                parse_x509_certificate(pem.contents.as_bytes()).map_err(|err| {
                    ApiError::internal(&format!(
                        "Error while parsing the certificate chain for {project_name}: {err}"
                    ))
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
        Err(err) => Err(err.into()),
    }
}

#[instrument(skip_all)]
async fn renew_gateway_acme_certificate(
    State(RouterState { service, .. }): State<RouterState>,
    Extension(acme_client): Extension<AcmeClient>,
    Extension(resolver): Extension<Arc<GatewayCertResolver>>,
    AxumJson(credentials): AxumJson<AccountCredentials<'_>>,
) -> Result<String, ApiError> {
    let account = AccountWrapper::from(credentials).0;
    let certs = service
        .fetch_certificate(&acme_client, account.credentials())
        .await;
    // Safe to unwrap because a 'ChainAndPrivateKey' is built from a PEM.
    let chain_and_pk = certs.into_pem().unwrap();

    let (_, pem) = parse_x509_pem(chain_and_pk.as_bytes())
        .unwrap_or_else(|_| panic!("Malformed existing PEM certificate for the gateway."));
    let (_, x509_cert) = parse_x509_certificate(pem.contents.as_bytes())
        .unwrap_or_else(|_| panic!("Malformed existing X509 certificate for the gateway."));

    // We compute the difference between the certificate expiry date and current timestamp because we want to trigger the
    // gateway certificate renewal only during it's last 30 days of validity or if the certificate is expired.
    let diff = x509_cert.validity().not_after.sub(ASN1Time::now());

    // Renew only when the difference is `None` (meaning certificate expired) or we're within the last 30 days of validity.
    if diff.is_none()
        || diff
            .expect("to be Some given we checked for None previously")
            .whole_days()
            <= RENEWAL_VALIDITY_THRESHOLD_IN_DAYS
    {
        let tls_path = service.state_dir.join("ssl.pem");
        let certs = service
            .create_certificate(&acme_client, account.credentials())
            .await;
        resolver
            .serve_default_der(certs.clone())
            .await
            .expect("Failed to serve the default certs");
        certs
            .save_pem(&tls_path)
            .expect("to save the certificate locally");
        return Ok(r#""Renewed the gateway certificate.""#.to_string());
    }

    Ok(format!(
        "\"Gateway certificate was not renewed. There are {} days until the certificate expires.\"",
        diff.expect("to be Some given we checked for None previously")
            .whole_days()
    ))
}

async fn get_projects(
    State(RouterState { service, .. }): State<RouterState>,
) -> Result<AxumJson<Vec<ProjectResponse>>, ApiError> {
    let projects = service
        .iter_projects_detailed()
        .await?
        .map(Into::into)
        .collect();

    Ok(AxumJson(projects))
}

async fn change_project_owner(
    State(RouterState { service, .. }): State<RouterState>,
    Path((project_name, new_user_id)): Path<(String, String)>,
) -> Result<(), ApiError> {
    service
        .update_project_owner(&project_name, &new_user_id)
        .await?;

    Ok(())
}

#[derive(Clone)]
pub(crate) struct RouterState {
    pub service: Arc<GatewayService>,
    pub sender: Sender<BoxedTask>,
    pub running_builds: Arc<Mutex<TtlCache<Uuid, ()>>>,
}

#[derive(Default)]
pub struct ApiBuilder {
    router: Router<RouterState>,
    service: Option<Arc<GatewayService>>,
    sender: Option<Sender<BoxedTask>>,
    bind: Option<SocketAddr>,
}

impl ApiBuilder {
    pub fn new() -> Self {
        Self::default()
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
                    account.user_id = field::Empty,
                    request.params.project_name = field::Empty,
                    request.params.user_id = field::Empty,
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
            .route(
                "/projects/change-owner/:project_name/:new_user_id",
                get(change_project_owner),
            )
            .route("/revive", post(revive_projects))
            .route("/destroy", post(destroy_projects))
            .route("/idle-cch", post(idle_cch_projects))
            .route("/stats/load", get(get_load_admin).delete(delete_load_admin))
            .layer(ScopedLayer::new(vec![Scope::Admin]));

        const CARGO_SHUTTLE_VERSION: &str = env!("CARGO_PKG_VERSION");

        let project_routes = Router::new()
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
            .route(
                // catch these deployer endpoints for extra metrics or processing before/after being proxied
                "/projects/:project_name/services/:service_name",
                post(override_create_service)
                    .get(override_get_delete_service)
                    .delete(override_get_delete_service),
            )
            .route("/projects/:project_name/*any", any(route_project))
            .route_layer(middleware::from_fn(project_name_tracing_layer));

        let team_routes = Router::new()
            .route("/", get(get_teams))
            .route("/name/:team_name", post(create_team))
            .route("/:team_id", get(get_team).delete(delete_team))
            .route("/:team_id/projects", get(get_team_projects))
            .route(
                "/:team_id/projects/:project_id",
                post(transfer_project_to_team).delete(transfer_project_from_team),
            )
            .route("/:team_id/members", get(get_team_members))
            .route(
                "/:team_id/members/:user_id",
                post(add_member_to_team).delete(remove_member_from_team),
            );

        self.router = self
            .router
            .route("/", get(get_status))
            .merge(project_routes)
            .nest("/teams", team_routes)
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
            .route("/stats/load", post(post_load).delete(delete_load))
            .nest("/admin", admin_routes);

        self
    }

    pub fn with_auth_service(mut self, auth_uri: Uri, gateway_admin_key: String) -> Self {
        let auth_public_key = AuthPublicKey::new(auth_uri.clone());

        let jwt_cache_manager = CacheManager::new(1000);

        self.router = self
            .router
            .layer(JwtAuthenticationLayer::new(auth_public_key))
            .layer(ShuttleAuthLayer::new(
                auth_uri,
                gateway_admin_key,
                Arc::new(Box::new(jwt_cache_manager)),
            ));

        self
    }

    pub fn with_cors(mut self, cors_origin: &str) -> Self {
        let cors_origin = cors_origin.to_owned();

        let cors_layer = CorsLayer::new()
            .allow_methods(vec![Method::GET, Method::POST, Method::DELETE])
            .allow_headers(vec![AUTHORIZATION])
            .max_age(Duration::from_secs(60) * 10)
            .allow_origin(AllowOrigin::predicate(
                move |origin: &HeaderValue, _request_parts: &request::Parts| {
                    origin.as_bytes().ends_with(cors_origin.as_bytes())
                },
            ));

        self.router = self.router.layer(cors_layer);

        self
    }

    pub fn into_router(self) -> Router {
        let service = self.service.expect("a GatewayService is required");
        let sender = self.sender.expect("a task Sender is required");

        // Allow about 4 cores per build, but use at most 75% (* 3 / 4) of all cores and at least 1 core
        // Assumes each builder (deployer) is assigned 4 cores
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
    use shuttle_backends::test_utils::gateway::PermissionsMock;
    use shuttle_common::constants::limits::{MAX_PROJECTS_DEFAULT, MAX_PROJECTS_EXTRA};
    use shuttle_common::models::user::AccountTier;
    use test_context::test_context;
    use tokio::sync::mpsc::channel;
    use tokio::sync::oneshot;
    use tokio::time::sleep;
    use tower::Service;

    use super::*;
    use crate::project::Project;
    use crate::project::ProjectError;
    use crate::service::GatewayService;
    use crate::tests::{RequestBuilderExt, TestGateway, TestProject, World};

    #[tokio::test]
    async fn api_create_get_delete_projects() -> anyhow::Result<()> {
        let world = World::new().await;
        let service = Arc::new(
            GatewayService::init(
                world.args(),
                world.pool(),
                "".into(),
                Box::<PermissionsMock>::default(),
            )
            .await?,
        );

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
            .with_auth_service(world.context().auth_uri, "dummykey".to_string())
            .into_router();

        let neo_key = world.create_user("neo", AccountTier::Basic);

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

        let trinity_key = world.create_user("trinity", AccountTier::Basic);

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
        let admin_neo_key = world.create_user("admin-neo", AccountTier::Basic);
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
        let service = Arc::new(
            GatewayService::init(
                world.args(),
                world.pool(),
                "".into(),
                Box::<PermissionsMock>::default(),
            )
            .await?,
        );

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
            .with_auth_service(world.context().auth_uri, "dummykey".to_string())
            .into_router();

        let neo_key = world.create_user("neo", AccountTier::Basic);

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
        let trinity_key = world.create_user("trinity", AccountTier::Basic);
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

        // It should be possible to still create a normal project
        let _normal_project = gateway.create_project("project").await;

        let more_code = gateway.try_create_project("project-normal-2").await;

        assert_eq!(
            more_code,
            StatusCode::SERVICE_UNAVAILABLE,
            "more normal projects should not go over soft limit"
        );

        // A pro user can go over the soft limits
        let pro_user = gateway.new_authorization_bearer("trinity", AccountTier::Pro);
        let _long_running = gateway.user_create_project("reload", &pro_user).await;

        // A pro user cannot go over the hard limits
        let code = gateway
            .try_user_create_project("training-simulation", &pro_user)
            .await;

        assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test_context(TestGateway)]
    #[tokio::test]
    async fn start_idle_project_when_above_container_limit(gateway: &mut TestGateway) {
        let mut cch_idle_project = gateway.create_project("cch23-project").await;
        // RUNNING PROJECTS = 1 [cch_idle_project]
        // Run four health checks to get the project to go into idle mode (cch projects always default to 5 min of idle time)
        cch_idle_project.run_health_check().await;
        cch_idle_project.run_health_check().await;
        cch_idle_project.run_health_check().await;
        cch_idle_project.run_health_check().await;

        cch_idle_project
            .wait_for_state(project::State::Stopped)
            .await;
        // RUNNING PROJECTS = 0 []
        let mut normal_idle_project = gateway.create_project("project").await;
        // RUNNING PROJECTS = 1 [normal_idle_project]
        // Run two health checks to get the project to go into idle mode
        normal_idle_project.run_health_check().await;
        normal_idle_project.run_health_check().await;

        normal_idle_project
            .wait_for_state(project::State::Stopped)
            .await;
        // RUNNING PROJECTS = 0 []
        let mut normal_idle_project2 = gateway.create_project("project-2").await;
        // RUNNING PROJECTS = 1 [normal_idle_project2]
        // Run two health checks to get the project to go into idle mode
        normal_idle_project2.run_health_check().await;
        normal_idle_project2.run_health_check().await;

        normal_idle_project2
            .wait_for_state(project::State::Stopped)
            .await;
        // RUNNING PROJECTS = 0 []
        let pro_user = gateway.new_authorization_bearer("trinity", AccountTier::Pro);
        let mut long_running = gateway.user_create_project("matrix", &pro_user).await;
        // RUNNING PROJECTS = 1 [long_running]
        // Now try to start the idle projects
        let cch_code = cch_idle_project
            .router_call(Method::GET, "/services/cch23-project")
            .await;
        // RUNNING PROJECTS = 1 [long_running]

        assert_eq!(cch_code, StatusCode::SERVICE_UNAVAILABLE);

        let normal_code = normal_idle_project
            .router_call(Method::GET, "/services/project")
            .await;
        // RUNNING PROJECTS = 2 [long_running, normal_idle_project]

        assert_eq!(
            normal_code,
            StatusCode::NOT_FOUND,
            "should not be able to find a service since nothing was deployed"
        );

        let normal_code2 = normal_idle_project2
            .router_call(Method::GET, "/services/project")
            .await;
        // RUNNING PROJECTS = 2 [long_running, normal_idle_project]

        assert_eq!(
            normal_code2,
            StatusCode::SERVICE_UNAVAILABLE,
            "should not be able to wake project that will go over soft limit"
        );

        // Now try to start a pro user's project
        // Have it idle so that we can wake it up
        long_running.run_health_check().await;
        long_running.run_health_check().await;

        long_running.wait_for_state(project::State::Stopped).await;
        // RUNNING PROJECTS = 1 [normal_idle_project]

        let normal_code2 = normal_idle_project2
            .router_call(Method::GET, "/services/project")
            .await;
        // RUNNING PROJECTS = 2 [normal_idle_project, normal_idle_project2]

        assert_eq!(
            normal_code2,
            StatusCode::NOT_FOUND,
            "should not be able to find a service since nothing was deployed"
        );

        let long_running_code = long_running
            .router_call(Method::GET, "/services/project")
            .await;
        // RUNNING PROJECTS = 3 [normal_idle_project, normal_idle_project2, long_running]

        assert_eq!(
            long_running_code,
            StatusCode::NOT_FOUND,
            "should be able to wake the project of a pro user. Even if we are over the soft limit"
        );

        // Now try to start a pro user's project when we are at the hard limit
        long_running.run_health_check().await;
        long_running.run_health_check().await;

        long_running.wait_for_state(project::State::Stopped).await;
        // RUNNING PROJECTS = 2 [normal_idle_project, normal_idle_project2]
        let _extra = gateway.user_create_project("reloaded", &pro_user).await;
        // RUNNING PROJECTS = 3 [normal_idle_project, normal_idle_project2, _extra]

        let long_running_code = long_running
            .router_call(Method::GET, "/services/project")
            .await;
        // RUNNING PROJECTS = 3 [normal_idle_project, normal_idle_project2, _extra]

        assert_eq!(
            long_running_code,
            StatusCode::SERVICE_UNAVAILABLE,
            "should be able to wake the project of a pro user. Even if we are over the soft limit"
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
        sleep(Duration::from_secs(10)).await;

        assert_eq!(
            project.router_call(Method::DELETE, "/delete").await,
            StatusCode::OK
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
        let service = Arc::new(
            GatewayService::init(
                world.args(),
                world.pool(),
                "".into(),
                Box::<PermissionsMock>::default(),
            )
            .await
            .unwrap(),
        );

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
            .with_auth_service(world.context().auth_uri, "dummykey".to_string())
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

        let neo_key = world.create_user("neo", AccountTier::Basic);
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
