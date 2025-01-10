use std::{convert::Infallible, fmt::Debug, net::Ipv4Addr, sync::Arc, time::Duration};

use axum::{
    body::{boxed, HttpBody},
    headers::{authorization::Bearer, Authorization, Header, HeaderMapExt},
    response::Response,
};
use futures::future::BoxFuture;
use http::{Request, StatusCode, Uri};
use hyper::{
    client::{connect::dns::GaiResolver, HttpConnector},
    Body, Client,
};
use hyper_reverse_proxy::ReverseProxy;
use once_cell::sync::Lazy;
use opentelemetry::global;
use opentelemetry_http::HeaderInjector;
use shuttle_backends::{
    auth::ConvertResponse, cache::CacheManagement, headers::XShuttleAdminSecret, key::ApiKey,
};
use tower::{Layer, Service};
use tracing::{error, trace, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

static PROXY_CLIENT: Lazy<ReverseProxy<HttpConnector<GaiResolver>>> =
    Lazy::new(|| ReverseProxy::new(Client::new()));

/// Time to cache tokens for. Currently tokens take 15 minutes to expire (see [EXP_MINUTES]) which leaves a 10 minutes
/// buffer (EXP_MINUTES - CACHE_MINUTES). We want the buffer to be atleast as long as the longest builds which has
/// been observed to be around 5 minutes.
const CACHE_MINUTES: u64 = 5;

/// The idea of this layer is to do two things:
/// 1. Forward all user related routes (`/users/*`) to our auth service
/// 2. Upgrade all Authorization Bearer keys to JWT tokens for internal communication inside and below gateway, fetching
///    the JWT token from a ttl-cache if it isn't expired, and inserting it in the cache if it isn't there.
#[derive(Clone)]
pub struct ShuttleAuthLayer {
    auth_uri: Uri,
    gateway_admin_key: String,
    cache_manager: Arc<Box<dyn CacheManagement<Value = String>>>,
}

impl ShuttleAuthLayer {
    pub fn new(
        auth_uri: Uri,
        gateway_admin_key: String,
        cache_manager: Arc<Box<dyn CacheManagement<Value = String>>>,
    ) -> Self {
        Self {
            auth_uri,
            gateway_admin_key,
            cache_manager,
        }
    }
}

impl<S> Layer<S> for ShuttleAuthLayer {
    type Service = ShuttleAuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ShuttleAuthService {
            inner,
            gateway_admin_key: self.gateway_admin_key.clone(),
            auth_uri: self.auth_uri.clone(),
            cache_manager: self.cache_manager.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ShuttleAuthService<S> {
    inner: S,
    auth_uri: Uri,
    gateway_admin_key: String,
    cache_manager: Arc<Box<dyn CacheManagement<Value = String>>>,
}

impl<S> Service<Request<Body>> for ShuttleAuthService<S>
where
    S: Service<Request<Body>, Response = Response> + Send + Clone + 'static,
    S::Error: Debug,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.inner.poll_ready(cx) {
            std::task::Poll::Ready(_) => std::task::Poll::Ready(Ok(())),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let req_path = req.uri().path();

        // Pass through status page
        if req_path == "/" {
            let future = self.inner.call(req);

            return Box::pin(async move {
                match future.await {
                    Ok(response) => Ok(response),
                    Err(_) => {
                        error!("unexpected internal error from gateway");

                        Ok(Response::builder()
                            .status(StatusCode::SERVICE_UNAVAILABLE)
                            .body(boxed(Body::empty()))
                            .unwrap())
                    }
                }
            });
        }

        // If /users/reset-api-key is called, invalidate the cached JWT.
        if req_path == "/users/reset-api-key" {
            if let Some((cache_key, _)) = cache_key_and_token_req(
                req.headers().typed_get::<Authorization<Bearer>>(),
                self.gateway_admin_key.as_str(),
            ) {
                self.cache_manager.invalidate(&cache_key);
            };
        }

        if req_path.starts_with("/users") {
            let target_url = self.auth_uri.to_string();

            let cx = Span::current().context();

            global::get_text_map_propagator(|propagator| {
                propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut()))
            });

            Box::pin(async move {
                let response = PROXY_CLIENT
                    .call(Ipv4Addr::LOCALHOST.into(), &target_url, req)
                    .await;

                match response {
                    Ok(res) => {
                        let (parts, body) = res.into_parts();
                        let body =
                            <Body as HttpBody>::map_err(body, axum::Error::new).boxed_unsync();

                        Ok(Response::from_parts(parts, body))
                    }
                    Err(error) => {
                        error!(?error, "failed to call authentication service");

                        Ok(Response::builder()
                            .status(StatusCode::SERVICE_UNAVAILABLE)
                            .body(boxed(Body::empty()))
                            .unwrap())
                    }
                }
            })
        } else {
            // Enrich the current key | session

            // TODO: read this page to get rid of this clone
            // https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md
            let mut this = self.clone();

            let bearer = req.headers().typed_get::<Authorization<Bearer>>();

            // If this is not a valid api key, then assume it must be a JWT token key. Therefore use it as is
            if !bearer
                .clone()
                .map(|key| ApiKey::parse(key.token()).is_ok())
                .unwrap_or_default()
            {
                return Box::pin(async move {
                    match this.inner.call(req).await {
                        Ok(response) => Ok(response),
                        Err(error) => {
                            error!(?error, "unexpected internal error from gateway");

                            Ok(Response::builder()
                                .status(StatusCode::SERVICE_UNAVAILABLE)
                                .body(boxed(Body::empty()))
                                .unwrap())
                        }
                    }
                });
            }

            Box::pin(async move {
                // Only if there is something to upgrade
                if let Some((cache_key, token_request)) =
                    cache_key_and_token_req(bearer, this.gateway_admin_key.as_str())
                {
                    let target_url = this.auth_uri.to_string();

                    // Check if the token is cached.
                    if let Some(token) = this.cache_manager.get(&cache_key) {
                        trace!("JWT cache hit, setting token from cache on request");

                        // Token is cached and not expired, return it in the response.
                        req.headers_mut()
                            .typed_insert(Authorization::bearer(&token).unwrap());
                    } else {
                        trace!("JWT cache missed, sending convert token request");

                        // Token is not in the cache, send a convert request.
                        let token_response = match PROXY_CLIENT
                            .call(Ipv4Addr::LOCALHOST.into(), &target_url, token_request)
                            .await
                        {
                            Ok(res) => res,
                            Err(error) => {
                                error!(?error, "failed to call authentication service");

                                return Ok(Response::builder()
                                    .status(StatusCode::SERVICE_UNAVAILABLE)
                                    .body(boxed(Body::empty()))
                                    .unwrap());
                            }
                        };

                        // Bubble up auth errors
                        if token_response.status() != StatusCode::OK {
                            let (parts, body) = token_response.into_parts();
                            let body = body.map_err(axum::Error::new).boxed_unsync();

                            return Ok(Response::from_parts(parts, body));
                        }

                        let body = match hyper::body::to_bytes(token_response.into_body()).await {
                            Ok(body) => body,
                            Err(error) => {
                                error!(
                                    error = &error as &dyn std::error::Error,
                                    "failed to get response body"
                                );

                                return Ok(Response::builder()
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .body(boxed(Body::empty()))
                                    .unwrap());
                            }
                        };

                        let response = match serde_json::from_slice::<ConvertResponse>(&body) {
                            Ok(response) => response,
                            Err(error) => {
                                error!(
                                    error = &error as &dyn std::error::Error,
                                    "failed to convert body to ConvertResponse"
                                );

                                return Ok(Response::builder()
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .body(boxed(Body::empty()))
                                    .unwrap());
                            }
                        };

                        let bearer = Authorization::bearer(&response.token).expect("bearer token");

                        this.cache_manager.insert(
                            cache_key.as_str(),
                            response.token,
                            Duration::from_secs(CACHE_MINUTES * 60),
                        );

                        trace!("token inserted in cache, request proceeding");
                        req.headers_mut().typed_insert(bearer);
                    };
                }

                match this.inner.call(req).await {
                    Ok(response) => Ok(response),
                    Err(error) => {
                        error!(?error, "unexpected internal error from gateway");

                        Ok(Response::builder()
                            .status(StatusCode::SERVICE_UNAVAILABLE)
                            .body(boxed(Body::empty()))
                            .unwrap())
                    }
                }
            })
        }
    }
}

fn cache_key_and_token_req(
    bearer: Option<Authorization<Bearer>>,
    gateway_admin_key: &str,
) -> Option<(String, Request<Body>)> {
    bearer.map(|bearer| {
        let cache_key = bearer.token().trim().to_string();
        let token_request = make_token_request("/auth/key", bearer, Some(gateway_admin_key));
        (cache_key, token_request)
    })
}

fn make_token_request(
    uri: &str,
    header: impl Header,
    gateway_admin_key: Option<&str>,
) -> Request<Body> {
    let mut token_request = Request::builder().uri(uri);
    token_request
        .headers_mut()
        .expect("manual request to be valid")
        .typed_insert(header);

    if let Some(key) = gateway_admin_key {
        trace!("XShuttleAdminSecret header inserted for token request");
        token_request
            .headers_mut()
            .expect("manual request to be valid")
            .typed_insert(XShuttleAdminSecret(key.to_string()));
    }

    let cx = Span::current().context();
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &cx,
            &mut HeaderInjector(token_request.headers_mut().expect("request to be valid")),
        )
    });

    token_request
        .body(Body::empty())
        .expect("manual request to be valid")
}
