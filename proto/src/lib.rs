// This clippy is disabled as per this prost comment
// https://github.com/tokio-rs/prost/issues/661#issuecomment-1156606409
#![allow(clippy::derive_partial_eq_without_eq)]

pub mod provisioner {
    use std::fmt::Display;

    use shuttle_common::{
        database::{self, AwsRdsEngine, SharedEngine},
        DatabaseReadyInfo,
    };

    include!("generated/provisioner.rs");

    impl From<DatabaseResponse> for DatabaseReadyInfo {
        fn from(response: DatabaseResponse) -> Self {
            DatabaseReadyInfo::new(
                response.engine,
                response.username,
                response.password,
                response.database_name,
                response.port,
                response.address_private,
                response.address_public,
            )
        }
    }

    impl From<database::Type> for database_request::DbType {
        fn from(db_type: database::Type) -> Self {
            match db_type {
                database::Type::Shared(engine) => {
                    let engine = match engine {
                        SharedEngine::Postgres => shared::Engine::Postgres(String::new()),
                        SharedEngine::MongoDb => shared::Engine::Mongodb(String::new()),
                    };
                    database_request::DbType::Shared(Shared {
                        engine: Some(engine),
                    })
                }
                database::Type::AwsRds(engine) => {
                    let config = RdsConfig {};
                    let engine = match engine {
                        AwsRdsEngine::Postgres => aws_rds::Engine::Postgres(config),
                        AwsRdsEngine::MariaDB => aws_rds::Engine::Mariadb(config),
                        AwsRdsEngine::MySql => aws_rds::Engine::Mysql(config),
                    };
                    database_request::DbType::AwsRds(AwsRds {
                        engine: Some(engine),
                    })
                }
            }
        }
    }

    impl From<database_request::DbType> for Option<database::Type> {
        fn from(db_type: database_request::DbType) -> Self {
            match db_type {
                database_request::DbType::Shared(Shared {
                    engine: Some(engine),
                }) => match engine {
                    shared::Engine::Postgres(_) => {
                        Some(database::Type::Shared(SharedEngine::Postgres))
                    }
                    shared::Engine::Mongodb(_) => {
                        Some(database::Type::Shared(SharedEngine::MongoDb))
                    }
                },
                database_request::DbType::AwsRds(AwsRds {
                    engine: Some(engine),
                }) => match engine {
                    aws_rds::Engine::Postgres(_) => {
                        Some(database::Type::AwsRds(AwsRdsEngine::Postgres))
                    }
                    aws_rds::Engine::Mysql(_) => Some(database::Type::AwsRds(AwsRdsEngine::MySql)),
                    aws_rds::Engine::Mariadb(_) => {
                        Some(database::Type::AwsRds(AwsRdsEngine::MariaDB))
                    }
                },
                database_request::DbType::Shared(Shared { engine: None })
                | database_request::DbType::AwsRds(AwsRds { engine: None }) => None,
            }
        }
    }

    impl Display for aws_rds::Engine {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Mariadb(_) => write!(f, "mariadb"),
                Self::Mysql(_) => write!(f, "mysql"),
                Self::Postgres(_) => write!(f, "postgres"),
            }
        }
    }
}

pub mod runtime {
    use std::{path::PathBuf, process::Stdio, time::Duration};

    use anyhow::Context;
    use shuttle_common::claims::{
        ClaimLayer, ClaimService, InjectPropagation, InjectPropagationLayer,
    };
    use tokio::process;
    use tonic::transport::{Channel, Endpoint};
    use tower::ServiceBuilder;
    use tracing::{info, trace};

    pub enum StorageManagerType {
        Artifacts(PathBuf),
        WorkingDir(PathBuf),
    }

    include!("generated/runtime.rs");

    pub async fn start(
        wasm: bool,
        storage_manager_type: StorageManagerType,
        provisioner_address: &str,
        logger_uri: &str,
        auth_uri: Option<&String>,
        port: u16,
        get_runtime_executable: impl FnOnce() -> PathBuf,
    ) -> anyhow::Result<(
        process::Child,
        runtime_client::RuntimeClient<ClaimService<InjectPropagation<Channel>>>,
    )> {
        let (storage_manager_type, storage_manager_path) = match storage_manager_type {
            StorageManagerType::Artifacts(path) => ("artifacts", path),
            StorageManagerType::WorkingDir(path) => ("working-dir", path),
        };

        let port = &port.to_string();
        let storage_manager_path = &storage_manager_path.display().to_string();
        let runtime_executable_path = get_runtime_executable();

        let args = if wasm {
            vec!["--port", port]
        } else {
            let mut args = vec![
                "--port",
                port,
                "--provisioner-address",
                provisioner_address,
                "--logger-uri",
                logger_uri,
                "--storage-manager-type",
                storage_manager_type,
                "--storage-manager-path",
                storage_manager_path,
            ];

            if let Some(auth_uri) = auth_uri {
                args.append(&mut vec!["--auth-uri", auth_uri]);
            }

            args
        };

        trace!(
            "Spawning runtime process {:?} {:?}",
            runtime_executable_path,
            args
        );
        let runtime = process::Command::new(runtime_executable_path)
            .args(&args)
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("spawning runtime process")?;

        info!("connecting runtime client");
        let conn = Endpoint::new(format!("http://127.0.0.1:{port}"))
            .context("creating runtime client endpoint")?
            .connect_timeout(Duration::from_secs(5));

        // Wait for the spawned process to open the endpoint port.
        // Connecting instantly does not give it enough time.
        let channel = tokio::time::timeout(Duration::from_millis(7000), async move {
            let mut ms = 5;
            loop {
                if let Ok(channel) = conn.connect().await {
                    break channel;
                }
                trace!("waiting for runtime endpoint to open");
                // exponential backoff
                tokio::time::sleep(Duration::from_millis(ms)).await;
                ms *= 2;
            }
        })
        .await
        .context("runtime client endpoint did not open in time")?;

        let channel = ServiceBuilder::new()
            .layer(ClaimLayer)
            .layer(InjectPropagationLayer)
            .service(channel);
        let runtime_client = runtime_client::RuntimeClient::new(channel);

        Ok((runtime, runtime_client))
    }
}

pub mod resource_recorder {
    use std::str::FromStr;

    include!("generated/resource_recorder.rs");

    impl From<record_request::Resource> for shuttle_common::resource::Response {
        fn from(resource: record_request::Resource) -> Self {
            shuttle_common::resource::Response {
                r#type: shuttle_common::resource::Type::from_str(resource.r#type.as_str())
                    .expect("to have a valid resource string"),
                config: serde_json::from_slice(&resource.config)
                    .expect("to have JSON valid config"),
                data: serde_json::from_slice(&resource.data).expect("to have JSON valid data"),
            }
        }
    }

    impl From<Resource> for shuttle_common::resource::Response {
        fn from(resource: Resource) -> Self {
            shuttle_common::resource::Response {
                r#type: shuttle_common::resource::Type::from_str(resource.r#type.as_str())
                    .expect("to have a valid resource string"),
                config: serde_json::from_slice(&resource.config)
                    .expect("to have JSON valid config"),
                data: serde_json::from_slice(&resource.data).expect("to have JSON valid data"),
            }
        }
    }
}

pub mod logger {
    use std::sync::mpsc;

    use tonic::async_trait;

    include!("generated/logger.rs");

    /// Adapter to some client which expects to receive a vector of items
    #[async_trait]
    pub trait VecReceiver: Send {
        type Item;

        async fn receive(&mut self, items: Vec<Self::Item>);
    }

    /// Wrapper to batch together items before forwarding them to some vector receiver
    pub struct Batcher<I: VecReceiver> {
        tx: mpsc::Sender<I::Item>,
    }

    impl<I: VecReceiver + 'static> Batcher<I>
    where
        I::Item: Send,
    {
        /// Create a new batcher around inner with the given batch capacity
        pub fn new(inner: I, capacity: usize) -> Self {
            let (tx, rx) = mpsc::channel();

            tokio::spawn(Self::batch(inner, rx, capacity));

            Self { tx }
        }

        /// Send a single item into this batcher
        pub fn send(&self, item: I::Item) {
            match self.tx.send(item) {
                Ok(_) => {}
                Err(_) => todo!(),
            }
        }

        /// Background task to forward the items ones the batch capacity has been reached
        async fn batch(mut inner: I, rx: mpsc::Receiver<I::Item>, capacity: usize) {
            let mut cache = Vec::with_capacity(capacity);

            loop {
                let item = rx.recv();

                match item {
                    Ok(item) => {
                        cache.push(item);

                        if cache.len() == capacity {
                            let old_cache = cache;
                            cache = Vec::with_capacity(capacity);

                            inner.receive(old_cache).await;
                        }
                    }
                    Err(_) => return,
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };

        use tokio::time::sleep;
        use tonic::async_trait;

        use super::{Batcher, VecReceiver};

        #[derive(Default, Clone)]
        struct MockGroupReceiver(Arc<Mutex<Option<Vec<u32>>>>);

        #[async_trait]
        impl VecReceiver for MockGroupReceiver {
            type Item = u32;

            async fn receive(&mut self, items: Vec<Self::Item>) {
                *self.0.lock().unwrap() = Some(items);
            }
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn capacity_reached() {
            let mock = MockGroupReceiver::default();
            let batcher = Batcher::new(mock.clone(), 2);

            batcher.send(1);
            sleep(Duration::from_millis(50)).await;
            assert_eq!(*mock.0.lock().unwrap(), None);

            batcher.send(2);
            sleep(Duration::from_millis(50)).await;
            assert_eq!(*mock.0.lock().unwrap(), Some(vec![1, 2]));

            batcher.send(3);
            sleep(Duration::from_millis(50)).await;
            assert_eq!(*mock.0.lock().unwrap(), Some(vec![1, 2]));

            batcher.send(4);
            sleep(Duration::from_millis(50)).await;
            assert_eq!(*mock.0.lock().unwrap(), Some(vec![3, 4]));
        }
    }
}
