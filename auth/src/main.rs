use std::time::Duration;

use clap::Parser;
use shuttle_common::{
    backends::tracing::{setup_tracing, ExtractPropagationLayer},
    ApiKey,
};
use shuttle_proto::auth::auth_server::AuthServer;
use tonic::transport::Server;
use tracing::trace;

use shuttle_auth::{AccountTier, Args, Commands, Dal, EdDsaManager, Service, SessionLayer, Sqlite};

#[tokio::main]
async fn main() {
    let args = Args::parse();

    trace!(args = ?args, "parsed args");

    setup_tracing(tracing_subscriber::registry(), "auth");

    let db_path = args.state.join("auth.sqlite");

    let sqlite = Sqlite::new(db_path.to_str().unwrap()).await;

    match args.command {
        Commands::Start(args) => {
            let random = ring::rand::SystemRandom::new();
            let key_manager = EdDsaManager::new(random.clone());

            let mut server_builder = Server::builder()
                .http2_keepalive_interval(Some(Duration::from_secs(60)))
                .layer(SessionLayer::new(sqlite.clone(), key_manager.clone()))
                .layer(ExtractPropagationLayer);

            let svc = Service::new(sqlite, key_manager, random);
            let svc = AuthServer::new(svc);
            let router = server_builder.add_service(svc);

            router.serve(args.address).await.unwrap();
        }
        Commands::Init(args) => {
            let key = args
                .key
                .map_or_else(ApiKey::generate, |key| ApiKey::parse(&key).unwrap());

            sqlite
                .create_user(args.name.into(), key, AccountTier::Admin)
                .await
                .unwrap();
        }
    }
}
