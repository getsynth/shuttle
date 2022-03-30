pub mod project;

use crate::project::ProjectName;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use rocket::Responder;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

extern crate lazy_static;

pub const SHUTTLE_PROJECT_HEADER: &str = "Shuttle-Project";

#[cfg(debug_assertions)]
lazy_static! {
    pub static ref API_URL: String =
        std::env::var("SHUTTLE_API").unwrap_or_else(|_| "http://localhost:8001".to_string());
}

#[cfg(not(debug_assertions))]
lazy_static! {
    pub static ref API_URL: String = "https://api.shuttle.rs".to_string();
}

pub type ApiKey = String;
pub type Host = String;
pub type DeploymentId = Uuid;
pub type Port = u16;

/// Deployment metadata. This serves two purposes. Storing information
/// used for the deployment process and also providing the client with
/// information on the state of the deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMeta {
    pub id: DeploymentId,
    pub project: ProjectName,
    pub state: DeploymentStateMeta,
    pub host: String,
    pub build_logs: Option<String>,
    pub runtime_logs: Option<String>,
    pub database_deployment: Option<DatabaseReadyInfo>,
    pub created_at: DateTime<Utc>,
}

impl DeploymentMeta {
    pub fn queued(project: ProjectName) -> Self {
        Self::new(project, DeploymentStateMeta::Queued)
    }

    pub fn built(project: ProjectName) -> Self {
        Self::new(project, DeploymentStateMeta::Built)
    }

    fn new(project: ProjectName, state: DeploymentStateMeta) -> Self {
        let host = Self::create_host(&project);
        Self {
            id: Uuid::new_v4(),
            project,
            state,
            host,
            build_logs: None,
            runtime_logs: None,
            database_deployment: None,
            created_at: Utc::now(),
        }
    }

    pub fn create_host(project_name: &ProjectName) -> Host {
        format!("{}.shuttleapp.rs", project_name)
    }
}

#[cfg(debug_assertions)]
const PUBLIC_IP: &str = "localhost";

#[cfg(not(debug_assertions))]
const PUBLIC_IP: &'static str = "pg.shuttle.rs";

impl Display for DeploymentMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let db = {
            if let Some(info) = &self.database_deployment {
                format!(
                    "\n        Database URI:       {}",
                    info.connection_string(PUBLIC_IP)
                )
            } else {
                "".to_string()
            }
        };
        write!(
            f,
            r#"
        Project:            {}
        Deployment Id:      {}
        Deployment Status:  {}
        Host:               {}
        Created At:         {}{}
        "#,
            self.project, self.id, self.state, self.host, self.created_at, db
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseReadyInfo {
    pub role_name: String,
    pub role_password: String,
    pub database_name: String,
}

impl DatabaseReadyInfo {
    pub fn new(role_name: String, role_password: String, database_name: String) -> Self {
        Self {
            role_name,
            role_password,
            database_name,
        }
    }
    pub fn connection_string(&self, ip: &str) -> String {
        format!(
            "postgres://{}:{}@{}/{}",
            self.role_name, self.role_password, ip, self.database_name
        )
    }
}

/// A label used to represent the deployment state in `DeploymentMeta`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStateMeta {
    Queued,
    Built,
    Loaded,
    Deployed,
    Error(String),
    Deleted,
}

impl Display for DeploymentStateMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DeploymentStateMeta::Queued => "QUEUED".to_string(),
            DeploymentStateMeta::Built => "BUILT".to_string(),
            DeploymentStateMeta::Loaded => "LOADED".to_string(),
            DeploymentStateMeta::Deployed => "DEPLOYED".to_string(),
            DeploymentStateMeta::Error(msg) => format!("ERROR: {}", &msg),
            DeploymentStateMeta::Deleted => "DELETED".to_string(),
        };
        write!(f, "{}", s)
    }
}

// TODO: Determine error handling strategy - error types or just use `anyhow`?
#[derive(Debug, Clone, Serialize, Deserialize, Responder)]
#[response(content_type = "json")]
pub enum DeploymentApiError {
    #[response(status = 500)]
    Internal(String),
    #[response(status = 503)]
    Unavailable(String),
    #[response(status = 404)]
    NotFound(String),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 409)]
    ProjectAlreadyExists(String),
}

impl Display for DeploymentApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentApiError::Internal(s) => write!(f, "internal: {}", s),
            DeploymentApiError::Unavailable(s) => write!(f, "unavailable: {}", s),
            DeploymentApiError::NotFound(s) => write!(f, "not found: {}", s),
            DeploymentApiError::BadRequest(s) => write!(f, "bad request: {}", s),
            DeploymentApiError::ProjectAlreadyExists(s) => write!(f, "conflict: {}", s),
        }
    }
}

impl std::error::Error for DeploymentApiError {}
