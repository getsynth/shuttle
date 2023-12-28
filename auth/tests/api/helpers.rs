use axum::{body::Body, response::Response, Router};
use http::{header::CONTENT_TYPE, StatusCode};
use hyper::http::{header::AUTHORIZATION, Request};
use once_cell::sync::Lazy;
use serde_json::Value;
use shuttle_auth::{pgpool_init, ApiBuilder};
use shuttle_common::{
    backends::headers::X_SHUTTLE_ADMIN_SECRET,
    claims::{AccountTier, Claim},
};
use shuttle_common_tests::postgres::DockerInstance;
use sqlx::query;
use tower::ServiceExt;
use wiremock::{
    matchers::{bearer_token, method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Admin user API key.
pub(crate) const ADMIN_KEY: &str = "ndh9z58jttoes3qv";
/// Stripe test API key.
pub(crate) const STRIPE_TEST_KEY: &str = "sk_test_123";
/// Stripe test RDS price id.
pub(crate) const STRIPE_TEST_RDS_PRICE_ID: &str = "price_1OIS06FrN7EDaGOjaV0GXD7P";

static PG: Lazy<DockerInstance> = Lazy::new(DockerInstance::default);
#[ctor::dtor]
fn cleanup() {
    PG.cleanup();
}

pub(crate) struct TestApp {
    pub router: Router,
    pub mock_server: MockServer,
}

/// Initialize a router with an in-memory sqlite database for each test.
pub(crate) async fn app() -> TestApp {
    let pg_pool = pgpool_init(PG.get_unique_uri().as_str()).await.unwrap();

    let mock_server = MockServer::start().await;

    // Insert an admin user for the tests.
    query("INSERT INTO users (account_name, key, account_tier) VALUES ($1, $2, $3)")
        .bind("admin")
        .bind(ADMIN_KEY)
        .bind(AccountTier::Admin.to_string())
        .execute(&pg_pool)
        .await
        .unwrap();

    let router = ApiBuilder::new("LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1DNENBUUF3QlFZREsyVndCQ0lFSUR5V0ZFYzhKYm05NnA0ZGNLTEwvQWNvVUVsbUF0MVVKSTU4WTc4d1FpWk4KLS0tLS1FTkQgUFJJVkFURSBLRVktLS0tLQo=".to_string())
        .with_pg_pool(pg_pool)
        .with_sessions()
        .with_stripe_client(stripe::Client::from_url(
            mock_server.uri().as_str(),
            STRIPE_TEST_KEY,
        ))
        .with_rds_price_id(STRIPE_TEST_RDS_PRICE_ID.to_string())
        .into_router();

    TestApp {
        router,
        mock_server,
    }
}

impl TestApp {
    pub async fn send_request(&self, request: Request<Body>) -> Response {
        self.router
            .clone()
            .oneshot(request)
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_user(&self, name: &str, tier: &str) -> Response {
        let request = Request::builder()
            .uri(format!("/users/{name}/{tier}"))
            .method("POST")
            .header(AUTHORIZATION, format!("Bearer {ADMIN_KEY}"))
            .body(Body::empty())
            .unwrap();

        self.send_request(request).await
    }

    pub async fn put_user(
        &self,
        name: &str,
        tier: &str,
        checkout_session: &'static str,
    ) -> Response {
        let request = Request::builder()
            .uri(format!("/users/{name}/{tier}"))
            .method("PUT")
            .header(AUTHORIZATION, format!("Bearer {ADMIN_KEY}"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(checkout_session))
            .unwrap();

        self.send_request(request).await
    }

    pub async fn get_user(&self, name: &str) -> Response {
        let request = Request::builder()
            .uri(format!("/users/{name}"))
            .header(AUTHORIZATION, format!("Bearer {ADMIN_KEY}"))
            .body(Body::empty())
            .unwrap();

        self.send_request(request).await
    }

    /// If we don't provide a valid admin key, then the`user_api_key` parameter
    /// should be of an admin user.
    pub async fn get_jwt_from_api_key(
        &self,
        user_api_key: &str,
        admin_api_key: Option<&str>,
    ) -> Response {
        let mut request_builder = Request::builder()
            .uri("/auth/key")
            .header(AUTHORIZATION, format!("Bearer {user_api_key}"));

        if let Some(key) = admin_api_key {
            request_builder = request_builder.header(X_SHUTTLE_ADMIN_SECRET.to_string(), key)
        }

        let request = request_builder.body(Body::empty()).unwrap();
        self.send_request(request).await
    }

    pub async fn claim_from_response(&self, res: Response) -> Claim {
        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let convert: Value = serde_json::from_slice(&body).unwrap();
        let token = convert["token"].as_str().unwrap();

        let request = Request::builder()
            .uri("/public-key")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = self.send_request(request).await;

        assert_eq!(response.status(), StatusCode::OK);

        let public_key = hyper::body::to_bytes(response.into_body()).await.unwrap();

        Claim::from_token(token, &public_key).unwrap()
    }

    /// A test util to get a user with a subscription, mocking the response from Stripe. A key part
    /// of this util is `mount_as_scoped`, since getting a user with a subscription can be done
    /// several times in a test, if they're not scoped the first mock would always apply.
    pub async fn get_user_with_mocked_stripe(
        &self,
        subscription_id: &str,
        response_body: &str,
        account_name: &str,
    ) -> Response {
        // This mock will apply until the end of this function scope.
        let _mock_guard = Mock::given(method("GET"))
            .and(bearer_token(STRIPE_TEST_KEY))
            .and(path(format!("/v1/subscriptions/{subscription_id}")))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::from_str::<Value>(response_body).unwrap()),
            )
            .mount_as_scoped(&self.mock_server)
            .await;

        self.get_user(account_name).await
    }
}
