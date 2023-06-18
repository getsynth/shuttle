use std::path::PathBuf;

use clap::Parser;
use shuttle_common::backends::tracing::setup_tracing;
use shuttle_deployer::args::Args;
use shuttle_deployer::handlers::RouterBuilder;
use tracing::trace;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    trace!(args = ?args, "parsed args");
    setup_tracing(tracing_subscriber::registry(), "deployer");

    // Configure the deployer router.
    let mut router_builder = RouterBuilder::new(&args.auth_uri).await;
    if args.local {
        router_builder = router_builder.with_local_admin_layer();
    }

    let iapb = PathBuf::from(args.image_archive_path);
    router_builder = router_builder.with_mocked_builder_image_archive_path(iapb);

    let router = router_builder.into_router();
    axum::Server::bind(&args.api_address)
        .serve(router.into_make_service())
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address: {}", args.api_address));
}
