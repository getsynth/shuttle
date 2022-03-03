use crate::Error;
use async_trait::async_trait;
use sqlx::pool::PoolConnection;

#[async_trait]
pub trait Factory: Send + Sync {
    async fn get_postgres_connection_pool(
        &mut self,
    ) -> Result<PoolConnection<sqlx::Postgres>, Error>;
}

#[async_trait]
impl Factory for Box<dyn Factory> {
    async fn get_postgres_connection_pool(
        &mut self,
    ) -> Result<PoolConnection<sqlx::Postgres>, Error> {
        self.as_mut().get_postgres_connection_pool().await
    }
}
