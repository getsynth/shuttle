use crate::database;
use async_trait::async_trait;
use lib::ProjectConfig;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tokio::sync::Mutex;
use unveil_service::Factory;

pub(crate) struct UnveilFactory {
    database: Arc<Mutex<database::State>>,
    project: ProjectConfig,
    ctx: database::Context,
}

impl UnveilFactory {
    pub(crate) fn new(
        database: Arc<Mutex<database::State>>,
        project: ProjectConfig,
        ctx: database::Context,
    ) -> Self {
        Self {
            database,
            project,
            ctx,
        }
    }
}

#[async_trait]
impl Factory for UnveilFactory {
    /// Lazily gets a connection pool
    async fn get_postgres_connection_pool(&mut self) -> Result<PgPool, unveil_service::Error> {
        let ready_state = self
            .database
            .lock()
            .await
            .advance(&self.project.name, &self.ctx)
            .await
            .map_err(unveil_service::Error::from)?;

        PgPoolOptions::new()
            .max_connections(10)
            .connect(&ready_state.connection_string())
            .await
            .map_err(unveil_service::Error::from)
    }
}
