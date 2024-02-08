pub mod database;

use shuttle_common::{claims::Claim, resource::Type as CommonResourceType};
use shuttle_proto::resource_recorder::{
    record_request, ResourceResponse, ResourcesResponse, ResultResponse,
};
use sqlx::{
    sqlite::{SqliteArgumentValue, SqliteRow, SqliteValueRef},
    Database, FromRow, Row, Sqlite,
};
use std::{borrow::Cow, fmt::Display, str::FromStr};
use ulid::Ulid;

pub use self::database::Type as DatabaseType;

#[async_trait::async_trait]
pub trait ResourceManager: Clone + Send + Sync + 'static {
    type Err: std::error::Error;

    async fn insert_resources(
        &mut self,
        resources: Vec<record_request::Resource>,
        service_id: &ulid::Ulid,
        claim: Claim,
    ) -> Result<ResultResponse, Self::Err>;
    async fn get_resources(
        &mut self,
        service_id: &ulid::Ulid,
        claim: Claim,
    ) -> Result<ResourcesResponse, Self::Err>;
    async fn get_resource(
        &mut self,
        service_id: &ulid::Ulid,
        r#type: CommonResourceType,
        claim: Claim,
    ) -> Result<ResourceResponse, Self::Err>;
    async fn delete_resource(
        &mut self,
        project_name: String,
        service_id: &ulid::Ulid,
        r#type: CommonResourceType,
        claim: Claim,
    ) -> Result<ResultResponse, Self::Err>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct Resource {
    pub service_id: Ulid,
    pub r#type: Type,
    pub data: serde_json::Value,
    pub config: serde_json::Value,
}

impl FromRow<'_, SqliteRow> for Resource {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            service_id: Ulid::from_string(row.try_get("service_id")?)
                .expect("to have a valid ulid string"),
            r#type: row.try_get("type")?,
            data: row.try_get("data")?,
            config: row.try_get("config")?,
        })
    }
}

impl From<Resource> for shuttle_common::resource::Response {
    fn from(resource: Resource) -> Self {
        shuttle_common::resource::Response {
            r#type: resource.r#type.into(),
            config: resource.config,
            data: resource.data,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Database(DatabaseType),
    Secrets,
    StaticFolder,
    Persist,
    Turso,
    Metadata,
    Opendal,
    Custom,
}

impl From<Type> for CommonResourceType {
    fn from(r#type: Type) -> Self {
        match r#type {
            Type::Database(r#type) => Self::Database(r#type.into()),
            Type::Secrets => Self::Secrets,
            Type::StaticFolder => Self::StaticFolder,
            Type::Persist => Self::Persist,
            Type::Turso => Self::Turso,
            Type::Metadata => Self::Metadata,
            Type::Opendal => Self::Opendal,
            Type::Custom => Self::Custom,
        }
    }
}

impl From<CommonResourceType> for Type {
    fn from(r#type: CommonResourceType) -> Self {
        match r#type {
            CommonResourceType::Database(r#type) => Self::Database(r#type.into()),
            CommonResourceType::Secrets => Self::Secrets,
            CommonResourceType::StaticFolder => Self::StaticFolder,
            CommonResourceType::Persist => Self::Persist,
            CommonResourceType::Turso => Self::Turso,
            CommonResourceType::Opendal => Self::Opendal,
            CommonResourceType::Metadata => Self::Metadata,
            CommonResourceType::Custom => Self::Custom,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Database(db_type) => write!(f, "database::{db_type}"),
            Type::Secrets => write!(f, "secrets"),
            Type::StaticFolder => write!(f, "static_folder"),
            Type::Persist => write!(f, "persist"),
            Type::Turso => write!(f, "turso"),
            Type::Opendal => write!(f, "opendal"),
            Type::Metadata => write!(f, "metadata"),
            Type::Custom => write!(f, "custom"),
        }
    }
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((prefix, rest)) = s.split_once("::") {
            match prefix {
                "database" => Ok(Self::Database(DatabaseType::from_str(rest)?)),
                _ => Err(format!("'{prefix}' is an unknown resource type")),
            }
        } else {
            match s {
                "secrets" => Ok(Self::Secrets),
                "static_folder" => Ok(Self::StaticFolder),
                "persist" => Ok(Self::Persist),
                "turso" => Ok(Self::Turso),
                "metadata" => Ok(Self::Metadata),
                "opendal" => Ok(Self::Opendal),
                "custom" => Ok(Self::Custom),
                _ => Err(format!("'{s}' is an unknown resource type")),
            }
        }
    }
}

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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{database, Type};

    #[test]
    fn to_string_and_back() {
        let inputs = [
            Type::Database(database::Type::AwsRds(database::AwsRdsType::Postgres)),
            Type::Database(database::Type::AwsRds(database::AwsRdsType::MySql)),
            Type::Database(database::Type::AwsRds(database::AwsRdsType::MariaDB)),
            Type::Database(database::Type::Shared(database::SharedType::Postgres)),
            Type::Database(database::Type::Shared(database::SharedType::MongoDb)),
            Type::Secrets,
            Type::StaticFolder,
            Type::Persist,
            Type::Turso,
            Type::Metadata,
            Type::Custom,
        ];

        for input in inputs {
            let actual = Type::from_str(&input.to_string()).unwrap();
            assert_eq!(input, actual, ":{} should map back to itself", input);
        }
    }
}
