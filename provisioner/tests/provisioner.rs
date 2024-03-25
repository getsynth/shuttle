mod helpers;

use ctor::dtor;
use helpers::{exec_mongosh, exec_psql, DbType, DockerInstance};
use once_cell::sync::Lazy;
use serde_json::Value;
use shuttle_backends::test_utils::{
    gateway::get_mocked_gateway_server, resource_recorder::get_mocked_resource_recorder,
};
use shuttle_proto::provisioner::shared;
use shuttle_provisioner::ShuttleProvisioner;
use tonic::transport::Uri;

static PG: Lazy<DockerInstance> = Lazy::new(|| DockerInstance::new(DbType::Postgres));
static MONGODB: Lazy<DockerInstance> = Lazy::new(|| DockerInstance::new(DbType::MongoDb));

async fn get_rr_uri() -> Uri {
    let port = get_mocked_resource_recorder().await;

    format!("http://localhost:{port}").parse().unwrap()
}

async fn get_gateway_uri() -> Uri {
    let server = get_mocked_gateway_server().await;

    server.uri().parse().unwrap()
}

#[dtor]
fn cleanup() {
    PG.cleanup();
    MONGODB.cleanup();
}

mod needs_docker {
    use serde_json::json;
    use shuttle_common::{
        claims::{AccountTier, Claim},
        limits::Limits,
    };
    use shuttle_common_tests::ClaimTestsExt;
    use shuttle_proto::{
        provisioner::{
            aws_rds::Engine, database_request::DbType, provisioner_server::Provisioner, AwsRds,
            DatabaseRequest,
        },
        resource_recorder::{self, record_request, RecordRequest},
    };
    use tonic::{Code, Request};

    use super::*;

    #[tokio::test]
    async fn going_over_rds_quota() {
        let rr_uri = get_rr_uri().await;
        let provisioner = ShuttleProvisioner::new(
            &PG.uri,
            &MONGODB.uri,
            "fqdn".to_string(),
            "pg".to_string(),
            "mongodb".to_string(),
            rr_uri.clone(),
            get_gateway_uri().await,
        )
        .await
        .unwrap();

        // First record some resources
        let mut r_r_client = resource_recorder::get_client(rr_uri).await;
        r_r_client
            .record_resources(Request::new(RecordRequest {
                project_id: "00000000000000000000000001".to_string(),
                service_id: "00000000000000000000000001".to_string(),
                resources: vec![
                    record_request::Resource {
                        r#type: "database::shared::postgres".to_string(),
                        config: serde_json::to_vec(&json!({"public": true})).unwrap(),
                        data: serde_json::to_vec(&json!({"username": "test"})).unwrap(),
                    },
                    // Make one RDS record that already exists
                    record_request::Resource {
                        r#type: "database::aws_rds::mariadb".to_string(),
                        config: serde_json::to_vec(&json!({})).unwrap(),
                        data: serde_json::to_vec(&json!({"username": "maria"})).unwrap(),
                    },
                ],
            }))
            .await
            .unwrap();

        let mut req = Request::new(DatabaseRequest {
            project_name: "user-1-project-1".to_string(),
            db_type: Some(DbType::AwsRds(AwsRds {
                engine: Some(Engine::Postgres(Default::default())),
            })),
            db_name: Some("custom-name".to_string()),
        });

        // Add a claim that only allows for one RDS - the one that will be returned by r-r
        req.extensions_mut().insert(
            Claim::new(
                "user-1".to_string(),
                AccountTier::Basic.into(),
                AccountTier::Basic,
                Limits::new(1, 1),
            )
            .fill_token(),
        );

        let err = provisioner.provision_database(req).await.unwrap_err();

        assert_eq!(
            err.code(),
            Code::PermissionDenied,
            "quota has been reached so user should not be able to provision another RDS"
        );
    }

    #[tokio::test]
    async fn shared_db_role_does_not_exist() {
        let provisioner = ShuttleProvisioner::new(
            &PG.uri,
            &MONGODB.uri,
            "fqdn".to_string(),
            "pg".to_string(),
            "mongodb".to_string(),
            get_rr_uri().await,
            get_gateway_uri().await,
        )
        .await
        .unwrap();

        assert_eq!(
            exec_psql("SELECT rolname FROM pg_roles WHERE rolname = 'user-not_exist'",),
            ""
        );

        provisioner
            .request_shared_db("not_exist", shared::Engine::Postgres(String::new()))
            .await
            .unwrap();

        assert_eq!(
            exec_psql("SELECT rolname FROM pg_roles WHERE rolname = 'user-not_exist'",),
            "user-not_exist"
        );
    }

    #[tokio::test]
    async fn shared_db_role_does_exist() {
        let provisioner = ShuttleProvisioner::new(
            &PG.uri,
            &MONGODB.uri,
            "fqdn".to_string(),
            "pg".to_string(),
            "mongodb".to_string(),
            get_rr_uri().await,
            get_gateway_uri().await,
        )
        .await
        .unwrap();

        exec_psql("CREATE ROLE \"user-exist\" WITH LOGIN PASSWORD 'temp'");
        let password = exec_psql("SELECT passwd FROM pg_shadow WHERE usename = 'user-exist'");

        provisioner
            .request_shared_db("exist", shared::Engine::Postgres(String::new()))
            .await
            .unwrap();

        // Make sure password got cycled
        assert_ne!(
            exec_psql("SELECT passwd FROM pg_shadow WHERE usename = 'user-exist'",),
            password
        );
    }

    #[tokio::test]
    #[should_panic(
        expected = "CreateRole(\"error returned from database: cannot insert multiple commands into a prepared statement\""
    )]
    async fn injection_safe() {
        let provisioner = ShuttleProvisioner::new(
            &PG.uri,
            &MONGODB.uri,
            "fqdn".to_string(),
            "pg".to_string(),
            "mongodb".to_string(),
            get_rr_uri().await,
            get_gateway_uri().await,
        )
        .await
        .unwrap();

        provisioner
            .request_shared_db(
                "new\"; CREATE ROLE \"injected",
                shared::Engine::Postgres(String::new()),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn shared_db_missing() {
        let provisioner = ShuttleProvisioner::new(
            &PG.uri,
            &MONGODB.uri,
            "fqdn".to_string(),
            "pg".to_string(),
            "mongodb".to_string(),
            get_rr_uri().await,
            get_gateway_uri().await,
        )
        .await
        .unwrap();

        assert_eq!(
            exec_psql("SELECT datname FROM pg_database WHERE datname = 'db-missing'",),
            ""
        );

        provisioner
            .request_shared_db("missing", shared::Engine::Postgres(String::new()))
            .await
            .unwrap();

        assert_eq!(
            exec_psql("SELECT datname FROM pg_database WHERE datname = 'db-missing'",),
            "db-missing"
        );
    }

    #[tokio::test]
    async fn shared_db_filled() {
        let provisioner = ShuttleProvisioner::new(
            &PG.uri,
            &MONGODB.uri,
            "fqdn".to_string(),
            "pg".to_string(),
            "mongodb".to_string(),
            get_rr_uri().await,
            get_gateway_uri().await,
        )
        .await
        .unwrap();

        exec_psql("CREATE ROLE \"user-filled\" WITH LOGIN PASSWORD 'temp'");
        exec_psql("CREATE DATABASE \"db-filled\" OWNER 'user-filled'");
        assert_eq!(
            exec_psql("SELECT datname FROM pg_database WHERE datname = 'db-filled'",),
            "db-filled"
        );

        provisioner
            .request_shared_db("filled", shared::Engine::Postgres(String::new()))
            .await
            .unwrap();

        assert_eq!(
            exec_psql("SELECT datname FROM pg_database WHERE datname = 'db-filled'",),
            "db-filled"
        );
    }

    #[tokio::test]
    async fn shared_mongodb_role_does_not_exist() {
        let provisioner = ShuttleProvisioner::new(
            &PG.uri,
            &MONGODB.uri,
            "fqdn".to_string(),
            "pg".to_string(),
            "mongodb".to_string(),
            get_rr_uri().await,
            get_gateway_uri().await,
        )
        .await
        .unwrap();

        let user = exec_mongosh("db.getUser(\"user-not_exist\")", Some("mongodb-not_exist"));
        assert_eq!(user, "null");

        provisioner
            .request_shared_db("not_exist", shared::Engine::Mongodb(String::new()))
            .await
            .unwrap();

        let user = exec_mongosh("db.getUser(\"user-not_exist\")", Some("mongodb-not_exist"));
        assert!(user.contains("mongodb-not_exist.user-not_exist"));
    }

    #[tokio::test]
    async fn shared_mongodb_role_does_exist() {
        let provisioner = ShuttleProvisioner::new(
            &PG.uri,
            &MONGODB.uri,
            "fqdn".to_string(),
            "pg".to_string(),
            "mongodb".to_string(),
            get_rr_uri().await,
            get_gateway_uri().await,
        )
        .await
        .unwrap();

        exec_mongosh(
            r#"db.createUser({ 
            user: "user-exist", 
            pwd: "secure_password", 
            roles: [
                { role: "readWrite", db: "mongodb-exist" }
            ]
        })"#,
            Some("mongodb-exist"),
        );

        let user: Value = serde_json::from_str(&exec_mongosh(
            r#"EJSON.stringify(db.getUser("user-exist", 
            { showCredentials: true }
        ))"#,
            Some("mongodb-exist"),
        ))
        .unwrap();

        // Extract the user's stored password hash key from the `getUser` output
        let user_stored_key = &user["credentials"]["SCRAM-SHA-256"]["storedKey"];
        assert_eq!(user["_id"], "mongodb-exist.user-exist");

        provisioner
            .request_shared_db("exist", shared::Engine::Mongodb(String::new()))
            .await
            .unwrap();

        let user: Value = serde_json::from_str(&exec_mongosh(
            r#"EJSON.stringify(db.getUser("user-exist", 
            { showCredentials: true }
        ))"#,
            Some("mongodb-exist"),
        ))
        .unwrap();

        // Make sure it's the same user
        assert_eq!(user["_id"], "mongodb-exist.user-exist");

        // Make sure password got cycled by comparing password hash keys
        let user_cycled_key = &user["credentials"]["SCRAM-SHA-256"]["storedKey"];
        assert_ne!(user_stored_key, user_cycled_key);
    }
}
