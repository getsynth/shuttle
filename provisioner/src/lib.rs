use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

pub use args::Args;
use aws_config::timeout;
use aws_sdk_rds::{
    error::SdkError, operation::modify_db_instance::ModifyDBInstanceError, types::DbInstance,
    Client,
};
pub use error::Error;
use mongodb::{bson::doc, options::ClientOptions};
use rand::Rng;
use shuttle_backends::auth::VerifyClaim;
use shuttle_backends::client::ServicesApiClient;
use shuttle_backends::project_name::ProjectName;
use shuttle_backends::ClaimExt;
use shuttle_common::claims::{Claim, Scope};
pub use shuttle_proto::provisioner::provisioner_server::ProvisionerServer;
use shuttle_proto::provisioner::{
    aws_rds, database_request::DbType, provisioner_server::Provisioner, shared, AwsRds,
    DatabaseDeletionResponse, DatabaseRequest, DatabaseResponse, Ping, Pong, Shared,
};
use shuttle_proto::resource_recorder;
use sqlx::{postgres::PgPoolOptions, ConnectOptions, Executor, PgPool};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tonic::transport::Uri;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};

mod args;
mod error;

const AWS_RDS_CLASS: &str = "db.t4g.micro";
const MASTER_USERNAME: &str = "master";
const RDS_SUBNET_GROUP: &str = "shuttle_rds";

pub struct ShuttleProvisioner {
    pool: PgPool,
    rds_client: aws_sdk_rds::Client,
    mongodb_client: mongodb::Client,
    fqdn: String,
    internal_pg_address: String,
    internal_mongodb_address: String,
    rr_client: Arc<Mutex<resource_recorder::Client>>,
    gateway_client: ServicesApiClient,
}

impl ShuttleProvisioner {
    pub async fn new(
        shared_pg_uri: &str,
        shared_mongodb_uri: &str,
        fqdn: String,
        internal_pg_address: String,
        internal_mongodb_address: String,
        resource_recorder_uri: Uri,
        gateway_uri: Uri,
    ) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .min_connections(4)
            .max_connections(12)
            .acquire_timeout(Duration::from_secs(60))
            .connect_lazy(shared_pg_uri)?;

        let mongodb_options = ClientOptions::parse(shared_mongodb_uri).await?;
        let mongodb_client = mongodb::Client::with_options(mongodb_options)?;

        // Default timeout is too long so lowering it
        let timeout_config = timeout::TimeoutConfig::builder()
            .operation_timeout(Duration::from_secs(120))
            .operation_attempt_timeout(Duration::from_secs(120))
            .build();

        let aws_config = aws_config::from_env()
            .timeout_config(timeout_config)
            .load()
            .await;

        let rds_client = aws_sdk_rds::Client::new(&aws_config);

        let rr_client = resource_recorder::get_client(resource_recorder_uri).await;

        let gateway_client = ServicesApiClient::new(gateway_uri);

        Ok(Self {
            pool,
            rds_client,
            mongodb_client,
            fqdn,
            internal_pg_address,
            internal_mongodb_address,
            rr_client: Arc::new(Mutex::new(rr_client)),
            gateway_client,
        })
    }

    pub async fn request_shared_db(
        &self,
        project_name: &str,
        engine: shared::Engine,
    ) -> Result<DatabaseResponse, Error> {
        match engine {
            shared::Engine::Postgres(_) => {
                let (username, password) = self.shared_pg_role(project_name).await?;
                let database_name = self.shared_pg(project_name, &username).await?;

                Ok(DatabaseResponse {
                    engine: "postgres".to_string(),
                    username,
                    password,
                    database_name,
                    address_private: self.internal_pg_address.clone(),
                    address_public: self.fqdn.clone(),
                    port: "5432".to_string(),
                })
            }
            shared::Engine::Mongodb(_) => {
                let database_name = format!("mongodb-{project_name}");
                let (username, password) =
                    self.shared_mongodb(project_name, &database_name).await?;

                Ok(DatabaseResponse {
                    engine: "mongodb".to_string(),
                    username,
                    password,
                    database_name,
                    address_private: self.internal_mongodb_address.clone(),
                    address_public: self.fqdn.clone(),
                    port: "27017".to_string(),
                })
            }
        }
    }

    async fn shared_pg_role(&self, project_name: &str) -> Result<(String, String), Error> {
        let username = format!("user-{project_name}");
        let password = generate_password();

        let matching_user = sqlx::query("SELECT rolname FROM pg_roles WHERE rolname = $1")
            .bind(&username)
            .fetch_optional(&self.pool)
            .await?;

        if matching_user.is_none() {
            info!("creating new user");

            // Binding does not work for identifiers
            // https://stackoverflow.com/questions/63723236/sql-statement-to-create-role-fails-on-postgres-12-using-dapper
            let create_role_query =
                format!("CREATE ROLE \"{username}\" WITH LOGIN PASSWORD '{password}'");
            sqlx::query(&create_role_query)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::CreateRole(e.to_string()))?;
        } else {
            info!("cycling password of user");

            // Binding does not work for identifiers
            // https://stackoverflow.com/questions/63723236/sql-statement-to-create-role-fails-on-postgres-12-using-dapper
            let update_role_query =
                format!("ALTER ROLE \"{username}\" WITH LOGIN PASSWORD '{password}'");
            sqlx::query(&update_role_query)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::UpdateRole(e.to_string()))?;
        }

        Ok((username, password))
    }

    async fn shared_pg(&self, project_name: &str, username: &str) -> Result<String, Error> {
        let database_name = format!("db-{project_name}");

        let matching_db = sqlx::query("SELECT datname FROM pg_database WHERE datname = $1")
            .bind(&database_name)
            .fetch_optional(&self.pool)
            .await?;

        if matching_db.is_none() {
            info!("creating database");

            // Binding does not work for identifiers
            // https://stackoverflow.com/questions/63723236/sql-statement-to-create-role-fails-on-postgres-12-using-dapper
            let create_db_query = format!("CREATE DATABASE \"{database_name}\" OWNER '{username}'");
            sqlx::query(&create_db_query)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::CreateDB(e.to_string()))?;

            // Make sure database can't see other databases or other users
            // For #557
            let options = self
                .pool
                .connect_options()
                .deref()
                .clone()
                .database(&database_name);

            let mut conn = options.connect().await?;

            let stmts = vec![
                "REVOKE ALL ON pg_user FROM public;",
                "REVOKE ALL ON pg_roles FROM public;",
                "REVOKE ALL ON pg_database FROM public;",
            ];

            for stmt in stmts {
                conn.execute(stmt)
                    .await
                    .map_err(|e| Error::CreateDB(e.to_string()))?;
            }
        }

        Ok(database_name)
    }

    async fn shared_mongodb(
        &self,
        project_name: &str,
        database_name: &str,
    ) -> Result<(String, String), Error> {
        let username = format!("user-{project_name}");
        let password = generate_password();

        // Get a handle to the DB, create it if it doesn't exist
        let db = self.mongodb_client.database(database_name);

        // Create a new user if it doesn't already exist and assign them
        // permissions to read and write to their own database only
        let new_user = doc! {
            "createUser": &username,
            "pwd": &password,
            "roles": [
                {"role": "readWrite", "db": database_name}
            ]
        };
        let result = db.run_command(new_user, None).await;

        match result {
            Ok(_) => {
                info!("new user created");
                Ok((username, password))
            }
            Err(e) => {
                // If user already exists (error code: 51003) cycle their password
                if e.to_string().contains("51003") {
                    info!("cycling password of user");

                    let change_password = doc! {
                        "updateUser": &username,
                        "pwd": &password,
                    };
                    db.run_command(change_password, None).await?;

                    Ok((username, password))
                } else {
                    Err(Error::UnexpectedMongodb(e))
                }
            }
        }
    }

    async fn request_aws_rds(
        &self,
        project_name: &str,
        engine: aws_rds::Engine,
        database_name: &Option<String>,
    ) -> Result<DatabaseResponse, Error> {
        let client = &self.rds_client;

        let password = generate_password();
        let instance_name = format!("{}-{}", project_name, engine);

        debug!("trying to get AWS RDS instance: {instance_name}");
        let instance = client
            .modify_db_instance()
            .db_instance_identifier(&instance_name)
            .master_user_password(&password)
            .send()
            .await;

        match instance {
            Ok(_) => {
                wait_for_instance(client, &instance_name, "resetting-master-credentials").await?;
            }
            Err(SdkError::ServiceError(err)) => {
                if let ModifyDBInstanceError::DbInstanceNotFoundFault(_) = err.err() {
                    debug!("creating new AWS RDS {instance_name}");

                    let db_name = database_name
                        .to_owned()
                        .unwrap_or_else(|| project_name.to_string());

                    client
                        .create_db_instance()
                        .db_instance_identifier(&instance_name)
                        .master_username(MASTER_USERNAME)
                        .master_user_password(&password)
                        .engine(engine.to_string())
                        .db_instance_class(AWS_RDS_CLASS)
                        .allocated_storage(20)
                        .backup_retention_period(0) // Disable backups
                        .publicly_accessible(true)
                        .db_name(db_name)
                        .set_db_subnet_group_name(Some(RDS_SUBNET_GROUP.to_string()))
                        .send()
                        .await?
                        .db_instance
                        .expect("to be able to create instance");

                    wait_for_instance(client, &instance_name, "creating").await?;
                } else {
                    return Err(Error::Plain(format!(
                        "got unexpected error from AWS RDS service: {}",
                        err.err()
                    )));
                }
            }
            Err(unexpected) => {
                return Err(Error::Plain(format!(
                    "got unexpected error from AWS during API call: {}",
                    unexpected
                )))
            }
        };

        // Wait for up
        let instance = wait_for_instance(client, &instance_name, "available").await?;

        // TODO: find private IP somehow
        let address = instance
            .endpoint
            .expect("instance to have an endpoint")
            .address
            .expect("endpoint to have an address");

        Ok(DatabaseResponse {
            engine: engine.to_string(),
            username: instance
                .master_username
                .expect("instance to have a username"),
            password,
            database_name: instance
                .db_name
                .expect("instance to have a default database"),
            address_private: address.clone(),
            address_public: address,
            port: engine_to_port(engine),
        })
    }

    async fn delete_shared_db(
        &self,
        project_name: &str,
        engine: shared::Engine,
    ) -> Result<DatabaseDeletionResponse, Error> {
        match engine {
            shared::Engine::Postgres(_) => self.delete_shared_postgres(project_name).await?,
            shared::Engine::Mongodb(_) => self.delete_shared_mongodb(project_name).await?,
        }
        Ok(DatabaseDeletionResponse {})
    }

    async fn delete_shared_postgres(&self, project_name: &str) -> Result<(), Error> {
        let database_name = format!("db-{project_name}");
        let role_name = format!("user-{project_name}");

        if sqlx::query("SELECT 1 FROM pg_database WHERE datname = $1")
            .bind(&database_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DeleteDB(e.to_string()))?
            .is_some()
        {
            // Identifiers cannot be used as query parameters.
            let drop_db_query = format!("DROP DATABASE \"{database_name}\" WITH (FORCE)");

            // Drop the database with force, which will try to terminate existing connections to the
            // database. This can fail if prepared transactions, active logical replication slots or
            // subscriptions are present in the database.
            sqlx::query(&drop_db_query)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::DeleteDB(e.to_string()))?;

            info!("dropped shared postgres database: {database_name}");
        } else {
            warn!("did not drop shared postgres database: {database_name}. Does not exist.");
        }

        if sqlx::query("SELECT 1 FROM pg_roles WHERE rolname = $1")
            .bind(&role_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DeleteRole(e.to_string()))?
            .is_some()
        {
            // Drop the role.
            let drop_role_query = format!("DROP ROLE IF EXISTS \"{role_name}\"");
            sqlx::query(&drop_role_query)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::DeleteRole(e.to_string()))?;

            info!("dropped shared postgres role: {role_name}");
        } else {
            warn!("did not drop shared postgres role: {role_name}. Does not exist.");
        }

        Ok(())
    }

    async fn delete_shared_mongodb(&self, project_name: &str) -> Result<(), Error> {
        let database_name = format!("mongodb-{project_name}");
        let db = self.mongodb_client.database(&database_name);

        // Dropping a database in mongodb doesn't delete any associated users
        // so do that first.
        let drop_users_command = doc! {
            "dropAllUsersFromDatabase": 1
        };

        db.run_command(drop_users_command, None)
            .await
            .map_err(|e| Error::DeleteRole(e.to_string()))?;

        info!("dropped users from shared mongodb database: {database_name}");

        // Drop the actual database.
        db.drop(None)
            .await
            .map_err(|e| Error::DeleteDB(e.to_string()))?;

        info!("dropped shared mongodb database: {database_name}");

        Ok(())
    }

    async fn delete_aws_rds(
        &self,
        project_name: &str,
        engine: aws_rds::Engine,
    ) -> Result<DatabaseDeletionResponse, Error> {
        let client = &self.rds_client;
        let instance_name = format!("{project_name}-{engine}");

        // Try to delete the db instance.
        client
            .delete_db_instance()
            .skip_final_snapshot(true)
            .db_instance_identifier(&instance_name)
            .send()
            .await?;

        info!("deleted database instance: {instance_name}");

        Ok(DatabaseDeletionResponse {})
    }

    async fn verify_ownership(&self, claim: &Claim, project_name: &str) -> Result<(), Status> {
        if !claim.is_admin()
            && !claim.is_deployer()
            && !claim
                .owns_project(&self.gateway_client, project_name)
                .await
                .map_err(|_| Status::internal("could not verify project ownership"))?
        {
            let status = Status::permission_denied("the request lacks the authorizations");
            error!(error = &status as &dyn std::error::Error);
            return Err(status);
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl Provisioner for ShuttleProvisioner {
    #[tracing::instrument(skip(self))]
    async fn provision_database(
        &self,
        request: Request<DatabaseRequest>,
    ) -> Result<Response<DatabaseResponse>, Status> {
        request.verify(Scope::ResourcesWrite)?;
        let claim = request.get_claim()?;
        let request = request.into_inner();
        if !ProjectName::is_valid(&request.project_name) {
            return Err(Status::invalid_argument("invalid project name"));
        }
        self.verify_ownership(&claim, &request.project_name).await?;

        let db_type = request.db_type.unwrap();

        let reply = match db_type {
            DbType::Shared(Shared { engine }) => {
                self.request_shared_db(&request.project_name, engine.expect("engine to be set"))
                    .await?
            }
            DbType::AwsRds(AwsRds { engine }) => {
                {
                    let mut rr_client = self.rr_client.lock().await;

                    let can_provision = match claim
                        .can_provision_rds(&self.gateway_client, &mut *rr_client)
                        .await
                    {
                        Ok(can_provision) => can_provision,
                        Err(error) => {
                            error!(
                                error = &error as &dyn std::error::Error,
                                "failed to check current RDS subscription"
                            );

                            return Err(tonic::Status::internal(
                                "failed to check current RDS subscription",
                            ));
                        }
                    };

                    if !can_provision {
                        return Err(tonic::Status::permission_denied(
                            "don't have permission to provision rds instances",
                        ));
                    }
                }

                self.request_aws_rds(
                    &request.project_name,
                    engine.expect("engine to be set"),
                    &request.db_name,
                )
                .await?
            }
        };

        Ok(Response::new(reply))
    }

    #[tracing::instrument(skip(self))]
    async fn delete_database(
        &self,
        request: Request<DatabaseRequest>,
    ) -> Result<Response<DatabaseDeletionResponse>, Status> {
        request.verify(Scope::ResourcesWrite)?;
        let claim = request.get_claim()?;
        let request = request.into_inner();
        if !ProjectName::is_valid(&request.project_name) {
            return Err(Status::invalid_argument("invalid project name"));
        }
        self.verify_ownership(&claim, &request.project_name).await?;

        let db_type = request.db_type.unwrap();

        let reply = match db_type {
            DbType::Shared(Shared { engine }) => {
                self.delete_shared_db(&request.project_name, engine.expect("engine to be set"))
                    .await?
            }
            DbType::AwsRds(AwsRds { engine }) => {
                self.delete_aws_rds(&request.project_name, engine.expect("engine to be set"))
                    .await?
            }
        };

        Ok(Response::new(reply))
    }

    #[tracing::instrument(skip(self))]
    async fn health_check(&self, _request: Request<Ping>) -> Result<Response<Pong>, Status> {
        Ok(Response::new(Pong {}))
    }
}

fn generate_password() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

async fn wait_for_instance(
    client: &Client,
    name: &str,
    wait_for: &str,
) -> Result<DbInstance, Error> {
    debug!("waiting for {name} to enter {wait_for} state");
    loop {
        let instance = client
            .describe_db_instances()
            .db_instance_identifier(name)
            .send()
            .await?
            .db_instances
            .expect("aws to return instances")
            .first()
            .expect("to find the instance just created or modified")
            .clone();

        let status = instance
            .db_instance_status
            .as_ref()
            .expect("instance to have a status")
            .clone();

        if status == wait_for {
            return Ok(instance);
        }

        sleep(Duration::from_secs(1)).await;
    }
}

fn engine_to_port(engine: aws_rds::Engine) -> String {
    match engine {
        aws_rds::Engine::Postgres(_) => "5432".to_string(),
        aws_rds::Engine::Mariadb(_) => "3306".to_string(),
        aws_rds::Engine::Mysql(_) => "3306".to_string(),
    }
}
