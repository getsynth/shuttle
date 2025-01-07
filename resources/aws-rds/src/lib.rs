#![doc = include_str!("../README.md")]

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

/// Conditionally request a Shuttle resource
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybeRequest {
    Request(ProvisionResourceRequest),
    NotRequest(DatabaseResource),
}

macro_rules! aws_engine {
    ($feature:expr, $struct_ident:ident) => {
        paste::paste! {
            #[cfg(feature = $feature)]
            #[derive(Default)]
            #[doc = "Shuttle managed AWS RDS " $struct_ident " instance"]
            pub struct $struct_ident(DbInput);

            #[cfg(feature = $feature)]
            impl $struct_ident {
                /// Use a custom connection string for local runs
                pub fn local_uri(mut self, local_uri: &str) -> Self {
                    self.0.local_uri = Some(local_uri.to_string());

                    self
                }

                /// Use something other than the project name as the DB name
                pub fn database_name(mut self, database_name: &str) -> Self {
                    self.0.db_name = Some(database_name.to_string());

                    self
                }
            }

            #[cfg(feature = $feature)]
            #[async_trait::async_trait]
            impl ResourceInputBuilder for $struct_ident {
                type Input = MaybeRequest;
                type Output = OutputWrapper;

                async fn build(self, factory: &ResourceFactory) -> Result<Self::Input, Error> {
                    let md = factory.get_metadata();
                    Ok(match md.env {
                        Environment::Deployment => MaybeRequest::Request(ProvisionResourceRequest::new(
                            Type::Database(
                                database::Type::AwsRds(
                                    database::AwsRdsEngine::$struct_ident
                                )
                            ),
                            serde_json::to_value(&self.0).unwrap(),
                            serde_json::Value::Null,
                        )),
                        Environment::Local => match self.0.local_uri {
                            Some(local_uri) => {
                                MaybeRequest::NotRequest(DatabaseResource::ConnectionString(local_uri))
                            }
                            None => MaybeRequest::Request(ProvisionResourceRequest::new(
                                Type::Database(
                                    database::Type::AwsRds(
                                        database::AwsRdsEngine::$struct_ident
                                    )
                                ),
                                serde_json::to_value(&self.0).unwrap(),
                                serde_json::Value::Null,
                            )),
                        },
                    })
                }
            }
        }
    };
}

aws_engine!("postgres", Postgres);

aws_engine!("mysql", MySql);

aws_engine!("mariadb", MariaDB);

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

// If these were done in the main macro above, this would produce two conflicting `impl IntoResource<sqlx::MySqlPool>`

#[cfg(feature = "diesel-async")]
mod _diesel_async {
    use super::*;

    #[cfg(feature = "postgres")]
    #[async_trait]
    impl IntoResource<diesel_async::AsyncPgConnection> for OutputWrapper {
        async fn into_resource(self) -> Result<diesel_async::AsyncPgConnection, Error> {
            use diesel_async::{AsyncConnection, AsyncPgConnection};

            let connection_string: String = self.into_resource().await.unwrap();
            Ok(AsyncPgConnection::establish(&connection_string)
                .await
                .map_err(shuttle_service::error::CustomError::new)?)
        }
    }

    #[cfg(any(feature = "mysql", feature = "mariadb"))]
    #[async_trait]
    impl IntoResource<diesel_async::AsyncMysqlConnection> for OutputWrapper {
        async fn into_resource(self) -> Result<diesel_async::AsyncMysqlConnection, Error> {
            use diesel_async::{AsyncConnection, AsyncMysqlConnection};

            let connection_string: String = self.into_resource().await.unwrap();
            Ok(AsyncMysqlConnection::establish(&connection_string)
                .await
                .map_err(shuttle_service::error::CustomError::new)?)
        }
    }
}

#[cfg(feature = "diesel-async-bb8")]
mod _diesel_async_bb8 {
    use super::*;

    #[cfg(feature = "postgres")]
    #[async_trait]
    impl IntoResource<diesel_bb8::Pool<diesel_async::AsyncPgConnection>> for OutputWrapper {
        async fn into_resource(
            self,
        ) -> Result<diesel_bb8::Pool<diesel_async::AsyncPgConnection>, Error> {
            let connection_string: String = self.into_resource().await.unwrap();

            Ok(diesel_bb8::Pool::builder()
                .min_idle(Some(MIN_CONNECTIONS))
                .max_size(MAX_CONNECTIONS)
                .build(AsyncDieselConnectionManager::new(connection_string))
                .await
                .map_err(shuttle_service::error::CustomError::new)?)
        }
    }

    #[cfg(any(feature = "mysql", feature = "mariadb"))]
    #[async_trait]
    impl IntoResource<diesel_bb8::Pool<diesel_async::AsyncMysqlConnection>> for OutputWrapper {
        async fn into_resource(
            self,
        ) -> Result<diesel_bb8::Pool<diesel_async::AsyncMysqlConnection>, Error> {
            let connection_string: String = self.into_resource().await.unwrap();

            Ok(diesel_bb8::Pool::builder()
                .min_idle(Some(MIN_CONNECTIONS))
                .max_size(MAX_CONNECTIONS)
                .build(AsyncDieselConnectionManager::new(connection_string))
                .await
                .map_err(shuttle_service::error::CustomError::new)?)
        }
    }
}

#[cfg(feature = "diesel-async-deadpool")]
mod _diesel_async_deadpool {
    use super::*;

    #[cfg(feature = "postgres")]
    #[async_trait]
    impl IntoResource<diesel_deadpool::Pool<diesel_async::AsyncPgConnection>> for OutputWrapper {
        async fn into_resource(
            self,
        ) -> Result<diesel_deadpool::Pool<diesel_async::AsyncPgConnection>, Error> {
            let connection_string: String = self.into_resource().await.unwrap();

            Ok(
                diesel_deadpool::Pool::builder(AsyncDieselConnectionManager::new(
                    connection_string,
                ))
                .max_size(MAX_CONNECTIONS as usize)
                .build()
                .map_err(shuttle_service::error::CustomError::new)?,
            )
        }
    }

    #[cfg(any(feature = "mysql", feature = "mariadb"))]
    #[async_trait]
    impl IntoResource<diesel_deadpool::Pool<diesel_async::AsyncMysqlConnection>> for OutputWrapper {
        async fn into_resource(
            self,
        ) -> Result<diesel_deadpool::Pool<diesel_async::AsyncMysqlConnection>, Error> {
            let connection_string: String = self.into_resource().await.unwrap();

            Ok(
                diesel_deadpool::Pool::builder(AsyncDieselConnectionManager::new(
                    connection_string,
                ))
                .max_size(MAX_CONNECTIONS as usize)
                .build()
                .map_err(shuttle_service::error::CustomError::new)?,
            )
        }
    }
}

#[cfg(feature = "sqlx")]
mod _sqlx {
    use super::*;

    #[cfg(feature = "postgres")]
    #[async_trait]
    impl IntoResource<sqlx::PgPool> for OutputWrapper {
        async fn into_resource(self) -> Result<sqlx::PgPool, Error> {
            let connection_string: String = self.into_resource().await.unwrap();

            Ok(sqlx::postgres::PgPoolOptions::new()
                .min_connections(MIN_CONNECTIONS)
                .max_connections(MAX_CONNECTIONS)
                .connect(&connection_string)
                .await
                .map_err(shuttle_service::error::CustomError::new)?)
        }
    }

    #[cfg(any(feature = "mysql", feature = "mariadb"))]
    #[async_trait]
    impl IntoResource<sqlx::MySqlPool> for OutputWrapper {
        async fn into_resource(self) -> Result<sqlx::MySqlPool, Error> {
            let connection_string: String = self.into_resource().await.unwrap();

            Ok(sqlx::mysql::MySqlPoolOptions::new()
                .min_connections(MIN_CONNECTIONS)
                .max_connections(MAX_CONNECTIONS)
                .connect(&connection_string)
                .await
                .map_err(shuttle_service::error::CustomError::new)?)
        }
    }
}
