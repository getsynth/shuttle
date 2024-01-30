use std::net::Ipv4Addr;

use axum::{
    headers::{authorization::Bearer, Authorization, Header, HeaderMapExt},
    http::Request,
    middleware::Next,
    response::Response,
    Extension,
};
use hyper::{
    client::{connect::dns::GaiResolver, HttpConnector},
    Body, Client, StatusCode, Uri,
};
use hyper_reverse_proxy::ReverseProxy;
use once_cell::sync::Lazy;
use serde_json::Value;
use tracing::error;

static PROXY_CLIENT: Lazy<ReverseProxy<HttpConnector<GaiResolver>>> =
    Lazy::new(|| ReverseProxy::new(Client::new()));

/// This middleware proxies a request to the auth service to get a JWT, which we need to access
/// the deployer endpoints, and we'll also need it in the claim layer of the provisioner and runtime
/// clients.
///
/// Follow the steps in https://github.com/shuttle-hq/shuttle/blob/main/CONTRIBUTING.md#testing-deployer-only
/// to learn how to insert an admin user in the auth state.
///
/// WARNING: do not set this layer in production.
pub async fn set_jwt_bearer<B>(
    Extension(auth_uri): Extension<Uri>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    if let Some(bearer) = request.headers().typed_get::<Authorization<Bearer>>() {
        let token_request = make_token_request("/auth/key", bearer);

        let response = PROXY_CLIENT
            .call(
                Ipv4Addr::LOCALHOST.into(),
                &auth_uri.to_string(),
                token_request,
            )
            .await
            .expect("failed to proxy request to auth service");

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let convert: Value = serde_json::from_slice(&body)
            .expect("failed to deserialize body as JSON, did you login?");

        let token = convert["token"]
            .as_str()
            .expect("response body should have a token");

        request
            .headers_mut()
            .typed_insert(Authorization::bearer(token).expect("to set JWT token"));

        let response = next.run(request).await;

        Ok(response)
    } else {
        error!("No api-key bearer token found, make sure you are logged in.");
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn make_token_request(uri: &str, header: impl Header) -> Request<Body> {
    let mut token_request = Request::builder().uri(uri);
    token_request
        .headers_mut()
        .expect("manual request to be valid")
        .typed_insert(header);

    token_request
        .body(Body::empty())
        .expect("manual request to be valid")
}
