use std::time::Duration;

pub use error::Error;
use provisioner::provisioner_server::Provisioner;
pub use provisioner::provisioner_server::ProvisionerServer;
use provisioner::{DatabaseRequest, DatabaseResponse};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tonic::{Request, Response, Status};

mod error;

pub mod provisioner {
    tonic::include_proto!("provisioner");
}

pub struct MyProvisioner {
    pool: PgPool,
}

impl MyProvisioner {
    pub fn new(uri: &str) -> sqlx::Result<Self> {
        Ok(Self {
            pool: PgPoolOptions::new()
                .min_connections(4)
                .max_connections(12)
                .connect_timeout(Duration::from_secs(60))
                .connect_lazy(&uri)?,
        })
    }

    pub async fn request_shared_db(&self, project_name: String) -> Result<DatabaseResponse, Error> {
        // Binding does not work for identifiers
        // https://stackoverflow.com/questions/63723236/sql-statement-to-create-role-fails-on-postgres-12-using-dapper
        let create_role_query = format!(
            "CREATE ROLE \"{}\" WITH LOGIN PASSWORD '{}'",
            project_name, "test"
        );
        sqlx::query(&create_role_query)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::CreateRole(e.to_string()))?;

        Ok(DatabaseResponse {
            username: project_name,
            password: "test".to_string(),
            database_name: "".to_string(),
        })
    }
}

#[tonic::async_trait]
impl Provisioner for MyProvisioner {
    async fn provision_database(
        &self,
        request: Request<DatabaseRequest>,
    ) -> Result<Response<DatabaseResponse>, Status> {
        println!("request: {:?}", request.into_inner());

        let reply = DatabaseResponse {
            username: "postgres".to_string(),
            password: "tmp".to_string(),
            database_name: "postgres".to_string(),
        };

        Ok(Response::new(reply))
    }
}
