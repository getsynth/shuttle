use async_trait::async_trait;
use libsql_client::{Client, Config};
use serde::{Deserialize, Serialize};
use shuttle_service::{
    error::{CustomError, Error as ShuttleError},
    Environment, Factory, ResourceBuilder, Type,
};
use url::Url;

#[derive(Serialize, Deserialize, Default)]
pub struct Turso {
    addr: String,
    token: String,
    local_addr: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TursoOutput {
    conn_url: Url,
    token: Option<String>,
}

impl Turso {
    pub fn addr(mut self, addr: &str) -> Self {
        self.addr = addr.to_string();
        self
    }

    pub fn token(mut self, token: &str) -> Self {
        self.token = token.to_string();
        self
    }

    pub fn local_addr(mut self, local_addr: &str) -> Self {
        self.local_addr = Some(local_addr.to_string());
        self
    }
}

pub enum Error {
    UrlParseError(url::ParseError),
    LocateLocalDB(std::io::Error),
}

impl From<Error> for shuttle_service::Error {
    fn from(error: Error) -> Self {
        let msg = match error {
            Error::UrlParseError(err) => format!("Failed to parse Turso Url: {}", err),
            Error::LocateLocalDB(err) => format!("Failed to get path to local db file: {}", err),
        };

        ShuttleError::Custom(CustomError::msg(msg))
    }
}

impl Turso {
    async fn output_from_addr(
        &self,
        addr: &str,
    ) -> Result<<Turso as ResourceBuilder<Client>>::Output, shuttle_service::Error> {
        Ok(TursoOutput {
            conn_url: Url::parse(addr).map_err(Error::UrlParseError)?,
            token: if self.token.is_empty() {
                None
            } else {
                Some(self.token.clone())
            },
        })
    }
}

#[async_trait]
impl ResourceBuilder<Client> for Turso {
    const TYPE: Type = Type::Turso;

    type Config = Self;
    type Output = TursoOutput;

    fn new() -> Self {
        Self::default()
    }

    fn config(&self) -> &Self::Config {
        self
    }

    async fn output(
        self,
        factory: &mut dyn Factory,
    ) -> Result<Self::Output, shuttle_service::Error> {
        let md = factory.get_metadata();
        match md.env {
            Environment::Deployment => {
                if self.addr.is_empty() {
                    Err(ShuttleError::Custom(CustomError::msg("missing addr")))
                } else {
                    let addr = if self.addr.starts_with("libsql://") {
                        self.addr.to_string()
                    } else {
                        format!("libsql://{}", self.addr)
                    };
                    self.output_from_addr(&addr).await
                }
            }
            Environment::Local => {
                match self.local_addr {
                    Some(ref local_addr) => self.output_from_addr(local_addr).await,
                    None => {
                        // Default to a local db of the name of the service.
                        let db_file = std::env::current_dir() // Should be root of the project's workspace
                            .and_then(dunce::canonicalize)
                            .map(|cd| {
                                let mut p = cd.join(md.service_name);
                                p.set_extension("db");
                                p
                            })
                            .map_err(Error::LocateLocalDB)?;
                        let conn_url = format!("file:///{}", db_file.display());
                        Ok(TursoOutput {
                            conn_url: Url::parse(&conn_url).map_err(Error::UrlParseError)?,
                            // Nullify the token since we're using a file as database.
                            token: None,
                        })
                    }
                }
            }
        }
    }

    async fn build(config: &Self::Output) -> Result<Client, shuttle_service::Error> {
        let client = Client::from_config(Config {
            url: config.conn_url.clone(),
            auth_token: config.token.clone(),
        })
        .await?;
        Ok(client)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use shuttle_service::Secret;

    struct MockFactory {
        pub environment: Environment,
    }

    impl MockFactory {
        fn new(environment: Environment) -> Self {
            Self { environment }
        }
    }

    #[async_trait]
    impl Factory for MockFactory {
        async fn get_db_connection(
            &mut self,
            _db_type: shuttle_service::database::Type,
        ) -> Result<shuttle_service::DatabaseReadyInfo, shuttle_service::Error> {
            panic!("no turso test should try to get a db connection string")
        }

        async fn get_secrets(
            &mut self,
        ) -> Result<std::collections::BTreeMap<String, Secret<String>>, shuttle_service::Error>
        {
            panic!("no turso test should try to get secrets")
        }

        fn get_metadata(&self) -> shuttle_service::DeploymentMetadata {
            shuttle_service::DeploymentMetadata {
                env: self.environment,
                project_name: "my-turso-service".to_string(),
                service_name: "my-turso-service".to_string(),
                storage_path: std::path::PathBuf::new(),
            }
        }
    }

    #[tokio::test]
    async fn local_database_default() {
        let mut factory = MockFactory::new(Environment::Local);

        let turso = Turso::new();
        let output = turso.output(&mut factory).await.unwrap();
        assert_eq!(output.token, None);
        assert!(output.conn_url.to_string().starts_with("file:///"));
        assert!(output.conn_url.to_string().ends_with("my-turso-service.db"));
    }

    #[tokio::test]
    async fn local_database_user_supplied() {
        let mut factory = MockFactory::new(Environment::Local);

        let mut turso = Turso::new();
        let local_addr = "libsql://test-addr.turso.io";
        turso = turso.local_addr(local_addr);

        let output = turso.output(&mut factory).await.unwrap();
        assert_eq!(
            output,
            TursoOutput {
                conn_url: Url::parse(local_addr).unwrap(),
                token: None
            }
        )
    }

    #[tokio::test]
    #[should_panic(expected = "missing addr")]
    async fn remote_database_empty_addr() {
        let mut factory = MockFactory::new(Environment::Deployment);

        let turso = Turso::new();
        turso.output(&mut factory).await.unwrap();
    }

    #[tokio::test]
    async fn remote_database() {
        let mut factory = MockFactory::new(Environment::Deployment);

        let mut turso = Turso::new();
        let addr = "my-turso-addr.turso.io".to_string();
        turso.addr = addr.clone();
        turso.token = "token".to_string();
        let output = turso.output(&mut factory).await.unwrap();

        assert_eq!(
            output,
            TursoOutput {
                conn_url: Url::parse(&format!("libsql://{}", addr)).unwrap(),
                token: Some("token".to_string())
            }
        )
    }
}
