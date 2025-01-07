use std::io;
use std::io::Cursor;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use axum::body::Body;
use axum::headers::{Authorization, HeaderMapExt};
use axum::http::Request;
use axum::response::Response;
use bollard::container::StatsOptions;
use bollard::{Docker, API_DEFAULT_VERSION};
use fqdn::{Fqdn, FQDN};
use http::{StatusCode, Uri};
use hyper::client::connect::dns::GaiResolver;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper_reverse_proxy::ReverseProxy;
use instant_acme::{AccountCredentials, ChallengeType};
use once_cell::sync::Lazy;
use opentelemetry::global;
use opentelemetry_http::HeaderInjector;
use shuttle_backends::client::{permit, PermissionsDal};
use shuttle_backends::headers::XShuttleAdminSecret;
use shuttle_backends::project_name::ProjectName;
use shuttle_common::constants::SHUTTLE_IDLE_DOCS_URL;
use shuttle_common::models::error::{
    ApiError, Deprecated, ProjectNotFound, ProjectNotReady, ProjectUnavailable,
};
use shuttle_common::models::project::State;
use shuttle_common::models::user::{AccountTier, UserId};
use sqlx::error::DatabaseError;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePool;
use sqlx::types::Json as SqlxJson;
use sqlx::{query, query_as, Error as SqlxError, QueryBuilder, Row};
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tokio::time::timeout;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::transport::Endpoint;
use tracing::{debug, error, info, instrument, trace, warn, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;

use crate::acme::{AcmeClient, AcmeClientError, CustomDomain};
use crate::args::ServiceArgs;
use crate::project::{Project, ProjectCreating, ProjectError, IS_HEALTHY_TIMEOUT};
use crate::task::{self, BoxedTask, TaskBuilder};
use crate::tls::ChainAndPrivateKey;
use crate::worker::TaskRouter;
use crate::{
    DockerContext, DockerStatsSource, ProjectDetails, AUTH_CLIENT, DOCKER_STATS_PATH_CGROUP_V1,
    DOCKER_STATS_PATH_CGROUP_V2,
};

pub static MIGRATIONS: Migrator = sqlx::migrate!("./migrations");
static PROXY_CLIENT: Lazy<ReverseProxy<HttpConnector<GaiResolver>>> =
    Lazy::new(|| ReverseProxy::new(Client::new()));

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Sql(#[from] SqlxError),

    #[error(transparent)]
    ProjectNotReady(#[from] ProjectNotReady),

    #[error(transparent)]
    ProjectUnavailable(#[from] ProjectUnavailable),

    #[error(transparent)]
    ProjectNotFound(#[from] ProjectNotFound),

    /// Contains a message describing a running state of the project.
    /// Used if the project already exists but is owned
    /// by the caller, which means they can modify the project.
    #[error("{0}")]
    OwnProjectAlreadyExists(String),

    #[error("You cannot create more projects. Delete some projects first.")]
    TooManyProjects,

    #[error(transparent)]
    Deprecated(#[from] Deprecated),

    #[error("A project with the same name already exists. Try using a different name.")]
    ProjectAlreadyExists,

    #[error(transparent)]
    Permissions(#[from] permit::Error),

    // Errors that are safe to expose to the user
    #[error("{0}")]
    InternalSafe(String),

    #[error("Custom domain not found")]
    CustomDomainNotFound,

    #[error(transparent)]
    AcmeClient(#[from] AcmeClientError),

    #[error("Our server is at capacity and cannot serve your request at this time. Please try again in a few minutes.")]
    CapacityLimit,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        let status_code = match error {
            Error::Sql(error) => {
                error!(
                    error = &error as &dyn std::error::Error,
                    "internal SQLx error"
                );

                return Self::internal("Internal server error");
            }
            Error::ProjectNotReady(_) => StatusCode::SERVICE_UNAVAILABLE,
            Error::Deprecated(e) => return e.into(),
            Error::ProjectUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Error::ProjectNotFound(e) => return e.into(),
            Error::OwnProjectAlreadyExists(_) => StatusCode::CONFLICT,
            Error::TooManyProjects => StatusCode::PAYMENT_REQUIRED,
            Error::ProjectAlreadyExists => StatusCode::CONFLICT,
            Error::Permissions(error) => {
                error!(
                    error = &error as &dyn std::error::Error,
                    "Failed to check permissions"
                );

                return Self::internal("Internal server error");
            }
            Error::InternalSafe(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::CustomDomainNotFound => StatusCode::NOT_FOUND,
            Error::AcmeClient(e) => return e.into(),
            Error::CapacityLimit => StatusCode::SERVICE_UNAVAILABLE,
        };

        Self {
            message: error.to_string(),
            status_code: status_code.as_u16(),
        }
    }
}

#[derive(Default)]
pub struct ContainerSettingsBuilder {
    prefix: Option<String>,
    image: Option<String>,
    provisioner_uri: Option<String>,
    auth_uri: Option<String>,
    resource_recorder_uri: Option<String>,
    network_name: Option<String>,
    fqdn: Option<String>,
    extra_hosts: Option<Vec<String>>,
}

impl ContainerSettingsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn from_args(self, args: &ServiceArgs) -> ContainerSettings {
        let ServiceArgs {
            prefix,
            network_name,
            provisioner_uri,
            auth_uri,
            resource_recorder_uri,
            image,
            proxy_fqdn,
            extra_hosts,
            ..
        } = args;
        self.prefix(prefix)
            .image(image)
            .provisioner_uri(provisioner_uri)
            .auth_uri(auth_uri)
            .resource_recorder_uri(resource_recorder_uri)
            .network_name(network_name)
            .fqdn(proxy_fqdn)
            .extra_hosts(extra_hosts)
            .build()
            .await
    }

    pub fn prefix<S: ToString>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    pub fn image<S: ToString>(mut self, image: S) -> Self {
        self.image = Some(image.to_string());
        self
    }

    pub fn provisioner_uri<S: ToString>(mut self, provisioner_uri: S) -> Self {
        self.provisioner_uri = Some(provisioner_uri.to_string());
        self
    }

    pub fn auth_uri<S: ToString>(mut self, auth_uri: S) -> Self {
        self.auth_uri = Some(auth_uri.to_string());
        self
    }

    pub fn resource_recorder_uri<S: ToString>(mut self, resource_recorder_uri: S) -> Self {
        self.resource_recorder_uri = Some(resource_recorder_uri.to_string());
        self
    }

    pub fn network_name<S: ToString>(mut self, name: S) -> Self {
        self.network_name = Some(name.to_string());
        self
    }

    pub fn fqdn<S: ToString>(mut self, fqdn: S) -> Self {
        self.fqdn = Some(fqdn.to_string().trim_end_matches('.').to_string());
        self
    }

    pub fn extra_hosts<S: ToString>(mut self, extra_hosts: &[S]) -> Self {
        self.extra_hosts = Some(extra_hosts.iter().map(ToString::to_string).collect());
        self
    }

    pub async fn build(mut self) -> ContainerSettings {
        let prefix = self.prefix.take().unwrap();
        let image = self.image.take().unwrap();
        let provisioner_uri = self.provisioner_uri.take().unwrap();
        let auth_uri = self.auth_uri.take().unwrap();
        let resource_recorder_uri = self.resource_recorder_uri.take().unwrap();
        let extra_hosts = self.extra_hosts.take().unwrap();

        let network_name = self.network_name.take().unwrap();
        let fqdn = self.fqdn.take().unwrap();

        ContainerSettings {
            prefix,
            image,
            provisioner_uri,
            auth_uri,
            resource_recorder_uri,
            network_name,
            fqdn,
            extra_hosts,
        }
    }
}

#[derive(Clone)]
pub struct ContainerSettings {
    pub prefix: String,
    pub image: String,
    pub provisioner_uri: String,
    pub auth_uri: String,
    pub resource_recorder_uri: String,
    pub network_name: String,
    pub fqdn: String,
    pub extra_hosts: Vec<String>,
}

impl ContainerSettings {
    pub fn builder() -> ContainerSettingsBuilder {
        ContainerSettingsBuilder::new()
    }
}

pub struct GatewayService {
    context: GatewayContext,
    db: SqlitePool,
    task_router: TaskRouter,
    pub state_dir: PathBuf,
    pub permit_client: Box<dyn PermissionsDal + Send + Sync>,

    /// Maximum number of containers the gateway can start before blocking cch projects
    cch_container_limit: u32,
    /// Maximum number of containers the gateway can start before blocking non-pro projects
    soft_container_limit: u32,
    /// Maximum number of containers the gateway can start before blocking any project
    hard_container_limit: u32,

    // We store these because we'll need them for the health checks
    provisioner_uri: Endpoint,
    auth_host: Uri,
}

impl GatewayService {
    /// Initialize `GatewayService` and its required dependencies.
    ///
    /// * `args` - The [`Args`] with which the service was
    ///   started. Will be passed as [`Context`] to workers and state.
    pub async fn init(
        args: ServiceArgs,
        db: SqlitePool,
        state_dir: PathBuf,
        permit_client: Box<dyn PermissionsDal + Send + Sync>,
    ) -> io::Result<Self> {
        let docker_stats_path_v1 = PathBuf::from_str(DOCKER_STATS_PATH_CGROUP_V1)
            .expect("to parse docker stats path for cgroup v1");
        let docker_stats_path_v2 = PathBuf::from_str(DOCKER_STATS_PATH_CGROUP_V2)
            .expect("to parse docker stats path for cgroup v2");

        let docker_stats_source = if docker_stats_path_v1.exists() {
            DockerStatsSource::CgroupV1
        } else if docker_stats_path_v2.exists() {
            DockerStatsSource::CgroupV2
        } else {
            DockerStatsSource::Bollard
        };

        info!("docker stats source: {:?}", docker_stats_source.to_string());

        let shuttle_env = std::env::var("SHUTTLE_ENV").unwrap_or("".to_string());
        if (shuttle_env == "staging" || shuttle_env == "production")
            && docker_stats_source == DockerStatsSource::Bollard
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "SHUTTLE_ENV is {} and could not find docker stats at path: {:?} or {:?}",
                    shuttle_env, DOCKER_STATS_PATH_CGROUP_V1, DOCKER_STATS_PATH_CGROUP_V2,
                ),
            ));
        }

        let docker = Docker::connect_with_unix(&args.docker_host, 60, API_DEFAULT_VERSION).unwrap();

        let container_settings = ContainerSettings::builder().from_args(&args).await;

        let provider = GatewayContext {
            docker,
            settings: container_settings,
            gateway_api_key: args.admin_key,
            deploys_api_key: args.deploys_api_key,
            auth_key_uri: format!("{}auth/key", args.auth_uri).parse().unwrap(),
            docker_stats_source,
        };

        let task_router = TaskRouter::default();
        Ok(Self {
            context: provider,
            db,
            task_router,
            state_dir,
            permit_client,
            provisioner_uri: Endpoint::new(args.provisioner_uri)
                .expect("to have a valid provisioner endpoint"),
            auth_host: args.auth_uri,
            cch_container_limit: args.cch_container_limit,
            soft_container_limit: args.soft_container_limit,
            hard_container_limit: args.hard_container_limit,
        })
    }

    pub async fn route(
        &self,
        project: &Project,
        project_name: &ProjectName,
        user_id: &UserId,
        mut req: Request<Body>,
    ) -> Result<Response<Body>, Error> {
        let target_ip = project.target_ip().ok_or(ProjectNotReady)?;

        let target_url = format!("http://{target_ip}:8001");

        debug!(target_url, "routing control");

        let control_key = self.control_key_from_project_name(project_name).await?;

        let headers = req.headers_mut();
        headers.typed_insert(XShuttleAdminSecret(control_key));
        // deprecated, used for soft backward compatibility
        headers.insert("x-shuttle-account-name", user_id.parse().unwrap());

        let cx = Span::current().context();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut HeaderInjector(headers))
        });

        let resp = PROXY_CLIENT
            .call(Ipv4Addr::LOCALHOST.into(), &target_url, req)
            .await
            .map_err(|_| ProjectUnavailable)?;

        Ok(resp)
    }

    pub async fn iter_projects(
        &self,
    ) -> Result<impl ExactSizeIterator<Item = (ProjectName, UserId)>, Error> {
        let iter = query("SELECT project_name, user_id FROM projects")
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|row| (row.get("project_name"), row.get("user_id")));
        Ok(iter)
    }

    /// Only get an iterator for the projects that are ready
    pub async fn iter_projects_ready(
        &self,
    ) -> Result<impl ExactSizeIterator<Item = (ProjectName, UserId)>, Error> {
        let iter = query("SELECT project_name, user_id FROM projects, JSON_EACH(project_state) WHERE key = 'ready'")
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|row| (row.get("project_name"), row.get("user_id")));
        Ok(iter)
    }

    pub async fn iter_cch_projects(
        &self,
    ) -> Result<impl ExactSizeIterator<Item = ProjectName>, Error> {
        let iter = query("SELECT project_name FROM projects WHERE project_name LIKE 'cch23-%'")
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|row| (row.get("project_name")));
        Ok(iter)
    }

    /// The number of projects that are currently in the ready state
    pub async fn count_ready_projects(&self) -> Result<u32, Error> {
        let ready_count: u32 =
            query("SELECT COUNT(*) FROM projects, JSON_EACH(project_state) WHERE key = 'ready'")
                .fetch_one(&self.db)
                .await?
                .get::<_, usize>(0);

        Ok(ready_count)
    }

    /// The number of cch projects that are currently in the ready state
    pub async fn count_ready_cch_projects(&self) -> Result<u32, Error> {
        let ready_count: u32 =
            query("SELECT COUNT(*) FROM projects, JSON_EACH(project_state) WHERE key = 'ready' AND project_name LIKE 'cch23-%'")
                .fetch_one(&self.db)
                .await?
                .get::<_, usize>(0);

        Ok(ready_count)
    }

    pub async fn find_project_by_name(
        &self,
        project_name: &str,
    ) -> Result<FindProjectPayload, Error> {
        query(
            "SELECT project_id, project_name, project_state FROM projects WHERE project_name = ?1",
        )
        .bind(project_name)
        .fetch_optional(&self.db)
        .await?
        .map(|r| FindProjectPayload {
            id: r.get("project_id"),
            name: r.get("project_name"),
            state: r
                .try_get::<SqlxJson<Project>, _>("project_state")
                .map(|p| p.0)
                .unwrap_or_else(|err| {
                    error!(
                        error = &err as &dyn std::error::Error,
                        "Failed to deser `project_state`"
                    );
                    Project::Errored(ProjectError::internal(
                        "Error when trying to deserialize state of project.",
                    ))
                }),
        })
        .ok_or_else(|| ProjectNotFound(project_name.to_string()).into())
    }

    pub async fn find_project_by_id(&self, project_id: &str) -> Result<FindProjectPayload, Error> {
        query("SELECT project_id, project_name, project_state FROM projects WHERE project_id = ?1")
            .bind(project_id)
            .fetch_optional(&self.db)
            .await?
            .map(|r| FindProjectPayload {
                id: r.get("project_id"),
                name: r.get("project_name"),
                state: r
                    .try_get::<SqlxJson<Project>, _>("project_state")
                    .map(|p| p.0)
                    .unwrap_or_else(|err| {
                        error!(
                            error = &err as &dyn std::error::Error,
                            "Failed to deser `project_state`"
                        );
                        Project::Errored(ProjectError::internal(
                            "Error when trying to deserialize state of project.",
                        ))
                    }),
            })
            .ok_or_else(|| ProjectNotFound(project_id.to_string()).into())
    }

    pub async fn project_name_exists(&self, project_name: &ProjectName) -> Result<bool, Error> {
        Ok(
            query("SELECT project_name FROM projects WHERE project_name=?1")
                .bind(project_name)
                .fetch_optional(&self.db)
                .await?
                .is_some(),
        )
    }

    pub async fn iter_user_projects_detailed(
        &self,
        user_id: &UserId,
        offset: u32,
        limit: u32,
    ) -> Result<impl Iterator<Item = (String, ProjectName, Project)>, Error> {
        let mut query = QueryBuilder::new(
            "SELECT project_id, project_name, project_state FROM projects WHERE user_id = ",
        );

        query
            .push_bind(user_id)
            .push(" ORDER BY project_id DESC, project_name LIMIT ")
            .push_bind(limit);

        if offset > 0 {
            query.push(" OFFSET ").push_bind(offset);
        }

        let iter = query
            .build()
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|row| {
                (
                    row.get("project_id"),
                    row.get("project_name"),
                    // This can be invalid JSON if it refers to an outdated Project state
                    row.try_get::<SqlxJson<Project>, _>("project_state")
                        .map(|p| p.0)
                        .unwrap_or_else(|err| {
                            error!(
                                error = &err as &dyn std::error::Error,
                                "Failed to deser `project_state`"
                            );
                            Project::Errored(ProjectError::internal(
                                "Error when trying to deserialize state of project.",
                            ))
                        }),
                )
            });
        Ok(iter)
    }

    pub async fn update_project(
        &self,
        project_name: &ProjectName,
        project: &Project,
    ) -> Result<(), Error> {
        let query = match project {
            Project::Creating(state) => query(
                "UPDATE projects SET initial_key = ?1, project_state = ?2 WHERE project_name = ?3",
            )
            .bind(state.initial_key())
            .bind(SqlxJson(project))
            .bind(project_name),
            _ => query("UPDATE projects SET project_state = ?1 WHERE project_name = ?2")
                .bind(SqlxJson(project))
                .bind(project_name),
        };
        query.execute(&self.db).await?;
        Ok(())
    }

    pub async fn update_project_owner(
        &self,
        project_name: &str,
        new_user_id: &str,
    ) -> Result<(), Error> {
        let mut tr = self.db.begin().await?;
        let (project_id, user_id) = query_as::<_, (String, String)>(
            "SELECT project_id, user_id FROM projects WHERE project_name = ?1",
        )
        .bind(project_name)
        .fetch_one(&mut *tr)
        .await?;
        query("UPDATE projects SET user_id = ?1 WHERE project_name = ?2")
            .bind(new_user_id)
            .bind(project_name)
            .execute(&mut *tr)
            .await?;

        self.permit_client
            .transfer_project_to_user(&user_id, &project_id, new_user_id)
            .await?;

        tr.commit().await?;

        Ok(())
    }

    pub async fn user_id_from_project(&self, project_name: &ProjectName) -> Result<UserId, Error> {
        query("SELECT user_id FROM projects WHERE project_name = ?1")
            .bind(project_name)
            .fetch_optional(&self.db)
            .await?
            .map(|row| row.get("user_id"))
            .ok_or_else(|| ProjectNotFound(project_name.to_string()).into())
    }

    pub async fn control_key_from_project_name(
        &self,
        project_name: &ProjectName,
    ) -> Result<String, Error> {
        let control_key = query("SELECT initial_key FROM projects WHERE project_name = ?1")
            .bind(project_name)
            .fetch_optional(&self.db)
            .await?
            .map(|row| row.try_get("initial_key").unwrap())
            .ok_or_else(|| ProjectNotFound(project_name.to_string()))?;
        Ok(control_key)
    }

    pub async fn iter_user_projects(
        &self,
        user_id: &UserId,
    ) -> Result<impl Iterator<Item = ProjectName>, Error> {
        let iter = query("SELECT project_name FROM projects WHERE user_id = ?1")
            .bind(user_id)
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|row| row.try_get::<ProjectName, _>("project_name").unwrap());
        Ok(iter)
    }

    pub async fn create_project(
        &self,
        project_name: ProjectName,
        user_id: &UserId,
        is_admin: bool,
        can_create_project: bool,
        idle_minutes: u64,
    ) -> Result<FindProjectPayload, Error> {
        if let Some(row) = query(
            r#"
            SELECT project_id, project_state
            FROM projects
            WHERE (project_name = ?1) AND (user_id = ?2 OR ?3)
            "#,
        )
        .bind(&project_name)
        .bind(user_id)
        .bind(is_admin)
        .fetch_optional(&self.db)
        .await?
        {
            // If the project already exists and belongs to this account
            let project = row
                .try_get::<SqlxJson<Project>, _>("project_state")
                .map(|p| p.0)
                .unwrap_or_else(|err| {
                    error!(
                        error = &err as &dyn std::error::Error,
                        "Failed to deser `project_state`"
                    );
                    Project::Errored(ProjectError::internal(
                        "Error when trying to deserialize state of project.",
                    ))
                });
            let project_id = row.get::<String, _>("project_id");
            if project.is_destroyed() {
                // But is in `::Destroyed` state, recreate it
                let creating = ProjectCreating::new_with_random_initial_key(
                    project_name.clone(),
                    Ulid::from_string(project_id.as_str()).map_err(|err| {
                        Error::InternalSafe(format!(
                            "The project id of the destroyed project is not a valid ULID: {err}"
                        ))
                    })?,
                    idle_minutes,
                );
                let project = Project::Creating(creating);
                self.update_project(&project_name, &project).await?;
                Ok(FindProjectPayload {
                    id: project_id,
                    name: project_name.to_string(),
                    state: project,
                })
            } else {
                // Otherwise it already exists. Because the caller of this command is the
                // project owner, this means that the project is already in some running state.
                let state = State::from(project);
                let message = match state {
                    // Ongoing processes.
                    State::Creating { .. }
                    | State::Attaching { .. }
                    | State::Recreating { .. }
                    | State::Starting { .. }
                    | State::Restarting { .. }
                    | State::Stopping
                    | State::Rebooting
                    | State::Destroying => {
                        format!("project '{project_name}' is already {state}. You can check the status again using `cargo shuttle project status`.")
                    }
                    // Use different message than the default for `State::Ready`.
                    State::Ready => {
                        format!("project '{project_name}' is already running")
                    }
                    State::Started | State::Destroyed => {
                        format!("project '{project_name}' is already {state}. Try using `cargo shuttle project restart` instead.")
                    }
                    State::Stopped => {
                        format!("project '{project_name}' is idled. Find out more about idle projects here: {SHUTTLE_IDLE_DOCS_URL}")
                    }
                    State::Errored { message } => {
                        format!("project '{project_name}' is in an errored state.\nproject message: {message}")
                    }
                    State::Deleted => unreachable!(
                        "deleted project should not never remain in gateway. please report this."
                    ),
                };
                Err(Error::OwnProjectAlreadyExists(message))
            }
        } else if can_create_project {
            // Project creation is deprecated on the legacy platform as of the 19th of December
            // 2024.
            Err(Deprecated(
                "New project creation on the shuttle.rs platform has been deprecated.".to_string(),
            )
            .into())
        } else {
            Err(Error::TooManyProjects)
        }
    }

    pub async fn get_project_count(&self, user_id: &UserId) -> Result<u32, Error> {
        let proj_count: u32 = query("SELECT COUNT(project_name) FROM projects WHERE user_id = ?1")
            .bind(user_id)
            .fetch_one(&self.db)
            .await?
            .get::<_, usize>(0);

        Ok(proj_count)
    }

    pub async fn insert_project(
        &self,
        project_name: ProjectName,
        project_id: Ulid,
        user_id: &UserId,
        idle_minutes: u64,
    ) -> Result<FindProjectPayload, Error> {
        let project = SqlxJson(Project::Creating(
            ProjectCreating::new_with_random_initial_key(
                project_name.clone(),
                project_id,
                idle_minutes,
            ),
        ));

        let mut transaction = self.db.begin().await?;
        query("INSERT INTO projects (project_id, project_name, account_name, user_id, initial_key, project_state) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
            .bind(project_id.to_string())
            .bind(&project_name)
            .bind("")
            .bind(user_id)
            .bind(project.initial_key().unwrap())
            .bind(&project)
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                // If the error is a broken PK constraint, this is a
                // project name clash
                if let Some(db_err_code) = err.as_database_error().and_then(DatabaseError::code) {
                    if db_err_code == "2067" {  // SQLITE_CONSTRAINT_UNIQUE
                        return Error::ProjectAlreadyExists
                    }
                }
                // Otherwise this is internal
                err.into()
            })?;

        self.permit_client
            .create_project(user_id, &project_id.to_string())
            .await?;

        transaction.commit().await?;

        let project = project.0;

        Ok(FindProjectPayload {
            id: project_id.to_string(),
            name: project_name.to_string(),
            state: project,
        })
    }

    pub async fn delete_project(&self, project_name: &ProjectName) -> Result<(), Error> {
        let project_id = query("SELECT project_id FROM projects WHERE project_name = ?1")
            .bind(project_name)
            .fetch_one(&self.db)
            .await?
            .get::<String, _>("project_id");

        let mut transaction = self.db.begin().await?;

        query("DELETE FROM custom_domains WHERE project_id = ?1")
            .bind(&project_id)
            .execute(&mut *transaction)
            .await?;

        query("DELETE FROM projects WHERE project_name = ?1")
            .bind(project_name)
            .execute(&mut *transaction)
            .await?;

        self.permit_client.delete_project(&project_id).await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn create_custom_domain(
        &self,
        project_name: &ProjectName,
        fqdn: &Fqdn,
        certs: &str,
        private_key: &str,
    ) -> Result<(), Error> {
        let project_id = query("SELECT project_id FROM projects WHERE project_name = ?1")
            .bind(project_name)
            .fetch_one(&self.db)
            .await?
            .get::<String, _>("project_id");

        query("INSERT OR REPLACE INTO custom_domains (fqdn, project_id, certificate, private_key) VALUES (?1, ?2, ?3, ?4)")
            .bind(fqdn.to_string())
            .bind(project_id)
            .bind(certs)
            .bind(private_key)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    pub async fn iter_custom_domains(&self) -> Result<impl Iterator<Item = CustomDomain>, Error> {
        query("SELECT fqdn, project_name, certificate, private_key FROM custom_domains AS cd JOIN projects AS p ON cd.project_id = p.project_id")
            .fetch_all(&self.db)
            .await
            .map(|res| {
                res.into_iter().map(|row| CustomDomain {
                    fqdn: row.get::<&str, _>("fqdn").parse().unwrap(),
                    project_name: row.try_get("project_name").unwrap(),
                    certificate: row.get("certificate"),
                    private_key: row.get("private_key"),
                })
            })
            .map_err(Error::from)
    }

    pub async fn find_custom_domain_for_project(
        &self,
        project_name: &str,
    ) -> Result<Option<CustomDomain>, Error> {
        let custom_domain = query(
            "SELECT fqdn, project_name, certificate, private_key FROM custom_domains AS cd JOIN projects AS p ON cd.project_id = p.project_id WHERE p.project_name = ?1",
        )
        .bind(project_name)
        .fetch_optional(&self.db)
        .await?
        .map(|row| CustomDomain {
            fqdn: row.get::<&str, _>("fqdn").parse().unwrap(),
            project_name: row.try_get("project_name").unwrap(),
            certificate: row.get("certificate"),
            private_key: row.get("private_key"),
        });

        Ok(custom_domain)
    }

    pub async fn project_details_for_custom_domain(
        &self,
        fqdn: &Fqdn,
    ) -> Result<CustomDomain, Error> {
        let custom_domain = query(
            "SELECT fqdn, project_name, certificate, private_key FROM custom_domains AS cd JOIN projects AS p ON cd.project_id = p.project_id WHERE fqdn = ?1",
        )
        .bind(fqdn.to_string())
        .fetch_optional(&self.db)
        .await?
        .map(|row| CustomDomain {
            fqdn: row.get::<&str, _>("fqdn").parse().unwrap(),
            project_name: row.try_get("project_name").unwrap(),
            certificate: row.get("certificate"),
            private_key: row.get("private_key"),
        })
        .ok_or_else(|| Error::CustomDomainNotFound)?;
        Ok(custom_domain)
    }

    pub async fn iter_projects_detailed(
        &self,
    ) -> Result<impl Iterator<Item = ProjectDetails>, Error> {
        let iter = query("SELECT project_name, account_name, user_id FROM projects")
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|row| ProjectDetails {
                project_name: row.try_get("project_name").unwrap(),
                account_name: row.try_get("account_name").unwrap(),
                user_id: row.try_get("user_id").unwrap(),
            });
        Ok(iter)
    }

    /// Returns the current certificate as a pair of the chain and private key.
    /// If the pair doesn't exist for a specific project, create both the certificate
    /// and the custom domain it will represent.
    pub async fn create_custom_domain_certificate(
        &self,
        fqdn: &Fqdn,
        acme_client: &AcmeClient,
        project_name: &ProjectName,
        creds: AccountCredentials<'_>,
    ) -> Result<(String, String), Error> {
        match self.project_details_for_custom_domain(fqdn).await {
            Ok(CustomDomain {
                certificate,
                private_key,
                ..
            }) => Ok((certificate, private_key)),
            Err(Error::CustomDomainNotFound) => {
                let (certs, private_key) = acme_client
                    .create_certificate(&fqdn.to_string(), ChallengeType::Http01, creds)
                    .await?;
                self.create_custom_domain(project_name, fqdn, &certs, &private_key)
                    .await?;
                Ok((certs, private_key))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn create_certificate<'a>(
        &self,
        acme: &AcmeClient,
        creds: AccountCredentials<'a>,
    ) -> ChainAndPrivateKey {
        let public: FQDN = self.context().settings.fqdn.parse().unwrap();
        let identifier = format!("*.{public}");

        // Use ::Dns01 challenge because that's the only supported
        // challenge type for wildcard domains.
        let (chain, private_key) = acme
            .create_certificate(&identifier, ChallengeType::Dns01, creds)
            .await
            .unwrap();

        let mut buf = Vec::new();
        buf.extend(chain.as_bytes());
        buf.extend(private_key.as_bytes());

        ChainAndPrivateKey::parse_pem(Cursor::new(buf)).expect("Malformed PEM buffer.")
    }

    /// Fetch the gateway certificate from the state location.
    /// If not existent, create the gateway certificate and save it to the
    /// gateway state.
    pub async fn fetch_certificate(
        &self,
        acme: &AcmeClient,
        creds: AccountCredentials<'_>,
    ) -> ChainAndPrivateKey {
        let tls_path = self.state_dir.join("ssl.pem");
        match ChainAndPrivateKey::load_pem(&tls_path) {
            Ok(valid) => valid,
            Err(_) => {
                warn!(
                    "no valid certificate found at {}, creating one...",
                    tls_path.display()
                );

                let certs = self.create_certificate(acme, creds).await;
                certs.clone().save_pem(&tls_path).unwrap();
                certs
            }
        }
    }

    pub fn context(&self) -> &GatewayContext {
        &self.context
    }

    /// Create a builder for a new [ProjectTask]
    pub fn new_task(self: &Arc<Self>) -> TaskBuilder {
        TaskBuilder::new(
            self.clone(),
            Span::current().metadata().map(|m| m.name().to_string()),
        )
    }

    /// Find a project by name. And start the project if it is idle, waiting for it to start up
    pub async fn find_or_start_project(
        self: &Arc<Self>,
        project_name: &ProjectName,
        task_sender: Sender<BoxedTask>,
    ) -> Result<(FindProjectPayload, bool), Error> {
        let mut project = self.find_project_by_name(project_name).await?;

        // Start the project if it is idle
        let is_stopped = project.state.is_stopped();
        if is_stopped {
            trace!(shuttle.project.name = %project_name, "starting up idle project");

            let handle = self
                .new_task()
                .project(project_name.clone())
                .and_then(task::start())
                .and_then(task::run_until_done())
                .and_then(task::start_idle_deploys())
                .send(&task_sender)
                .await
                .map_err(|task_error| Error::InternalSafe(task_error.to_string()))?;

            // Wait for project to come up and set new state
            handle.await;
            project = self.find_project_by_name(project_name).await?;
        }

        Ok((project, is_stopped))
    }

    /// Get project id of a project, by name.
    pub async fn project_id(self: &Arc<Self>, project_name: &ProjectName) -> Result<String, Error> {
        Ok(
            query("SELECT project_id FROM projects WHERE project_name = ?1")
                .bind(project_name)
                .fetch_one(&self.db)
                .await?
                .get::<String, _>("project_id"),
        )
    }

    pub fn task_router(&self) -> TaskRouter {
        self.task_router.clone()
    }

    pub fn credentials(&self) -> AccountCredentials<'_> {
        let creds_path = self.state_dir.join("acme.json");
        if !creds_path.exists() {
            panic!(
                "no ACME credentials found at {}, cannot continue with certificate creation",
                creds_path.display()
            );
        }

        serde_json::from_reader(std::fs::File::open(creds_path).expect("Invalid credentials path"))
            .expect("Can not parse admin credentials from path")
    }

    pub fn provisioner_uri(&self) -> &Endpoint {
        &self.provisioner_uri
    }
    pub fn auth_uri(&self) -> &Uri {
        &self.auth_host
    }

    /// Is there enough capacity to start this project
    ///
    /// There is capacity if we are below the cch limit.
    /// Else free and pro tier projects below the soft limit is allowed.
    /// But only pro tier projects are allowed between the soft and hard limit.
    /// Nothing should be allowed above the hard limits so that our own services don't crash.
    pub async fn has_capacity(
        &self,
        is_cch_project: bool,
        account_tier: &AccountTier,
    ) -> Result<(), Error> {
        // If this control file exists, block routing to cch23 projects.
        // Used for emergency load mitigation
        const CCH_CONTROL_FILE: &str = "/var/lib/shuttle/BLOCK_CCH23_PROJECT_TRAFFIC";
        const CCH_CONCURRENT_LIMIT: u32 = 20;

        if is_cch_project
            && (std::fs::metadata(CCH_CONTROL_FILE).is_ok()
                || self.count_ready_cch_projects().await? >= CCH_CONCURRENT_LIMIT)
        {
            return Err(Error::CapacityLimit);
        }

        let current_container_count = self.count_ready_projects().await?;
        let has_capacity = if current_container_count < self.cch_container_limit {
            true
        } else if current_container_count < self.soft_container_limit {
            !is_cch_project
        } else if current_container_count < self.hard_container_limit {
            matches!(account_tier, AccountTier::Pro | AccountTier::Employee)
        } else {
            false
        };

        if has_capacity {
            Ok(())
        } else {
            Err(Error::CapacityLimit)
        }
    }
}

#[derive(Clone)]
pub struct GatewayContext {
    docker: Docker,
    settings: ContainerSettings,
    gateway_api_key: String,
    deploys_api_key: String,
    auth_key_uri: Uri,
    docker_stats_source: DockerStatsSource,
}

#[async_trait]
impl DockerContext for GatewayContext {
    fn docker(&self) -> &Docker {
        &self.docker
    }

    fn container_settings(&self) -> &ContainerSettings {
        &self.settings
    }

    #[instrument(name = "get container stats", skip_all, fields(docker_stats_source = %self.docker_stats_source, shuttle.container.id = container_id))]
    async fn get_stats(&self, container_id: &str) -> Result<u64, String> {
        match self.docker_stats_source {
            DockerStatsSource::CgroupV1 => {
                let cpu_usage: u64 = tokio::fs::read_to_string(format!(
                    "{DOCKER_STATS_PATH_CGROUP_V1}/{container_id}/cpuacct.usage"
                ))
                .await
                .map_err(|err| {
                    error!(
                        error = &err as &dyn std::error::Error,
                        shuttle.container.id = container_id,
                        "failed to read docker stats file for container"
                    );
                    "failed to read docker stats file for container".to_string()
                })?
                .trim()
                .parse()
                .map_err(|err| {
                    error!(
                        error = &err as &dyn std::error::Error,
                        shuttle.container.id = container_id,
                        "failed to parse cpu usage stat"
                    );

                    "failed to parse cpu usage to u64".to_string()
                })?;
                Ok(cpu_usage)
            }
            DockerStatsSource::CgroupV2 => {
                // 'usage_usec' is on the first line and the needed stat
                let cpu_usage: u64 = tokio::fs::read_to_string(format!(
                    "{DOCKER_STATS_PATH_CGROUP_V2}/docker-{container_id}.scope/cpu.stat"
                ))
                .await
                .map_err(|err| {
                    error!(
                        error = &err as &dyn std::error::Error,
                        shuttle.container.id = container_id,
                        "failed to read docker stats file for container"
                    );
                    "failed to read docker stats file for container".to_string()
                })?
                .lines()
                .next()
                .ok_or_else(|| {
                    let err =
                        "failed to read first line of docker stats file for container".to_string();
                    error!(shuttle.container.id = container_id, error = &err);

                    err
                })?
                .split(' ')
                .nth(1)
                .ok_or_else(|| {
                    let err = "failed to split docker stats line for container".to_string();
                    error!(shuttle.container.id = container_id, error = &err);

                    err
                })?
                .parse::<u64>()
                .map_err(|err| {
                    error!(
                        error = &err as &dyn std::error::Error,
                        shuttle.container.id = container_id,
                        "failed to parse cpu usage stat"
                    );
                    "failed to parse cpu usage to u64".to_string()
                })?;
                Ok(cpu_usage * 1_000)
            }
            DockerStatsSource::Bollard => {
                let new_stat = self
                    .docker()
                    .stats(
                        container_id,
                        Some(StatsOptions {
                            one_shot: true,
                            stream: false,
                        }),
                    )
                    .next()
                    .await
                    .ok_or_else(|| {
                        error!(
                            shuttle.container.id = container_id,
                            "there was no stats for the container"
                        );
                        "there was no stats for the container".to_string()
                    })?
                    .map_err(|error| {
                        error!(
                            shuttle.container.id = container_id,
                            error = %error,
                            "failed to get container stats from bollard"
                        );
                        "failed to get stats for container".to_string()
                    })?;

                Ok(new_stat.cpu_stats.cpu_usage.total_usage)
            }
        }
    }
}

impl GatewayContext {
    #[instrument(skip(self), fields(auth_key_uri = %self.auth_key_uri))]
    pub async fn get_jwt(&self) -> String {
        let mut req = Request::builder().uri(self.auth_key_uri.clone());

        let headers = req
            .headers_mut()
            .expect("to get headers on manually created request");

        headers.typed_insert(
            Authorization::bearer(&self.deploys_api_key).expect("to build an authorization bearer"),
        );
        headers.typed_insert(XShuttleAdminSecret(self.gateway_api_key.clone()));

        let req = req.body(Body::empty()).unwrap();

        trace!("getting jwt");

        let resp = timeout(IS_HEALTHY_TIMEOUT, AUTH_CLIENT.request(req)).await;

        if let Ok(Ok(resp)) = resp {
            let body = hyper::body::to_bytes(resp.into_body())
                .await
                .unwrap_or_default();
            let convert: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();

            trace!(?convert, "got jwt response");

            convert["token"].as_str().unwrap_or_default().to_string()
        } else {
            error!("was not able to get JWT for gateway");
            Default::default()
        }
    }
}

pub struct FindProjectPayload {
    pub id: String,
    pub name: String,
    pub state: Project,
}

#[cfg(test)]
pub mod tests {
    use fqdn::FQDN;
    use shuttle_backends::test_utils::gateway::PermissionsMock;

    use super::*;

    use crate::task::{self, TaskResult};
    use crate::tests::World;

    #[tokio::test]
    async fn service_create_find_stop_delete_project() {
        let world = World::new().await;
        let svc = Arc::new(
            GatewayService::init(
                world.args(),
                world.pool(),
                "".into(),
                Box::<PermissionsMock>::default(),
            )
            .await
            .unwrap(),
        );

        let neo: UserId = "neo".to_owned();
        let trinity: UserId = "trinity".to_owned();
        let matrix: ProjectName = "matrix".parse().unwrap();

        let admin: UserId = "admin".to_owned();

        let creating_same_project_name = |project: &Project, project_name: &ProjectName| {
            matches!(
                project,
                Project::Creating(creating) if creating.project_name() == project_name
            )
        };

        let project = svc
            .create_project(matrix.clone(), &neo, false, true, 0)
            .await
            .unwrap();

        assert!(creating_same_project_name(&project.state, &matrix));

        assert_eq!(
            svc.find_project_by_name(&matrix).await.unwrap().state,
            project.state
        );
        assert_eq!(
            svc.iter_projects_detailed()
                .await
                .unwrap()
                .next()
                .expect("to get one project with its user"),
            ProjectDetails {
                project_name: matrix.clone(),
                account_name: Some("".to_owned()),
                user_id: neo.clone(),
            }
        );
        assert_eq!(
            svc.iter_user_projects_detailed(&neo, 0, u32::MAX)
                .await
                .unwrap()
                .map(|item| item.1)
                .collect::<Vec<_>>(),
            vec![matrix.clone()]
        );

        // Test project pagination, first create 20 projects.
        for p in (0..20).map(|p| format!("matrix-{p}")) {
            svc.create_project(p.parse().unwrap(), &admin, true, true, 0)
                .await
                .unwrap();
        }

        // Creating a project with can_create_project set to false should fail.
        assert!(matches!(
            svc.create_project("final-one".parse().unwrap(), &admin, false, false, 0)
                .await
                .err()
                .unwrap(),
            Error::TooManyProjects
        ));

        // We need to fetch all of them from the DB since they are ordered by created_at (in the id) and project_name,
        // and created_at will be the same for some of them.
        let all_projects = svc
            .iter_user_projects_detailed(&admin, 0, u32::MAX)
            .await
            .unwrap()
            .map(|item| item.0)
            .collect::<Vec<_>>();

        assert_eq!(all_projects.len(), 20);

        // Get first 5 projects.
        let paginated = svc
            .iter_user_projects_detailed(&admin, 0, 5)
            .await
            .unwrap()
            .map(|item| item.0)
            .collect::<Vec<_>>();

        assert_eq!(all_projects[..5], paginated);

        // Get 10 projects starting at an offset of 10.
        let paginated = svc
            .iter_user_projects_detailed(&admin, 10, 10)
            .await
            .unwrap()
            .map(|item| item.0)
            .collect::<Vec<_>>();
        assert_eq!(all_projects[10..20], paginated);

        // Get 20 projects starting at an offset of 200.
        let paginated = svc
            .iter_user_projects_detailed(&admin, 200, 20)
            .await
            .unwrap()
            .collect::<Vec<_>>();

        assert!(paginated.is_empty());

        let mut work = svc
            .new_task()
            .project(matrix.clone())
            .and_then(task::destroy())
            .build();

        while let TaskResult::Pending(_) = work.poll(()).await {}
        assert!(matches!(work.poll(()).await, TaskResult::Done(())));

        // After project has been destroyed...
        assert!(matches!(
            svc.find_project_by_name(&matrix).await.unwrap().state,
            Project::Destroyed(_)
        ));

        // If recreated by a different user
        assert!(matches!(
            svc.create_project(matrix.clone(), &trinity, false, true, 0)
                .await,
            Err(Error::ProjectAlreadyExists)
        ));

        // If recreated by the same user
        assert!(matches!(
            svc.create_project(matrix.clone(), &neo, false, true, 0)
                .await
                .unwrap()
                .state,
            Project::Creating(_),
        ));

        // If recreated by the same user again while it's running
        assert!(matches!(
            svc.create_project(matrix.clone(), &neo, false, true, 0)
                .await,
            Err(Error::OwnProjectAlreadyExists(_))
        ));

        let mut work = svc
            .new_task()
            .project(matrix.clone())
            .and_then(task::destroy())
            .build();

        while let TaskResult::Pending(_) = work.poll(()).await {}
        assert!(matches!(work.poll(()).await, TaskResult::Done(())));

        // After project has been destroyed again...
        assert!(matches!(
            svc.find_project_by_name(&matrix).await.unwrap().state,
            Project::Destroyed(_),
        ));

        // If recreated by an admin
        assert!(matches!(
            svc.create_project(matrix.clone(), &admin, true, true, 0)
                .await
                .unwrap()
                .state,
            Project::Creating(_)
        ));

        // If recreated by an admin again while it's running
        assert!(matches!(
            svc.create_project(matrix.clone(), &admin, true, true, 0)
                .await,
            Err(Error::OwnProjectAlreadyExists(_))
        ));

        // We can delete a project
        assert!(matches!(svc.delete_project(&matrix).await, Ok(())));

        // Project is gone
        assert!(matches!(
            svc.find_project_by_name(&matrix).await,
            Err(Error::ProjectNotFound(_))
        ));

        // It can be re-created by anyone, with the same project name
        assert!(matches!(
            svc.create_project(matrix, &trinity, false, true, 0)
                .await
                .unwrap()
                .state,
            Project::Creating(_)
        ));
    }

    #[tokio::test]
    async fn service_create_ready_kill_restart_docker() {
        let world = World::new().await;
        let svc = Arc::new(
            GatewayService::init(
                world.args(),
                world.pool(),
                "".into(),
                Box::<PermissionsMock>::default(),
            )
            .await
            .unwrap(),
        );

        let neo: UserId = "neo".to_owned();
        let matrix: ProjectName = "matrix".parse().unwrap();

        svc.create_project(matrix.clone(), &neo, false, true, 0)
            .await
            .unwrap();

        let mut task = svc.new_task().project(matrix.clone()).build();

        while let TaskResult::Pending(_) = task.poll(()).await {
            // keep polling
        }

        let project = svc.find_project_by_name(&matrix).await.unwrap();
        println!("{:?}", project.state);
        assert!(project.state.is_ready());

        let container = project.state.container().unwrap();
        svc.context()
            .docker()
            .kill_container::<String>(container.name.unwrap().strip_prefix('/').unwrap(), None)
            .await
            .unwrap();

        println!("killed container");

        let mut ambulance_task = svc.new_task().project(matrix.clone()).build();

        // the first poll will trigger a refresh
        let _ = ambulance_task.poll(()).await;

        let project = svc.find_project_by_name(&matrix).await.unwrap();
        println!("{:?}", project.state);
        assert!(!project.state.is_ready());

        // the subsequent will trigger a restart task
        while let TaskResult::Pending(_) = ambulance_task.poll(()).await {
            // keep polling
        }

        let project = svc.find_project_by_name(&matrix).await.unwrap();
        println!("{:?}", project.state);
        assert!(project.state.is_ready());
    }

    #[tokio::test]
    async fn service_create_find_custom_domain() {
        let world = World::new().await;
        let svc = Arc::new(
            GatewayService::init(
                world.args(),
                world.pool(),
                "".into(),
                Box::<PermissionsMock>::default(),
            )
            .await
            .unwrap(),
        );

        let account: UserId = "neo".to_owned();
        let project_name: ProjectName = "matrix".parse().unwrap();
        let domain: FQDN = "neo.the.matrix".parse().unwrap();
        let certificate = "dummy certificate";
        let private_key = "dummy private key";

        assert!(matches!(
            svc.project_details_for_custom_domain(&domain)
                .await
                .unwrap_err(),
            Error::CustomDomainNotFound
        ));

        let _ = svc
            .create_project(project_name.clone(), &account, false, true, 0)
            .await
            .unwrap();

        svc.create_custom_domain(&project_name, &domain, certificate, private_key)
            .await
            .unwrap();

        let custom_domain = svc
            .project_details_for_custom_domain(&domain)
            .await
            .unwrap();

        assert_eq!(custom_domain.project_name, project_name);
        assert_eq!(custom_domain.certificate, certificate);
        assert_eq!(custom_domain.private_key, private_key);

        // Should auto replace the domain details
        let certificate = "dummy certificate update";
        let private_key = "dummy private key update";

        svc.create_custom_domain(&project_name, &domain, certificate, private_key)
            .await
            .unwrap();

        let custom_domain = svc
            .project_details_for_custom_domain(&domain)
            .await
            .unwrap();

        assert_eq!(custom_domain.project_name, project_name);
        assert_eq!(custom_domain.certificate, certificate);
        assert_eq!(custom_domain.private_key, private_key);
    }

    #[tokio::test]
    async fn service_create_custom_domain_destroy_recreate_project() {
        let world = World::new().await;
        let svc = Arc::new(
            GatewayService::init(
                world.args(),
                world.pool(),
                "".into(),
                Box::<PermissionsMock>::default(),
            )
            .await
            .unwrap(),
        );

        let account: UserId = "neo".to_owned();
        let project_name: ProjectName = "matrix".parse().unwrap();
        let domain: FQDN = "neo.the.matrix".parse().unwrap();
        let certificate = "dummy certificate";
        let private_key = "dummy private key";

        assert!(matches!(
            svc.project_details_for_custom_domain(&domain)
                .await
                .unwrap_err(),
            Error::CustomDomainNotFound
        ));

        let _ = svc
            .create_project(project_name.clone(), &account, false, true, 0)
            .await
            .unwrap();

        svc.create_custom_domain(&project_name, &domain, certificate, private_key)
            .await
            .unwrap();

        let mut work = svc
            .new_task()
            .project(project_name.clone())
            .and_then(task::destroy())
            .build();

        while let TaskResult::Pending(_) = work.poll(()).await {}
        assert!(matches!(work.poll(()).await, TaskResult::Done(())));

        let recreated_project = svc
            .create_project(project_name.clone(), &account, false, true, 0)
            .await
            .unwrap();

        assert!(matches!(recreated_project.state, Project::Creating(_)));
    }
}
