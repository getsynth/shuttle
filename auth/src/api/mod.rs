use std::net::SocketAddr;

use axum::{Router, Server};

mod builder;
mod handlers;

pub(crate) use builder::ApiBuilder;

pub(crate) async fn serve(router: Router, address: &SocketAddr) {
    Server::bind(address)
        .serve(router.into_make_service())
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address: {}", address));
}
