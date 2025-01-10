use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use opendal::{Operator, Scheme};
use serde::{Deserialize, Serialize};
use shuttle_service::{
    error::{CustomError, Error as ShuttleError},
    IntoResource, ResourceFactory, ResourceInputBuilder,
};

#[derive(Serialize)]
pub struct Opendal {
    scheme: String,
}

impl Default for Opendal {
    fn default() -> Self {
        Self {
            scheme: "memory".to_string(),
        }
    }
}

impl Opendal {
    pub fn scheme(mut self, scheme: &str) -> Self {
        self.scheme = scheme.to_string();
        self
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpendalOutput {
    scheme: String,
    cfg: HashMap<String, String>,
}

pub struct Error(opendal::Error);

impl From<Error> for shuttle_service::Error {
    fn from(error: Error) -> Self {
        let msg = format!("Failed to build opendal resource: {:?}", error.0);
        ShuttleError::Custom(CustomError::msg(msg))
    }
}

#[async_trait]
impl ResourceInputBuilder for Opendal {
    type Input = OpendalOutput;
    type Output = OpendalOutput;

    async fn build(self, factory: &ResourceFactory) -> Result<Self::Input, ShuttleError> {
        Ok(OpendalOutput {
            scheme: self.scheme,
            cfg: factory
                .get_secrets()
                .into_iter()
                .map(|(k, v)| (k, v.expose().clone()))
                .collect(),
        })
    }
}

#[async_trait]
impl IntoResource<Operator> for OpendalOutput {
    async fn into_resource(self) -> Result<Operator, shuttle_service::Error> {
        let scheme = Scheme::from_str(&self.scheme).map_err(Error)?;

        Ok(Operator::via_iter(scheme, self.cfg).map_err(Error)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use shuttle_service::Secret;

    #[tokio::test]
    async fn opendal_fs() {
        let factory = ResourceFactory::new(
            Default::default(),
            [("root", "/tmp")]
                .into_iter()
                .map(|(k, v)| (k.to_string(), Secret::new(v.to_string())))
                .collect(),
            Default::default(),
        );

        let odal = Opendal::default().scheme("fs");
        let output = odal.build(&factory).await.unwrap();
        assert_eq!(output.scheme, "fs");

        let op: Operator = output.into_resource().await.unwrap();
        assert_eq!(op.info().scheme(), Scheme::Fs)
    }

    #[tokio::test]
    async fn opendal_s3() {
        let factory = ResourceFactory::new(
            Default::default(),
            [
                ("bucket", "test"),
                ("access_key_id", "ak"),
                ("secret_access_key", "sk"),
                ("region", "us-east-1"),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), Secret::new(v.to_string())))
            .collect(),
            Default::default(),
        );

        let odal = Opendal::default().scheme("s3");
        let output = odal.build(&factory).await.unwrap();
        assert_eq!(output.scheme, "s3");
        assert_eq!(output.cfg.get("access_key_id").unwrap(), "ak");
        assert_eq!(output.cfg.get("secret_access_key").unwrap(), "sk");

        let op: Operator = output.into_resource().await.unwrap();
        assert_eq!(op.info().scheme(), Scheme::S3)
    }
}
