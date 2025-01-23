use std::{collections::HashMap, io::stdout, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use bollard::{
    container::{Config, CreateContainerOptions, StartContainerOptions},
    exec::{CreateExecOptions, CreateExecResults},
    image::CreateImageOptions,
    models::{CreateImageInfo, HostConfig, PortBinding, ProgressDetail},
    service::ContainerInspectResponse,
    Docker,
};
use crossterm::{
    cursor::{MoveDown, MoveUp},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use futures::StreamExt;
use portpicker::pick_unused_port;
use shuttle_common::{
    database::{self, AwsRdsEngine, SharedEngine},
    ContainerRequest, ContainerResponse, Secret,
};
use shuttle_proto::provisioner::{
    provisioner_server::Provisioner, DatabaseDeletionResponse, DatabaseDumpRequest,
    DatabaseDumpResponse, DatabaseRequest, DatabaseResponse, Ping, Pong,
};
use shuttle_service::database::Type;
use tokio::time::sleep;
use tonic::{Request, Response, Status};
use tracing::{error, trace};

/// A provisioner for local runs
/// It uses Docker to create Databases
pub struct LocalProvisioner {
    docker: Docker,
}

impl LocalProvisioner {
    pub fn new() -> Result<Self> {
        // This only constructs the client and does not try to connect.
        // If the socket is not found, a "no such file" error will happen on the first request to Docker.
        Ok(Self {
            docker: Docker::connect_with_defaults()?,
        })
    }

    fn get_container_first_host_port(
        &self,
        container: &ContainerInspectResponse,
        port: &str,
    ) -> String {
        container
            .host_config
            .as_ref()
            .expect("container to have host config")
            .port_bindings
            .as_ref()
            .expect("port bindings on container")
            .get(port)
            .expect("a port bindings entry")
            .as_ref()
            .expect("a port bindings")
            .first()
            .expect("at least one port binding")
            .host_port
            .as_ref()
            .expect("a host port")
            .clone()
    }

    async fn start_container_if_not_running(
        &self,
        container: &ContainerInspectResponse,
        container_type: &str,
        name: &str,
    ) {
        if !container
            .state
            .as_ref()
            .expect("container to have a state")
            .running
            .expect("state to have a running key")
        {
            trace!("{container_type} container '{name}' not running, so starting it");
            self.docker
                .start_container(name, None::<StartContainerOptions<String>>)
                .await
                .expect("failed to start container");
        }
    }

    async fn get_container(
        &self,
        container_name: &str,
        image: &str,
        port: &str,
        env: Option<Vec<String>>,
    ) -> Result<ContainerInspectResponse, Status> {
        match self.docker.inspect_container(container_name, None).await {
            Ok(container) => {
                trace!("found container {container_name}");
                Ok(container)
            }
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => {
                self.pull_image(image).await.expect("failed to pull image");
                trace!("will create container {container_name}");
                let options = Some(CreateContainerOptions {
                    name: container_name,
                    platform: None,
                });
                let mut port_bindings = HashMap::new();
                let host_port = pick_unused_port().expect("system to have a free port");
                port_bindings.insert(
                    port.to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(host_port.to_string()),
                        ..Default::default()
                    }]),
                );
                let host_config = HostConfig {
                    port_bindings: Some(port_bindings),
                    ..Default::default()
                };

                let config: Config<String> = Config {
                    image: Some(image.to_string()),
                    env,
                    host_config: Some(host_config),
                    ..Default::default()
                };

                self.docker
                    .create_container(options, config)
                    .await
                    .expect("to be able to create container");

                Ok(self
                    .docker
                    .inspect_container(container_name, None)
                    .await
                    .expect("container to be created"))
            }
            Err(error) => {
                error!("Got unexpected error while inspecting docker container: {error}");
                error!(
                    "Make sure Docker is installed and running. For more help: https://docs.shuttle.dev/docs/local-run#docker-engines"
                );
                Err(Status::internal(error.to_string()))
            }
        }
    }

    async fn get_db_connection_string(
        &self,
        project_name: &str,
        db_type: Type,
        db_name: Option<String>,
    ) -> Result<DatabaseResponse, Status> {
        trace!("getting sql string for project '{project_name}'");

        let database_name = match db_type {
            database::Type::AwsRds(_) => db_name.unwrap_or_else(|| project_name.to_string()),
            database::Type::Shared(SharedEngine::MongoDb) => "admin".to_string(),
            _ => project_name.to_string(),
        };

        let EngineConfig {
            r#type,
            image,
            engine,
            username,
            password,
            port,
            env,
            is_ready_cmd,
        } = db_type_to_config(db_type, &database_name);
        let container_name = format!("shuttle_{project_name}_{type}");

        let container = self
            .get_container(&container_name, &image, &port, env)
            .await?;

        let host_port = self.get_container_first_host_port(&container, &port);

        self.start_container_if_not_running(&container, &r#type, &container_name)
            .await;

        self.wait_for_ready(&container_name, is_ready_cmd.clone())
            .await?;

        // The container enters the ready state, runs an init script and then reboots, so we sleep
        // a little and then check if it's ready again afterwards.
        sleep(Duration::from_millis(450)).await;
        self.wait_for_ready(&container_name, is_ready_cmd).await?;

        let res = DatabaseResponse {
            engine,
            username,
            password: password.expose().to_owned(),
            database_name,
            port: host_port,
            address_private: "localhost".to_string(),
            address_public: "localhost".to_string(),
        };

        Ok(res)
    }

    pub async fn start_container(
        &self,
        req: ContainerRequest,
    ) -> Result<ContainerResponse, Status> {
        let ContainerRequest {
            project_name,
            container_name,
            env,
            image,
            port,
        } = req;

        let container_name = format!("shuttle_{project_name}_{container_name}");

        let container = self
            .get_container(&container_name, &image, &port, Some(env))
            .await?;

        let host_port = self.get_container_first_host_port(&container, &port);

        self.start_container_if_not_running(&container, &container_name, &container_name)
            .await;

        Ok(ContainerResponse { host_port })
    }

    async fn wait_for_ready(
        &self,
        container_name: &str,
        is_ready_cmd: Vec<String>,
    ) -> Result<(), Status> {
        loop {
            trace!("waiting for '{container_name}' to be ready for connections");

            let config = CreateExecOptions {
                cmd: Some(is_ready_cmd.clone()),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            };

            let CreateExecResults { id } = self
                .docker
                .create_exec(container_name, config)
                .await
                .expect("failed to create exec to check if container is ready");

            let ready_result = self
                .docker
                .start_exec(&id, None)
                .await
                .expect("failed to execute ready command");

            if let bollard::exec::StartExecResults::Attached { mut output, .. } = ready_result {
                while let Some(line) = output.next().await {
                    trace!("line: {:?}", line);

                    if let bollard::container::LogOutput::StdOut { .. } =
                        line.expect("output to have a log line")
                    {
                        return Ok(());
                    }
                }
            }

            sleep(Duration::from_millis(500)).await;
        }
    }

    async fn pull_image(&self, image: &str) -> Result<(), String> {
        trace!("pulling latest image for '{image}'");
        let mut layers = Vec::new();

        let create_image_options = Some(CreateImageOptions {
            from_image: image,
            ..Default::default()
        });
        let mut output = self.docker.create_image(create_image_options, None, None);

        while let Some(line) = output.next().await {
            let info = line.expect("failed to create image");

            if let Some(id) = info.id.as_ref() {
                match layers
                    .iter_mut()
                    .find(|item: &&mut CreateImageInfo| item.id.as_deref() == Some(id))
                {
                    Some(item) => *item = info,
                    None => layers.push(info),
                }
            } else {
                layers.push(info);
            }

            print_layers(&layers);
        }

        // Undo last MoveUps
        stdout()
            .queue(MoveDown(
                layers.len().try_into().expect("to convert usize to u16"),
            ))
            .expect("to reset cursor position");

        Ok(())
    }
}

#[async_trait]
impl Provisioner for LocalProvisioner {
    async fn provision_database(
        &self,
        request: Request<DatabaseRequest>,
    ) -> Result<Response<DatabaseResponse>, Status> {
        let DatabaseRequest {
            project_name,
            db_type,
            db_name,
        } = request.into_inner();

        let db_type: Option<Type> = db_type.unwrap().into();

        let res = self
            .get_db_connection_string(&project_name, db_type.unwrap(), db_name)
            .await?;

        Ok(Response::new(res))
    }

    async fn delete_database(
        &self,
        _request: Request<DatabaseRequest>,
    ) -> Result<Response<DatabaseDeletionResponse>, Status> {
        panic!("local runner should not try to delete databases");
    }

    async fn dump_database(
        &self,
        _request: Request<DatabaseDumpRequest>,
    ) -> Result<Response<DatabaseDumpResponse>, Status> {
        panic!("local runner should not try to dump databases");
    }

    async fn health_check(&self, _request: Request<Ping>) -> Result<Response<Pong>, Status> {
        panic!("local runner should not try to do a health check");
    }
}

fn print_layers(layers: &Vec<CreateImageInfo>) {
    for info in layers {
        stdout()
            .queue(Clear(ClearType::CurrentLine))
            .expect("to be able to clear line");

        if let Some(id) = info.id.as_ref() {
            let text = match (info.status.as_deref(), info.progress_detail.as_ref()) {
                (
                    Some("Downloading"),
                    Some(ProgressDetail {
                        current: Some(c),
                        total: Some(t),
                    }),
                ) => {
                    let percent = *c as f64 / *t as f64 * 100.0;
                    let progress = (percent as i64 / 10) as usize;
                    let remaining = 10 - progress;
                    format!("{:=<progress$}>{:remaining$}   {percent:.0}%", "", "")
                }
                (Some(status), _) => status.to_string(),
                _ => "Unknown".to_string(),
            };
            println!("[{id} {text}]");
        } else {
            println!(
                "{}",
                info.status.as_ref().expect("image info to have a status")
            )
        }
    }
    stdout()
        .queue(MoveUp(
            layers.len().try_into().expect("to convert usize to u16"),
        ))
        .expect("to reset cursor position");
}

struct EngineConfig {
    r#type: String,
    image: String,
    engine: String,
    username: String,
    password: Secret<String>,
    port: String,
    env: Option<Vec<String>>,
    is_ready_cmd: Vec<String>,
}

fn db_type_to_config(db_type: Type, database_name: &str) -> EngineConfig {
    match db_type {
        Type::Shared(SharedEngine::Postgres) => EngineConfig {
            r#type: "shared_postgres".to_string(),
            image: "docker.io/library/postgres:14".to_string(),
            engine: "postgres".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string().into(),
            port: "5432/tcp".to_string(),
            env: Some(vec![
                "POSTGRES_PASSWORD=postgres".to_string(),
                format!("POSTGRES_DB={database_name}"),
            ]),
            is_ready_cmd: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "pg_isready | grep 'accepting connections'".to_string(),
            ],
        },
        Type::Shared(SharedEngine::MongoDb) => EngineConfig {
            r#type: "shared_mongodb".to_string(),
            image: "docker.io/library/mongo:5.0.10".to_string(),
            engine: "mongodb".to_string(),
            username: "mongodb".to_string(),
            password: "password".to_string().into(),
            port: "27017/tcp".to_string(),
            env: Some(vec![
                "MONGO_INITDB_ROOT_USERNAME=mongodb".to_string(),
                "MONGO_INITDB_ROOT_PASSWORD=password".to_string(),
                format!("MONGO_INITDB_DATABASE={database_name}"),
            ]),
            is_ready_cmd: vec![
                "mongosh".to_string(),
                "--quiet".to_string(),
                "--eval".to_string(),
                "db".to_string(),
            ],
        },
        Type::AwsRds(AwsRdsEngine::Postgres) => EngineConfig {
            r#type: "aws_rds_postgres".to_string(),
            image: "docker.io/library/postgres:13.4".to_string(),
            engine: "postgres".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string().into(),
            port: "5432/tcp".to_string(),
            env: Some(vec![
                "POSTGRES_PASSWORD=postgres".to_string(),
                format!("POSTGRES_DB={database_name}"),
            ]),
            is_ready_cmd: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "pg_isready | grep 'accepting connections'".to_string(),
            ],
        },
        Type::AwsRds(AwsRdsEngine::MariaDB) => EngineConfig {
            r#type: "aws_rds_mariadb".to_string(),
            image: "docker.io/library/mariadb:10.6.7".to_string(),
            engine: "mariadb".to_string(),
            username: "root".to_string(),
            password: "mariadb".to_string().into(),
            port: "3306/tcp".to_string(),
            env: Some(vec![
                "MARIADB_ROOT_PASSWORD=mariadb".to_string(),
                format!("MARIADB_DATABASE={database_name}"),
            ]),
            is_ready_cmd: vec![
                "mysql".to_string(),
                "-pmariadb".to_string(),
                "--silent".to_string(),
                "-e".to_string(),
                "show databases;".to_string(),
            ],
        },
        Type::AwsRds(AwsRdsEngine::MySql) => EngineConfig {
            r#type: "aws_rds_mysql".to_string(),
            image: "docker.io/library/mysql:8.0.28".to_string(),
            engine: "mysql".to_string(),
            username: "root".to_string(),
            password: "mysql".to_string().into(),
            port: "3306/tcp".to_string(),
            env: Some(vec![
                "MYSQL_ROOT_PASSWORD=mysql".to_string(),
                format!("MYSQL_DATABASE={database_name}"),
            ]),
            is_ready_cmd: vec![
                "mysql".to_string(),
                "-pmysql".to_string(),
                "--silent".to_string(),
                "-e".to_string(),
                "show databases;".to_string(),
            ],
        },
    }
}

pub mod beta {
    use std::{
        collections::HashMap, convert::Infallible, net::SocketAddr, process::exit, sync::Arc,
    };

    use anyhow::{bail, Context, Result};
    use hyper::{
        body,
        service::{make_service_fn, service_fn},
        Body, Method, Request as HyperRequest, Response, Server,
    };
    use shuttle_common::{
        database,
        models::resource::get_resource_tables_beta,
        resource::{
            self, ProvisionResourceRequestBeta, ResourceResponseBeta, ResourceState,
            ResourceTypeBeta,
        },
        DatabaseResource, DbInput,
    };
    use shuttle_proto::provisioner::{provisioner_server::Provisioner, DatabaseRequest};
    use tonic::Request;
    use tracing::debug;

    use super::LocalProvisioner;

    #[derive(Clone)]
    pub struct ProvApiState {
        pub project_name: String,
        pub secrets: HashMap<String, String>,
    }

    pub struct ProvisionerServerBeta;

    impl ProvisionerServerBeta {
        pub fn start(state: Arc<ProvApiState>, api_addr: &SocketAddr) {
            let make_svc = make_service_fn(move |_conn| {
                let state = state.clone();
                async {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        let state = state.clone();
                        handler(state, req)
                    }))
                }
            });
            let server = Server::bind(api_addr).serve(make_svc);
            tokio::spawn(async move {
                if let Err(e) = server.await {
                    eprintln!("Provisioner server error: {}", e);
                    exit(1);
                }
            });
        }
    }

    pub async fn handler(
        state: Arc<ProvApiState>,
        req: HyperRequest<Body>,
    ) -> std::result::Result<Response<Body>, hyper::Error> {
        let method = req.method().clone();
        let uri = req.uri().clone();
        debug!("Received {method} {uri}");

        let body = body::to_bytes(req.into_body()).await?.to_vec();
        let res = match provision(state, method, uri.to_string().as_str(), body).await {
            Ok(bytes) => Response::new(Body::from(bytes)),
            Err(e) => {
                eprintln!("Encountered error when provisioning: {e}");
                Response::builder().status(500).body(Body::empty()).unwrap()
            }
        };

        Ok(res)
    }

    async fn provision(
        state: Arc<ProvApiState>,
        method: Method,
        uri: &str,
        body: Vec<u8>,
    ) -> Result<Vec<u8>> {
        Ok(match (method, uri) {
            (Method::GET, "/projects/proj_LOCAL/resources/secrets") => {
                let response = ResourceResponseBeta {
                    r#type: ResourceTypeBeta::Secrets,
                    state: ResourceState::Ready,
                    config: serde_json::Value::Null,
                    output: serde_json::to_value(&state.secrets).unwrap(),
                };
                let table =
                    get_resource_tables_beta(&[response.clone()], "local service", false, true);
                println!("{table}");
                serde_json::to_vec(&response).unwrap()
            }
            (Method::POST, "/projects/proj_LOCAL/resources") => {
                let prov = LocalProvisioner::new().unwrap();
                let shuttle_resource: ProvisionResourceRequestBeta =
                    serde_json::from_slice(&body).context("deserializing resource request")?;

                let response = match shuttle_resource.r#type {
                    ResourceTypeBeta::DatabaseSharedPostgres
                    | ResourceTypeBeta::DatabaseAwsRdsMariaDB
                    | ResourceTypeBeta::DatabaseAwsRdsMysql
                    | ResourceTypeBeta::DatabaseAwsRdsPostgres => {
                        let config: DbInput =
                            serde_json::from_value(shuttle_resource.config.clone())
                                .context("deserializing resource config")?;
                        let res = DatabaseResource::Info(
                            prov.provision_database(Request::new(DatabaseRequest {
                                project_name: state.project_name.clone(),
                                db_type: Some(
                                    database::Type::try_from(shuttle_resource.r#type)
                                        .map_err(anyhow::Error::msg)?
                                        .into()
                                ),
                                db_name: config.db_name,
                            }))
                            .await
                            .context("Failed to start database container. Make sure that a Docker engine is running.")?
                            .into_inner()
                            .into());
                        ResourceResponseBeta {
                            r#type: shuttle_resource.r#type,
                            state: resource::ResourceState::Ready,
                            config: shuttle_resource.config,
                            output: serde_json::to_value(res).unwrap(),
                        }
                    }
                    ResourceTypeBeta::Container => {
                        let config = serde_json::from_value(shuttle_resource.config.clone())
                            .context("deserializing resource config")?;
                        let res = prov.start_container(config)
                            .await
                            .context("Failed to start Docker container. Make sure that a Docker engine is running.")?;
                        ResourceResponseBeta {
                            r#type: shuttle_resource.r#type,
                            state: resource::ResourceState::Ready,
                            config: shuttle_resource.config,
                            output: serde_json::to_value(res).unwrap(),
                        }
                    }
                    ResourceTypeBeta::Secrets => ResourceResponseBeta {
                        r#type: shuttle_resource.r#type,
                        state: resource::ResourceState::Ready,
                        config: shuttle_resource.config,
                        output: serde_json::to_value(&state.secrets).unwrap(),
                    },
                };

                let table =
                    get_resource_tables_beta(&[response.clone()], "local service", false, true);
                println!("{table}");

                serde_json::to_vec(&response).unwrap()
            }
            _ => bail!("Received unsupported resource request"),
        })
    }
}
