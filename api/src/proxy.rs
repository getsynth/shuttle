use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use ::hyper::server::conn::AddrStream;
use ::hyper::server::Server;
use ::hyper::service::{make_service_fn, service_fn};
use ::hyper::{Body, Request, Response, StatusCode};
use hyper::client::connect::dns::GaiResolver;
use hyper::client::HttpConnector;
use hyper::header::{HeaderValue, SERVER};
use hyper::Client;
use hyper_reverse_proxy::{ProxyError, ReverseProxy};
use lazy_static::lazy_static;
use shuttle_common::Port;

use crate::DeploymentSystem;

lazy_static! {
    static ref HEADER_SERVER: HeaderValue = "shuttle.rs".parse().unwrap();
    static ref PROXY_CLIENT: ReverseProxy<HttpConnector<GaiResolver>> =
        ReverseProxy::new(Client::new());
}

pub(crate) async fn start(
    bind_addr: IpAddr,
    proxy_port: Port,
    deployment_manager: Arc<DeploymentSystem>,
) {
    let socket_address = (bind_addr, proxy_port).into();

    // A `Service` is needed for every connection.
    let make_svc = make_service_fn(|socket: &AddrStream| {
        let dm_ref = deployment_manager.clone();
        let remote_addr = socket.remote_addr();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle(remote_addr, req, dm_ref.clone())
            }))
        }
    });

    let server = Server::bind(&socket_address).serve(make_svc);

    log::debug!("starting proxy server: {}", &socket_address);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
        eprintln!("proxy died, killing process...");
        std::process::exit(1);
    }
}

async fn handle(
    remote_addr: SocketAddr,
    req: Request<Body>,
    deployment_manager: Arc<DeploymentSystem>,
) -> Result<Response<Body>, Infallible> {
    // if no `Host:` or invalid value, return 400
    let host = match req.headers().get("Host") {
        Some(host) if host.to_str().is_ok() => host.to_str().unwrap().to_owned(),
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap())
        }
    };

    // if we could not get a port from the deployment manager,
    // the host does not exist or is not initialised yet - so
    // we return a 404
    let port = match deployment_manager.port_for_host(&host).await {
        None => {
            // no port being assigned here means that we couldn't
            // find a service for a given host
            let response_body = format!("could not find service for host: {}", host);
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(response_body.into())
                .unwrap());
        }
        Some(port) => port,
    };

    match reverse_proxy(remote_addr.ip(), port, req).await {
        Ok(response) => {
            info!("[PROXY, {}]", &host);
            Ok(response)
        }
        Err(error) => {
            match error {
                ProxyError::InvalidUri(e) => {
                    log::warn!("error while handling request in reverse proxy: {}", e);
                }
                ProxyError::HyperError(e) => {
                    log::warn!("error while handling request in reverse proxy: {}", e);
                }
                ProxyError::ForwardHeaderError => {
                    log::warn!("error while handling request in reverse proxy: 'fwd header error'");
                }
                ProxyError::UpgradeError(e) => log::warn!(
                    "error while handling request needing upgrade in reverse proxy: {}",
                    e
                ),
            };
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap())
        }
    }
}

async fn reverse_proxy(
    ip: IpAddr,
    port: Port,
    req: Request<Body>,
) -> Result<Response<Body>, ProxyError> {
    let forward_uri = format!("http://127.0.0.1:{}", port);
    let mut response = PROXY_CLIENT.call(ip, &forward_uri, req).await?;

    response.headers_mut().insert(SERVER, HEADER_SERVER.clone());

    Ok(response)
}
