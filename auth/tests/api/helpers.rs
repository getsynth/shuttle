use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use portpicker::pick_unused_port;
use shuttle_auth::{Dal, EdDsaManager, Service, SessionLayer, Sqlite};
use shuttle_common::{claims::AccountTier, ApiKey};
use shuttle_proto::auth::{
    auth_client::AuthClient, auth_server::AuthServer, NewUser, UserRequest, UserResponse,
};
use tonic::{
    metadata::MetadataValue,
    transport::{Channel, Server},
    Status,
};
use tonic::{Request, Response};

pub(crate) const ADMIN_KEY: &str = "ndh9z58jttoes3qv";

pub(crate) struct TestApp {
    pub client: AuthClient<Channel>,
}

/// Initialize a [AuthServer] with an in-memory sqlite database and spawn it in the background
/// for each test. Also initialize and return an [AuthClient].
pub(crate) async fn spawn_app() -> TestApp {
    let port = pick_unused_port().unwrap();
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);

    // Initialize an in-memory DB with an admin user.
    let sqlite = Sqlite::new_in_memory().await;

    let admin_key = ApiKey::parse(ADMIN_KEY).unwrap();

    sqlite
        .create_user("admin".to_string().into(), admin_key, AccountTier::Admin)
        .await
        .unwrap();

    let random = ring::rand::SystemRandom::new();
    let key_manager = EdDsaManager::new(random.clone());

    let mut server_builder = Server::builder()
        .http2_keepalive_interval(Some(Duration::from_secs(60)))
        .layer(SessionLayer::new(sqlite.clone(), key_manager.clone()));

    let svc = Service::new(sqlite, key_manager, random);
    let svc = AuthServer::new(svc);
    let router = server_builder.add_service(svc);

    // Spawn our server in the background.
    tokio::spawn(router.serve(addr));

    let client = AuthClient::connect(format!("http://localhost:{port}"))
        .await
        .unwrap();

    TestApp { client }
}

// Convenience methods for testing.
impl TestApp {
    pub async fn post_user(
        &mut self,
        name: &str,
        tier: &str,
    ) -> Result<Response<UserResponse>, Status> {
        let mut request = Request::new(NewUser {
            account_name: name.to_string(),
            account_tier: tier.to_string(),
        });

        // Insert admin bearer token in request metadata.
        let bearer: MetadataValue<_> = format!("Bearer {ADMIN_KEY}").parse().unwrap();
        request.metadata_mut().insert("authorization", bearer);

        self.client.post_user_request(request).await
    }

    pub async fn get_user(&mut self, name: &str) -> Result<Response<UserResponse>, Status> {
        let mut request = Request::new(UserRequest {
            account_name: name.to_string(),
        });

        // Insert admin bearer token in request metadata.
        let bearer: MetadataValue<_> = format!("Bearer {ADMIN_KEY}").parse().unwrap();
        request.metadata_mut().insert("authorization", bearer);

        self.client.get_user_request(request).await
    }
}
