use std::{convert::Infallible, net::SocketAddr};

pub use args::Args;
pub use deployment::deploy_layer::DeployLayer;
use deployment::{Built, DeploymentManager};
use fqdn::FQDN;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
};
pub use persistence::Persistence;
use proxy::AddressGetter;
use shuttle_proto::runtime::runtime_client::RuntimeClient;
use tonic::transport::Channel;
use tracing::{error, info};

use crate::deployment::gateway_client::GatewayClient;

mod args;
mod deployment;
mod error;
mod handlers;
mod persistence;
mod proxy;

pub async fn start(persistence: Persistence, runtime_client: RuntimeClient<Channel>, args: Args) {
    let deployment_manager = DeploymentManager::builder()
        .build_log_recorder(persistence.clone())
        .secret_recorder(persistence.clone())
        .active_deployment_getter(persistence.clone())
        .artifacts_path(args.artifacts_path)
        .runtime(runtime_client)
        .secret_getter(persistence.clone())
        .queue_client(GatewayClient::new(args.gateway_uri))
        .build();

    persistence.cleanup_invalid_states().await.unwrap();

    let runnable_deployments = persistence.get_all_runnable_deployments().await.unwrap();
    info!(count = %runnable_deployments.len(), "enqueuing runnable deployments");
    for existing_deployment in runnable_deployments {
        let built = Built {
            id: existing_deployment.id,
            service_name: existing_deployment.service_name,
            service_id: existing_deployment.service_id,
            tracing_context: Default::default(),
            is_next: existing_deployment.is_next,
        };
        deployment_manager.run_push(built).await;
    }

    let router = handlers::make_router(
        persistence,
        deployment_manager,
        args.proxy_fqdn,
        args.admin_secret,
        args.project,
    );
    let make_service = router.into_make_service();

    info!(address=%args.api_address, "Binding to and listening at address");

    axum::Server::bind(&args.api_address)
        .serve(make_service)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address: {}", args.api_address));
}

pub async fn start_proxy(
    proxy_address: SocketAddr,
    fqdn: FQDN,
    address_getter: impl AddressGetter,
) {
    let make_service = make_service_fn(move |socket: &AddrStream| {
        let remote_address = socket.remote_addr();
        let address_getter = address_getter.clone();
        let fqdn = fqdn.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                proxy::handle(remote_address, fqdn.clone(), req, address_getter.clone())
            }))
        }
    });

    let server = hyper::Server::bind(&proxy_address).serve(make_service);

    info!("Starting proxy server on: {}", proxy_address);

    if let Err(e) = server.await {
        error!(error = %e, "proxy died, killing process...");
        std::process::exit(1);
    }
}
