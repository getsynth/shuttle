use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::resource::ResourceTypeBeta;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    AwsRds(AwsRdsEngine),
    Shared(SharedEngine),
}

#[derive(Clone, Copy, Debug, Deserialize, Display, Serialize, EnumString, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AwsRdsEngine {
    Postgres,
    MySql,
    MariaDB,
}

#[derive(Clone, Copy, Debug, Deserialize, Display, Serialize, EnumString, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SharedEngine {
    Postgres,
    MongoDb,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::AwsRds(rds_type) => write!(f, "aws_rds::{rds_type}"),
            Type::Shared(shared_type) => write!(f, "shared::{shared_type}"),
        }
    }
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Some((prefix, rest)) = s.split_once("::") {
            match prefix {
                "aws_rds" => Ok(Self::AwsRds(
                    AwsRdsEngine::from_str(rest).map_err(|e| e.to_string())?,
                )),
                "shared" => Ok(Self::Shared(
                    SharedEngine::from_str(rest).map_err(|e| e.to_string())?,
                )),
                _ => Err(format!("'{prefix}' is an unknown database type")),
            }
        } else {
            Err(format!("'{s}' is an unknown database type"))
        }
    }
}

impl TryFrom<ResourceTypeBeta> for Type {
    type Error = String;

    fn try_from(value: ResourceTypeBeta) -> Result<Self, Self::Error> {
        match value {
            ResourceTypeBeta::DatabaseSharedPostgres => Ok(Self::Shared(SharedEngine::Postgres)),
            ResourceTypeBeta::DatabaseAwsRdsPostgres => Ok(Self::AwsRds(AwsRdsEngine::Postgres)),
            ResourceTypeBeta::DatabaseAwsRdsMysql => Ok(Self::AwsRds(AwsRdsEngine::MySql)),
            ResourceTypeBeta::DatabaseAwsRdsMariaDB => Ok(Self::AwsRds(AwsRdsEngine::MariaDB)),
            other => Err(format!(
                "{other} is not a valid database resource on the shuttle.dev platform"
            )),
        }
    }
}
