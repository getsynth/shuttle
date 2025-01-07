use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::{constants::RESOURCE_SCHEMA_VERSION, database};

/// Return this struct as a resource config to make Shuttle provision it
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvisionResourceRequest {
    /// The version of the config+data schema for this Shuttle resource
    pub version: u32,
    /// The type of this resource
    pub r#type: Type,
    /// The config used when creating this resource.
    /// Use `Self::version` and `Self::r#type` to know how to parse this data.
    pub config: Value,

    /// Arbitrary extra data to include in this resource
    pub custom: Value,
}

impl ProvisionResourceRequest {
    pub fn new(r#type: Type, config: Value, custom: Value) -> Self {
        Self {
            version: RESOURCE_SCHEMA_VERSION,
            r#type,
            config,
            custom,
        }
    }
}

impl From<ProvisionResourceRequest> for ProvisionResourceRequestBeta {
    fn from(value: ProvisionResourceRequest) -> Self {
        Self {
            r#type: match value.r#type {
                Type::Database(database::Type::Shared(database::SharedEngine::Postgres)) => {
                    ResourceTypeBeta::DatabaseSharedPostgres
                }
                Type::Database(database::Type::AwsRds(database::AwsRdsEngine::Postgres)) => {
                    ResourceTypeBeta::DatabaseAwsRdsPostgres
                }
                Type::Database(database::Type::AwsRds(database::AwsRdsEngine::MariaDB)) => {
                    ResourceTypeBeta::DatabaseAwsRdsMariaDB
                }
                Type::Database(database::Type::AwsRds(database::AwsRdsEngine::MySql)) => {
                    ResourceTypeBeta::DatabaseAwsRdsMysql
                }
                Type::Secrets => ResourceTypeBeta::Secrets,
                Type::Container => ResourceTypeBeta::Container,
                r => panic!("Resource not supported on shuttle.dev: {r}"),
            },
            config: value.config,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare::typeshare]
pub struct ProvisionResourceRequestBeta {
    /// The type of this resource
    pub r#type: ResourceTypeBeta,
    /// The config used when creating this resource.
    /// Use `Self::r#type` to know how to parse this data.
    pub config: Value,
}

/// Helper for deserializing
#[derive(Deserialize)]
#[serde(untagged)] // Try deserializing as a Shuttle resource, fall back to a custom value
pub enum ResourceInput {
    Shuttle(ProvisionResourceRequest),
    Custom(Value),
}

/// Helper for deserializing
#[derive(Deserialize)]
#[serde(untagged)] // Try deserializing as a Shuttle resource, fall back to a custom value
pub enum ResourceInputBeta {
    Shuttle(ProvisionResourceRequestBeta),
    Custom(Value),
}

/// The resource state represents the stage of the provisioning process the resource is in.
#[derive(
    Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[typeshare::typeshare]
pub enum ResourceState {
    Authorizing,
    Provisioning,
    Failed,
    Ready,
    Deleting,
    Deleted,
}

/// Returned when provisioning a Shuttle resource
#[derive(Serialize, Deserialize)]
pub struct ShuttleResourceOutput<T> {
    /// The output type for this Shuttle resource,
    /// contains the data from the provisioner response
    pub output: T,

    /// Arbitrary extra data in this resource
    pub custom: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare::typeshare]
pub struct ResourceResponseBeta {
    pub r#type: ResourceTypeBeta,
    pub state: ResourceState,
    /// The config used when creating this resource. Use the `r#type` to know how to parse this data.
    pub config: Value,
    /// The output type for this resource, if state is Ready. Use the `r#type` to know how to parse this data.
    pub output: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare::typeshare]
pub struct ResourceListResponseBeta {
    pub resources: Vec<ResourceResponseBeta>,
}

/// Common type to hold all the information we need for a generic resource
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Response {
    /// The type of this resource.
    pub r#type: Type,

    /// The config used when creating this resource. Use the `r#type` to know how to parse this data.
    pub config: Value,

    /// The data associated with this resource. Use the `r#type` to know how to parse this data.
    pub data: Value,
}

impl Response {
    pub fn into_bytes(self) -> Vec<u8> {
        self.to_bytes()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("to turn resource into a vec")
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        serde_json::from_slice(&bytes).expect("to turn bytes into a resource")
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Database(database::Type),
    Secrets,
    Persist,
    /// Local provisioner only
    Container,
}

#[derive(
    Clone, Copy, Debug, strum::EnumString, strum::Display, Deserialize, Serialize, Eq, PartialEq,
)]
#[typeshare::typeshare]
// is a flat enum instead of nested enum to allow typeshare
pub enum ResourceTypeBeta {
    #[strum(to_string = "database::shared::postgres")]
    #[serde(rename = "database::shared::postgres")]
    DatabaseSharedPostgres,
    #[strum(to_string = "database::aws_rds::postgres")]
    #[serde(rename = "database::aws_rds::postgres")]
    DatabaseAwsRdsPostgres,
    #[strum(to_string = "database::aws_rds::mysql")]
    #[serde(rename = "database::aws_rds::mysql")]
    DatabaseAwsRdsMysql,
    #[strum(to_string = "database::aws_rds::mariadb")]
    #[serde(rename = "database::aws_rds::mariadb")]
    DatabaseAwsRdsMariaDB,
    /// (Will probably be removed)
    #[strum(to_string = "secrets")]
    #[serde(rename = "secrets")]
    Secrets,
    /// Local provisioner only
    #[strum(to_string = "container")]
    #[serde(rename = "container")]
    Container,
}

impl TryFrom<ResourceTypeBeta> for database::AwsRdsEngine {
    type Error = String;

    fn try_from(value: ResourceTypeBeta) -> Result<Self, Self::Error> {
        Ok(match value {
            ResourceTypeBeta::DatabaseAwsRdsPostgres => Self::Postgres,
            ResourceTypeBeta::DatabaseAwsRdsMysql => Self::MySql,
            ResourceTypeBeta::DatabaseAwsRdsMariaDB => Self::MariaDB,
            other => return Err(format!("Invalid conversion of DB type: {other}")),
        })
    }
}

#[derive(Debug, Error)]
pub enum InvalidResourceType {
    #[error("'{0}' is an unknown database type")]
    Type(String),

    #[error("{0}")]
    Database(String),
}

impl FromStr for Type {
    type Err = InvalidResourceType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((prefix, rest)) = s.split_once("::") {
            match prefix {
                "database" => Ok(Self::Database(
                    database::Type::from_str(rest).map_err(InvalidResourceType::Database)?,
                )),
                _ => Err(InvalidResourceType::Type(prefix.to_string())),
            }
        } else {
            match s {
                "secrets" => Ok(Self::Secrets),
                "persist" => Ok(Self::Persist),
                "container" => Ok(Self::Container),
                _ => Err(InvalidResourceType::Type(s.to_string())),
            }
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Database(db_type) => write!(f, "database::{db_type}"),
            Type::Secrets => write!(f, "secrets"),
            Type::Persist => write!(f, "persist"),
            Type::Container => write!(f, "container"),
        }
    }
}

// this can be removed when deployers AND r-r no longer hold resources in sqlite state
#[cfg(feature = "sqlx")]
mod _sqlx {
    use std::{borrow::Cow, str::FromStr};

    use sqlx::{
        postgres::{PgArgumentBuffer, PgValueRef},
        sqlite::{SqliteArgumentValue, SqliteValueRef},
        Database, Postgres, Sqlite,
    };

    use super::{ResourceState, Type};

    impl<DB: Database> sqlx::Type<DB> for Type
    where
        str: sqlx::Type<DB>,
    {
        fn type_info() -> <DB as Database>::TypeInfo {
            <str as sqlx::Type<DB>>::type_info()
        }
    }

    impl<'q> sqlx::Encode<'q, Sqlite> for Type {
        fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'q>>) -> sqlx::encode::IsNull {
            args.push(SqliteArgumentValue::Text(Cow::Owned(self.to_string())));

            sqlx::encode::IsNull::No
        }
    }

    impl<'r> sqlx::Decode<'r, Sqlite> for Type {
        fn decode(value: SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
            let value = <&str as sqlx::Decode<Sqlite>>::decode(value)?;

            Self::from_str(value).map_err(Into::into)
        }
    }

    impl<DB: sqlx::Database> sqlx::Type<DB> for ResourceState
    where
        str: sqlx::Type<DB>,
    {
        fn type_info() -> <DB as sqlx::Database>::TypeInfo {
            <str as sqlx::Type<DB>>::type_info()
        }
    }

    impl<'q> sqlx::Encode<'q, sqlx::Postgres> for ResourceState {
        fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> sqlx::encode::IsNull {
            #[allow(clippy::needless_borrows_for_generic_args)]
            <&str as sqlx::Encode<Postgres>>::encode(&self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for ResourceState {
        fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
            let value = <&str as sqlx::Decode<Postgres>>::decode(value)?;

            let state = ResourceState::from_str(value)?;
            Ok(state)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_string_and_back() {
        let inputs = [
            Type::Database(database::Type::AwsRds(database::AwsRdsEngine::Postgres)),
            Type::Database(database::Type::AwsRds(database::AwsRdsEngine::MySql)),
            Type::Database(database::Type::AwsRds(database::AwsRdsEngine::MariaDB)),
            Type::Database(database::Type::Shared(database::SharedEngine::Postgres)),
            Type::Database(database::Type::Shared(database::SharedEngine::MongoDb)),
            Type::Secrets,
            Type::Persist,
            Type::Container,
        ];

        for input in inputs {
            let actual = Type::from_str(&input.to_string()).unwrap();
            assert_eq!(input, actual, ":{} should map back to itself", input);
        }
    }

    #[test]
    fn to_string_and_back_beta() {
        let inputs = [
            ResourceTypeBeta::DatabaseSharedPostgres,
            ResourceTypeBeta::Secrets,
            ResourceTypeBeta::Container,
        ];

        for input in inputs {
            let actual = ResourceTypeBeta::from_str(&input.to_string()).unwrap();
            assert_eq!(input, actual, ":{} should map back to itself", input);
        }
    }

    #[test]
    fn beta_compat() {
        let inputs = [
            (
                Type::Database(database::Type::Shared(database::SharedEngine::Postgres)),
                ResourceTypeBeta::DatabaseSharedPostgres,
            ),
            (
                Type::Database(database::Type::AwsRds(database::AwsRdsEngine::Postgres)),
                ResourceTypeBeta::DatabaseAwsRdsPostgres,
            ),
            (
                Type::Database(database::Type::AwsRds(database::AwsRdsEngine::MySql)),
                ResourceTypeBeta::DatabaseAwsRdsMysql,
            ),
            (
                Type::Database(database::Type::AwsRds(database::AwsRdsEngine::MariaDB)),
                ResourceTypeBeta::DatabaseAwsRdsMariaDB,
            ),
            (Type::Secrets, ResourceTypeBeta::Secrets),
            (Type::Container, ResourceTypeBeta::Container),
        ];

        for (alpha, beta) in inputs {
            assert_eq!(alpha.to_string(), beta.to_string());
        }
    }
}
