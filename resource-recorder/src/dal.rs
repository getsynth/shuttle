use std::{fmt, path::Path, str::FromStr, time::SystemTime};

use crate::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use shuttle_common::resource::Type;
use shuttle_proto::resource_recorder::{self, record_request};
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteRow},
    FromRow, Row, SqlitePool,
};
use tracing::{error, info};
use ulid::Ulid;

pub static MIGRATIONS: Migrator = sqlx::migrate!("./migrations");

#[derive(Error, Debug)]
pub enum DalError {
    Sqlx(#[from] sqlx::Error),
    ProjectId,
    ServiceId,
    Inactive,
}

// We are not using the `thiserror`'s `#[error]` syntax to prevent sensitive details from bubbling up to the users.
// Instead we are logging it as an error which we can inspect.
impl fmt::Display for DalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            DalError::Sqlx(error) => {
                error!(
                    error = error as &dyn std::error::Error,
                    "database request failed"
                );

                "failed to interact with recorder"
            }
            DalError::ProjectId => "resource belongs to a different project",
            DalError::ServiceId => "resource belongs to a different service",
            DalError::Inactive => "cannot add a resource that is inactive",
        };

        write!(f, "{msg}")
    }
}

#[async_trait]
pub trait Dal {
    /// Add a set of resources for a service
    async fn add_resources(
        &self,
        project_id: Ulid,
        service_id: Ulid,
        resources: Vec<Resource>,
    ) -> Result<(), DalError>;

    /// Get the resources that belong to a project
    async fn get_project_resources(&self, project_id: Ulid) -> Result<Vec<Resource>, DalError>;

    /// Get a resource
    async fn get_resource(
        &self,
        resource: resource_recorder::ResourceIds,
    ) -> Result<Option<Resource>, DalError>;

    /// Delete a resource
    async fn delete_resource(
        &self,
        resource: resource_recorder::ResourceIds,
    ) -> Result<(), DalError>;
}

pub struct Sqlite {
    pool: SqlitePool,
}

impl Sqlite {
    /// This function creates all necessary tables and sets up a database connection pool.
    pub async fn new(path: &str) -> Self {
        if !Path::new(path).exists() {
            sqlx::Sqlite::create_database(path).await.unwrap();
        }

        info!(
            "state db: {}",
            std::fs::canonicalize(path).unwrap().to_string_lossy()
        );

        // We have found in the past that setting synchronous to anything other than the default (full) breaks the
        // broadcast channel in deployer. The broken symptoms are that the ws socket connections won't get any logs
        // from the broadcast channel and would then close. When users did deploys, this would make it seem like the
        // deploy is done (while it is still building for most of the time) and the status of the previous deployment
        // would be returned to the user.
        //
        // If you want to activate a faster synchronous mode, then also do proper testing to confirm this bug is no
        // longer present.
        let sqlite_options = SqliteConnectOptions::from_str(path)
            .unwrap()
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePool::connect_with(sqlite_options).await.unwrap();

        Self::from_pool(pool).await
    }

    #[allow(dead_code)]
    pub async fn new_in_memory() -> Self {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        Self::from_pool(pool).await
    }

    async fn from_pool(pool: SqlitePool) -> Self {
        MIGRATIONS.run(&pool).await.unwrap();

        Self { pool }
    }
}

#[async_trait]
impl Dal for Sqlite {
    async fn add_resources(
        &self,
        project_id: Ulid,
        service_id: Ulid,
        resources: Vec<Resource>,
    ) -> Result<(), DalError> {
        let mut transaction = self.pool.begin().await?;
        let now = Utc::now();

        sqlx::query("UPDATE resources SET is_active = false WHERE service_id = ?")
            .bind(service_id.to_string())
            .execute(&mut *transaction)
            .await?;

        // Making multiple DB "connections" is fine since the sqlite is on the same machine
        for resource in resources {
            if let Some(r_project_id) = resource.project_id {
                if r_project_id != project_id {
                    return Err(DalError::ProjectId);
                }
            }

            if let Some(r_service_id) = resource.service_id {
                if r_service_id != service_id {
                    return Err(DalError::ServiceId);
                }
            }

            if !resource.is_active {
                return Err(DalError::Inactive);
            }

            sqlx::query("INSERT OR REPLACE INTO resources (project_id, service_id, type, config, data, is_active, last_updated) VALUES(?, ?, ?, ?, ?, ?, ?)")
            .bind(project_id.to_string())
            .bind(service_id.to_string())
            .bind(resource.r#type)
            .bind(resource.config)
            .bind(resource.data)
            .bind(resource.is_active)
            .bind(now)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn get_project_resources(&self, project_id: Ulid) -> Result<Vec<Resource>, DalError> {
        let result = sqlx::query_as(r#"SELECT * FROM resources WHERE project_id = ?"#)
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        Ok(result)
    }

    async fn get_resource(
        &self,
        resource: resource_recorder::ResourceIds,
    ) -> Result<Option<Resource>, DalError> {
        let result = sqlx::query_as(
            "SELECT * FROM resources WHERE project_id = ? AND service_id = ? AND type = ?",
        )
        .bind(resource.project_id)
        .bind(resource.service_id)
        .bind(resource.r#type)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn delete_resource(
        &self,
        resource: resource_recorder::ResourceIds,
    ) -> Result<(), DalError> {
        sqlx::query("DELETE FROM resources WHERE project_id = ? AND service_id = ? AND type = ?")
            .bind(resource.project_id)
            .bind(resource.service_id)
            .bind(resource.r#type)
            .execute(&self.pool)
            .await
            .map(|_| ())?;

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resource {
    project_id: Option<Ulid>,
    service_id: Option<Ulid>,
    r#type: Type,
    data: Vec<u8>,
    config: Vec<u8>,
    is_active: bool,
    created_at: DateTime<Utc>,
    last_updated: DateTime<Utc>,
}

impl FromRow<'_, SqliteRow> for Resource {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            project_id: Some(
                Ulid::from_string(row.try_get("project_id")?)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            ),
            service_id: Some(
                Ulid::from_string(row.try_get("service_id")?)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            ),
            r#type: row.try_get("type")?,
            data: row.try_get("data")?,
            config: row.try_get("config")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            last_updated: row.try_get("last_updated")?,
        })
    }
}

impl TryFrom<record_request::Resource> for Resource {
    type Error = String;

    fn try_from(value: record_request::Resource) -> Result<Self, Self::Error> {
        let r#type = value.r#type.parse()?;
        Ok(Self::new(r#type, value.data, value.config))
    }
}

impl From<Resource> for resource_recorder::Resource {
    fn from(value: Resource) -> Self {
        Self {
            project_id: value
                .project_id
                .expect("row to have a project id")
                .to_string(),
            service_id: value
                .service_id
                .expect("row to have a service id")
                .to_string(),
            r#type: value.r#type.to_string(),
            config: value.config,
            data: value.data,
            is_active: value.is_active,
            created_at: Some(Timestamp::from(SystemTime::from(value.created_at))),
            last_updated: Some(Timestamp::from(SystemTime::from(value.last_updated))),
        }
    }
}

impl TryFrom<resource_recorder::Resource> for Resource {
    type Error = Error;

    fn try_from(value: resource_recorder::Resource) -> Result<Self, Self::Error> {
        Ok(Self {
            project_id: Some(value.project_id.parse()?),
            service_id: Some(value.service_id.parse()?),
            r#type: value.r#type.parse()?,
            data: value.data,
            config: value.config,
            is_active: value.is_active,
            created_at: DateTime::from(SystemTime::try_from(value.created_at.unwrap_or_default())?),
            last_updated: DateTime::from(SystemTime::try_from(
                value.last_updated.unwrap_or_default(),
            )?),
        })
    }
}

impl Resource {
    /// Create a new type of resource
    fn new(r#type: Type, data: Vec<u8>, config: Vec<u8>) -> Self {
        Self {
            project_id: None,
            service_id: None,
            r#type,
            data,
            config,
            is_active: true,
            created_at: Default::default(),
            last_updated: Default::default(),
        }
    }
}
