use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, BoxBody};
use axum::extract::{Extension, Path};
use axum::http::Request;
use axum::response::Response;
use axum::routing::{any, get, post};
use axum::{Json as AxumJson, Router};
use http::StatusCode;
use instant_acme::AccountCredentials;
use serde::{Deserialize, Serialize};
use shuttle_common::models::error::ErrorKind;
use shuttle_common::models::{project, user};
use tokio::sync::mpsc::Sender;
use tower_http::trace::TraceLayer;
use tracing::{debug, debug_span, field, Span};

use crate::auth::{Admin, ScopedUser, User};
use crate::custom_domain::AcmeClient;
use crate::task::{self, BoxedTask};
use crate::worker::WORKER_QUEUE_SIZE;
use crate::{AccountName, Error, Fqdn, GatewayService, ProjectName};

pub const SVC_DEGRADED_THRESHOLD: usize = 128;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GatewayStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    status: GatewayStatus,
}

impl StatusResponse {
    pub fn healthy() -> Self {
        Self {
            status: GatewayStatus::Healthy,
        }
    }

    pub fn degraded() -> Self {
        Self {
            status: GatewayStatus::Degraded,
        }
    }

    pub fn unhealthy() -> Self {
        Self {
            status: GatewayStatus::Unhealthy,
        }
    }
}

async fn get_user(
    Extension(service): Extension<Arc<GatewayService>>,
    Path(account_name): Path<AccountName>,
    _: Admin,
) -> Result<AxumJson<user::Response>, Error> {
    let user = User::retrieve_from_account_name(&service, account_name).await?;

    Ok(AxumJson(user.into()))
}

async fn post_user(
    Extension(service): Extension<Arc<GatewayService>>,
    Path(account_name): Path<AccountName>,
    _: Admin,
) -> Result<AxumJson<user::Response>, Error> {
    let user = service.create_user(account_name).await?;

    Ok(AxumJson(user.into()))
}

async fn get_project(
    Extension(service): Extension<Arc<GatewayService>>,
    ScopedUser { scope, .. }: ScopedUser,
) -> Result<AxumJson<project::Response>, Error> {
    let state = service.find_project(&scope).await?.into();
    let response = project::Response {
        name: scope.to_string(),
        state,
    };

    Ok(AxumJson(response))
}

async fn post_project(
    Extension(service): Extension<Arc<GatewayService>>,
    Extension(sender): Extension<Sender<BoxedTask>>,
    User { name, .. }: User,
    Path(project): Path<ProjectName>,
) -> Result<AxumJson<project::Response>, Error> {
    let state = service
        .create_project(project.clone(), name.clone())
        .await?;

    service
        .new_task()
        .project(project.clone())
        .account(name.clone())
        .send(&sender)
        .await?;

    let response = project::Response {
        name: project.to_string(),
        state: state.into(),
    };

    Ok(AxumJson(response))
}

async fn delete_project(
    Extension(service): Extension<Arc<GatewayService>>,
    Extension(sender): Extension<Sender<BoxedTask>>,
    ScopedUser {
        scope: _,
        user: User { name, .. },
    }: ScopedUser,
    Path(project): Path<ProjectName>,
) -> Result<AxumJson<project::Response>, Error> {
    let project_name = project.clone();

    let state = service.find_project(&project_name).await?;

    let mut response = project::Response {
        name: project_name.to_string(),
        state: state.into(),
    };

    if response.state == shuttle_common::models::project::State::Destroyed {
        return Ok(AxumJson(response));
    }

    // if project exists and isn't `Destroyed`, send destroy task
    service
        .new_task()
        .project(project)
        .account(name)
        .and_then(task::destroy())
        .send(&sender)
        .await?;

    response.state = shuttle_common::models::project::State::Destroying;

    Ok(AxumJson(response))
}

async fn route_project(
    Extension(service): Extension<Arc<GatewayService>>,
    ScopedUser { scope, .. }: ScopedUser,
    req: Request<Body>,
) -> Result<Response<Body>, Error> {
    service.route(&scope, req).await
}

async fn get_status(Extension(sender): Extension<Sender<BoxedTask>>) -> Response<Body> {
    let (status, body) = if sender.is_closed() || sender.capacity() == 0 {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusResponse::unhealthy(),
        )
    } else if sender.capacity() < WORKER_QUEUE_SIZE - SVC_DEGRADED_THRESHOLD {
        (StatusCode::OK, StatusResponse::degraded())
    } else {
        (StatusCode::OK, StatusResponse::healthy())
    };

    let body = serde_json::to_vec(&body).unwrap();
    Response::builder()
        .status(status)
        .body(body.into())
        .unwrap()
}

async fn revive_projects(
    _: Admin,
    Extension(service): Extension<Arc<GatewayService>>,
    Extension(sender): Extension<Sender<BoxedTask>>,
) -> Result<(), Error> {
    crate::project::exec::revive(service, sender)
        .await
        .map_err(|_| Error::from_kind(ErrorKind::Internal))
}

async fn create_acme_account(
    _: Admin,
    Extension(acme_client): Extension<AcmeClient>,
    Path(email): Path<String>,
) -> Result<AxumJson<serde_json::Value>, Error> {
    let res = acme_client.create_account(&email).await?;

    Ok(AxumJson(res))
}

async fn request_acme_certificate(
    _: Admin,
    Extension(service): Extension<Arc<GatewayService>>,
    Extension(acme_client): Extension<AcmeClient>,
    Path((project_name, fqdn)): Path<(ProjectName, Fqdn)>,
    AxumJson(credentials): AxumJson<AccountCredentials<'_>>,
) -> Result<String, Error> {
    let (chain, async_keys) = acme_client.create_certificate(&fqdn, credentials).await?;
    let private_key = async_keys.serialize_private_key_pem();

    service
        .create_custom_domain(project_name, fqdn, &chain, &private_key)
        .await?;

    Ok("Certificate created".to_string())
}

pub fn make_api(
    service: Arc<GatewayService>,
    acme_client: AcmeClient,
    sender: Sender<BoxedTask>,
) -> Router<Body> {
    debug!("making api route");

    Router::<Body>::new()
        .route(
            "/",
            get(get_status)
        )
        .route(
            "/projects/:project",
            get(get_project).delete(delete_project).post(post_project)
        )
        .route("/users/:account_name", get(get_user).post(post_user))
        .route("/projects/:project/*any", any(route_project))
        .route("/admin/revive", post(revive_projects))
        .route("/admin/acme/:email", post(create_acme_account))
        .route("/admin/acme/request/:project_name/:fqdn", post(request_acme_certificate))
        .layer(Extension(service))
        .layer(Extension(acme_client))
        .layer(Extension(sender))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    debug_span!("request", http.uri = %request.uri(), http.method = %request.method(), http.status_code = field::Empty, account.name = field::Empty, account.project = field::Empty)
                })
                .on_response(
                    |response: &Response<BoxBody>, latency: Duration, span: &Span| {
                        span.record("http.status_code", response.status().as_u16());
                        debug!(latency = format_args!("{} ns", latency.as_nanos()), "finished processing request");
                    },
                ),
        )
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::headers::Authorization;
    use axum::http::Request;
    use futures::TryFutureExt;
    use hyper::StatusCode;
    use tokio::sync::mpsc::channel;
    use tokio::sync::oneshot;
    use tower::Service;

    use super::*;
    use crate::service::GatewayService;
    use crate::tests::{RequestBuilderExt, World};

    #[tokio::test]
    async fn api_create_get_delete_projects() -> anyhow::Result<()> {
        let world = World::new().await;
        let service = Arc::new(GatewayService::init(world.args(), world.pool()).await);

        let (sender, mut receiver) = channel::<BoxedTask>(256);
        tokio::spawn(async move {
            while receiver.recv().await.is_some() {
                // do not do any work with inbound requests
            }
        });

        let mut router = make_api(Arc::clone(&service), sender);

        let neo = service.create_user("neo".parse().unwrap()).await?;

        let create_project = |project: &str| {
            Request::builder()
                .method("POST")
                .uri(format!("/projects/{project}"))
                .body(Body::empty())
                .unwrap()
        };

        let delete_project = |project: &str| {
            Request::builder()
                .method("DELETE")
                .uri(format!("/projects/{project}"))
                .body(Body::empty())
                .unwrap()
        };

        router
            .call(create_project("matrix"))
            .map_ok(|resp| assert_eq!(resp.status(), StatusCode::UNAUTHORIZED))
            .await
            .unwrap();

        let authorization = Authorization::bearer(neo.key.as_str()).unwrap();

        router
            .call(create_project("matrix").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        router
            .call(create_project("matrix").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            })
            .await
            .unwrap();

        let get_project = |project| {
            Request::builder()
                .method("GET")
                .uri(format!("/projects/{project}"))
                .body(Body::empty())
                .unwrap()
        };

        router
            .call(get_project("matrix"))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            })
            .await
            .unwrap();

        router
            .call(get_project("matrix").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        router
            .call(delete_project("matrix").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        router
            .call(create_project("reloaded").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        let trinity = service.create_user("trinity".parse().unwrap()).await?;

        let authorization = Authorization::bearer(trinity.key.as_str()).unwrap();

        router
            .call(get_project("reloaded").with_header(&authorization))
            .map_ok(|resp| assert_eq!(resp.status(), StatusCode::NOT_FOUND))
            .await
            .unwrap();

        router
            .call(delete_project("reloaded").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            })
            .await
            .unwrap();

        service
            .set_super_user(&"trinity".parse().unwrap(), true)
            .await?;

        router
            .call(get_project("reloaded").with_header(&authorization))
            .map_ok(|resp| assert_eq!(resp.status(), StatusCode::OK))
            .await
            .unwrap();

        router
            .call(delete_project("reloaded").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        // delete returns 404 for project that doesn't exist
        router
            .call(delete_project("resurrections").with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            })
            .await
            .unwrap();

        Ok(())
    }

    #[tokio::test]
    async fn api_create_get_users() -> anyhow::Result<()> {
        let world = World::new().await;
        let service = Arc::new(GatewayService::init(world.args(), world.pool()).await);

        let (sender, mut receiver) = channel::<BoxedTask>(256);
        tokio::spawn(async move {
            while receiver.recv().await.is_some() {
                // do not do any work with inbound requests
            }
        });

        let mut router = make_api(Arc::clone(&service), sender);

        let get_neo = || {
            Request::builder()
                .method("GET")
                .uri("/users/neo")
                .body(Body::empty())
                .unwrap()
        };

        let post_trinity = || {
            Request::builder()
                .method("POST")
                .uri("/users/trinity")
                .body(Body::empty())
                .unwrap()
        };

        router
            .call(get_neo())
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            })
            .await
            .unwrap();

        let user = service.create_user("neo".parse().unwrap()).await?;

        router
            .call(get_neo())
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            })
            .await
            .unwrap();

        let authorization = Authorization::bearer(user.key.as_str()).unwrap();

        router
            .call(get_neo().with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::FORBIDDEN);
            })
            .await
            .unwrap();

        router
            .call(post_trinity().with_header(&authorization))
            .map_ok(|resp| assert_eq!(resp.status(), StatusCode::FORBIDDEN))
            .await
            .unwrap();

        service.set_super_user(&user.name, true).await?;

        router
            .call(get_neo().with_header(&authorization))
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        router
            .call(post_trinity().with_header(&authorization))
            .map_ok(|resp| assert_eq!(resp.status(), StatusCode::OK))
            .await
            .unwrap();

        router
            .call(post_trinity().with_header(&authorization))
            .map_ok(|resp| assert_eq!(resp.status(), StatusCode::BAD_REQUEST))
            .await
            .unwrap();

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn status() {
        let world = World::new().await;
        let service = Arc::new(GatewayService::init(world.args(), world.pool()).await);

        let (sender, mut receiver) = channel::<BoxedTask>(1);
        let (ctl_send, ctl_recv) = oneshot::channel();
        let (done_send, done_recv) = oneshot::channel();
        let worker = tokio::spawn(async move {
            let mut done_send = Some(done_send);
            // do not process until instructed
            ctl_recv.await.unwrap();

            while receiver.recv().await.is_some() {
                done_send.take().unwrap().send(()).unwrap();
                // do nothing
            }
        });

        let mut router = make_api(Arc::clone(&service), sender.clone());

        let get_status = || {
            Request::builder()
                .method("GET")
                .uri("/")
                .body(Body::empty())
                .unwrap()
        };

        let resp = router.call(get_status()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let neo: AccountName = "neo".parse().unwrap();
        let matrix: ProjectName = "matrix".parse().unwrap();

        let neo = service.create_user(neo).await.unwrap();
        let authorization = Authorization::bearer(neo.key.as_str()).unwrap();

        let create_project = Request::builder()
            .method("POST")
            .uri(format!("/projects/{matrix}"))
            .body(Body::empty())
            .unwrap()
            .with_header(&authorization);

        router.call(create_project).await.unwrap();

        let resp = router.call(get_status()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

        ctl_send.send(()).unwrap();
        done_recv.await.unwrap();

        let resp = router.call(get_status()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        worker.abort();
        let _ = worker.await;

        let resp = router.call(get_status()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
