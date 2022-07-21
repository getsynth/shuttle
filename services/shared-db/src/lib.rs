#![doc = include_str!("../README.md")]

use tokio::runtime::Runtime;

use async_trait::async_trait;
use shuttle_service::{database, error::CustomError, Error, Factory, ResourceBuilder};

#[cfg(feature = "postgres")]
pub struct Postgres;

#[cfg(feature = "postgres")]
/// Get an `sqlx::PgPool` from any factory
#[async_trait]
impl ResourceBuilder<sqlx::PgPool> for Postgres {
    fn new() -> Self {
        Self {}
    }

    async fn build(
        self,
        factory: &mut dyn Factory,
        runtime: &Runtime,
    ) -> Result<sqlx::PgPool, Error> {
        let connection_string = factory
            .get_sql_connection_string(database::Type::Shared)
            .await?;

        // A sqlx Pool cannot cross runtime boundaries, so make sure to create the Pool on the service end
        let pool = runtime
            .spawn(async move {
                sqlx::postgres::PgPoolOptions::new()
                    .min_connections(1)
                    .max_connections(5)
                    .connect(&connection_string)
                    .await
            })
            .await
            .map_err(CustomError::new)?
            .map_err(CustomError::new)?;

        Ok(pool)
    }
}
