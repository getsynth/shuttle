use std::process::exit;

use clap::Parser;
use shuttle_common::{
    backends::tracing::setup_tracing,
    claims::{ClaimLayer, InjectPropagationLayer},
};
use shuttle_deployer::{start, start_proxy, Args, DeployLayer, Persistence, RuntimeManager};
use shuttle_proto::logger::logger_client::LoggerClient;
use tokio::select;
use tower::ServiceBuilder;
use tracing::{error, trace};
use tracing_subscriber::prelude::*;
use ulid::Ulid;

// The `multi_thread` is needed to prevent a deadlock in shuttle_service::loader::build_crate() which spawns two threads
// Without this, both threads just don't start up
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    trace!(args = ?args, "parsed args");

    let (persistence, _) = Persistence::new(
        &args.state,
        &args.resource_recorder,
        Ulid::from_string(args.project_id.as_str())
            .expect("to get a valid ULID for project_id arg"),
    )
    .await;
    setup_tracing(
        tracing_subscriber::registry().with(DeployLayer::new(persistence.clone())),
        "deployer",
    );

    let channel = args
        .logger_uri
        .connect()
        .await
        .expect("failed to connect to logger");

    let channel = ServiceBuilder::new()
        .layer(ClaimLayer)
        .layer(InjectPropagationLayer)
        .service(channel);

    let logger_client = LoggerClient::new(channel);

    let runtime_manager = RuntimeManager::new(
        args.artifacts_path.clone(),
        args.provisioner_address.uri().to_string(),
        args.logger_uri.uri().to_string(),
        logger_client.clone(),
        Some(args.auth_uri.to_string()),
    );

    select! {
        _ = start_proxy(args.proxy_address, args.proxy_fqdn.clone(), persistence.clone()) => {
            error!("Proxy stopped.")
        },
        _ = start(persistence, runtime_manager, logger_client, args) => {
            error!("Deployment service stopped.")
        },
    }

    exit(1);
}
