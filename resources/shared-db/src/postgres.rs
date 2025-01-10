use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shuttle_service::{
    database,
    resource::{ProvisionResourceRequest, ShuttleResourceOutput, Type},
    DatabaseResource, DbInput, Environment, Error, IntoResource, ResourceFactory,
    ResourceInputBuilder,
};

#[cfg(any(feature = "diesel-async-bb8", feature = "diesel-async-deadpool"))]
use diesel_async::pooled_connection::AsyncDieselConnectionManager;

#[cfg(feature = "diesel-async-bb8")]
use diesel_async::pooled_connection::bb8 as diesel_bb8;

#[cfg(feature = "diesel-async-deadpool")]
use diesel_async::pooled_connection::deadpool as diesel_deadpool;

#[allow(dead_code)]
const MIN_CONNECTIONS: u32 = 1;
#[allow(dead_code)]
const MAX_CONNECTIONS: u32 = 5;

/// Shuttle managed Postgres DB in a shared cluster
#[derive(Default)]
pub struct Postgres(DbInput);

impl Postgres {
    /// Use a custom connection string for local runs
    pub fn local_uri(mut self, local_uri: &str) -> Self {
        self.0.local_uri = Some(local_uri.to_string());

        self
    }
}

/// Conditionally request a Shuttle resource
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybeRequest {
    Request(ProvisionResourceRequest),
    NotRequest(DatabaseResource),
}

#[async_trait]
impl ResourceInputBuilder for Postgres {
    type Input = MaybeRequest;
    type Output = OutputWrapper;

    async fn build(self, factory: &ResourceFactory) -> Result<Self::Input, Error> {
        let md = factory.get_metadata();
        Ok(match md.env {
            Environment::Deployment => MaybeRequest::Request(ProvisionResourceRequest::new(
                Type::Database(database::Type::Shared(database::SharedEngine::Postgres)),
                serde_json::to_value(self.0).unwrap(),
                serde_json::Value::Null,
            )),
            Environment::Local => match self.0.local_uri {
                Some(local_uri) => {
                    MaybeRequest::NotRequest(DatabaseResource::ConnectionString(local_uri))
                }
                None => MaybeRequest::Request(ProvisionResourceRequest::new(
                    Type::Database(database::Type::Shared(database::SharedEngine::Postgres)),
                    serde_json::to_value(self.0).unwrap(),
                    serde_json::Value::Null,
                )),
            },
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputWrapper {
    Alpha(ShuttleResourceOutput<DatabaseResource>),
    Beta(DatabaseResource),
}

#[async_trait]
impl IntoResource<String> for OutputWrapper {
    async fn into_resource(self) -> Result<String, Error> {
        let output = match self {
            Self::Alpha(o) => o.output,
            Self::Beta(o) => o,
        };
        Ok(match output {
            DatabaseResource::ConnectionString(s) => s,
            DatabaseResource::Info(info) => info.connection_string_shuttle(),
        })
    }
}

#[cfg(feature = "diesel-async")]
#[async_trait]
impl IntoResource<diesel_async::AsyncPgConnection> for OutputWrapper {
    async fn into_resource(self) -> Result<diesel_async::AsyncPgConnection, Error> {
        use diesel_async::{AsyncConnection, AsyncPgConnection};

        let connection_string: String = self.into_resource().await?;

        Ok(AsyncPgConnection::establish(&connection_string)
            .await
            .map_err(shuttle_service::error::CustomError::new)?)
    }
}

#[cfg(feature = "diesel-async-bb8")]
#[async_trait]
impl IntoResource<diesel_bb8::Pool<diesel_async::AsyncPgConnection>> for OutputWrapper {
    async fn into_resource(
        self,
    ) -> Result<diesel_bb8::Pool<diesel_async::AsyncPgConnection>, Error> {
        let connection_string: String = self.into_resource().await?;

        Ok(diesel_bb8::Pool::builder()
            .min_idle(Some(MIN_CONNECTIONS))
            .max_size(MAX_CONNECTIONS)
            .build(AsyncDieselConnectionManager::new(connection_string))
            .await
            .map_err(shuttle_service::error::CustomError::new)?)
    }
}

#[cfg(feature = "diesel-async-deadpool")]
#[async_trait]
impl IntoResource<diesel_deadpool::Pool<diesel_async::AsyncPgConnection>> for OutputWrapper {
    async fn into_resource(
        self,
    ) -> Result<diesel_deadpool::Pool<diesel_async::AsyncPgConnection>, Error> {
        let connection_string: String = self.into_resource().await?;

        Ok(
            diesel_deadpool::Pool::builder(AsyncDieselConnectionManager::new(connection_string))
                .max_size(MAX_CONNECTIONS as usize)
                .build()
                .map_err(shuttle_service::error::CustomError::new)?,
        )
    }
}

#[cfg(feature = "sqlx")]
#[async_trait]
impl IntoResource<sqlx::PgPool> for OutputWrapper {
    async fn into_resource(self) -> Result<sqlx::PgPool, Error> {
        let connection_string: String = self.into_resource().await?;

        Ok(sqlx::postgres::PgPoolOptions::new()
            .min_connections(MIN_CONNECTIONS)
            .max_connections(MAX_CONNECTIONS)
            .connect(&connection_string)
            .await
            .map_err(shuttle_service::error::CustomError::new)?)
    }
}

#[cfg(feature = "opendal-postgres")]
#[async_trait]
impl IntoResource<opendal::Operator> for OutputWrapper {
    async fn into_resource(self) -> Result<opendal::Operator, Error> {
        let connection_string: String = self.into_resource().await?;
        let pool = sqlx::postgres::PgPoolOptions::new()
            .min_connections(MIN_CONNECTIONS)
            .max_connections(MAX_CONNECTIONS)
            .connect(&connection_string)
            .await
            .map_err(shuttle_service::error::CustomError::new)?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS opendal (key TEXT PRIMARY KEY, value BYTEA NOT NULL)",
        )
        .execute(&pool)
        .await
        .map_err(shuttle_service::error::CustomError::new)?;

        let config = opendal::services::Postgresql::default()
            .root("/")
            .connection_string(&connection_string)
            .table("opendal")
            // key field type in the table should be compatible with Rust's &str like text
            .key_field("key")
            // value field type in the table should be compatible with Rust's Vec<u8> like bytea
            .value_field("value");
        let op = opendal::Operator::new(config)
            .map_err(shuttle_service::error::CustomError::new)?
            .finish();

        Ok(op)
    }
}

/// Alternative Operator wrapper that provides methods for serializing (and deserializing) data
/// as JSON before being stored in the operator's backend.
#[cfg(feature = "opendal-postgres")]
#[derive(Clone, Debug)]
pub struct SerdeJsonOperator(pub opendal::Operator);
#[cfg(feature = "opendal-postgres")]
impl SerdeJsonOperator {
    pub async fn read_serialized<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<T, opendal::Error> {
        let bytes = self.0.read(key).await?;

        serde_json::from_slice(&bytes.to_vec()).map_err(|e| {
            opendal::Error::new(opendal::ErrorKind::Unexpected, "deserialization error")
                .set_source(e)
        })
    }
    pub async fn write_serialized<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), opendal::Error> {
        let b = serde_json::to_vec(value).map_err(|e| {
            opendal::Error::new(opendal::ErrorKind::Unexpected, "serialization error").set_source(e)
        })?;

        self.0.write(key, b).await
    }
}
#[cfg(feature = "opendal-postgres")]
#[async_trait]
impl IntoResource<SerdeJsonOperator> for OutputWrapper {
    async fn into_resource(self) -> Result<SerdeJsonOperator, Error> {
        Ok(SerdeJsonOperator(self.into_resource().await?))
    }
}
