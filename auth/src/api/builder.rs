use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::FromRef,
    middleware::from_extractor,
    routing::{get, post, put},
    Router, Server,
};
use axum_sessions::{async_session::MemoryStore, SessionLayer};
use rand::RngCore;
use shuttle_common::{
    backends::metrics::{Metrics, TraceLayer},
    request_span,
};
use sqlx::SqlitePool;
use tracing::field;

use crate::{
    secrets::{EdDsaManager, KeyManager},
    user::{UserManagement, UserManager},
    COOKIE_EXPIRATION,
};

use super::handlers::{
    convert_cookie, convert_key, get_public_key, get_user, logout, post_user, put_user_reset_key,
    refresh_token,
};

pub type UserManagerState = Arc<Box<dyn UserManagement>>;
pub type KeyManagerState = Arc<Box<dyn KeyManager>>;

#[derive(Clone)]
pub struct RouterState {
    pub user_manager: UserManagerState,
    pub key_manager: KeyManagerState,
}

// Allow getting a user management state directly
impl FromRef<RouterState> for UserManagerState {
    fn from_ref(router_state: &RouterState) -> Self {
        router_state.user_manager.clone()
    }
}

// Allow getting a key manager state directly
impl FromRef<RouterState> for KeyManagerState {
    fn from_ref(router_state: &RouterState) -> Self {
        router_state.key_manager.clone()
    }
}

pub struct ApiBuilder {
    router: Router<RouterState>,
    pool: Option<SqlitePool>,
    session_layer: Option<SessionLayer<MemoryStore>>,
}

impl Default for ApiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiBuilder {
    pub fn new() -> Self {
        let router = Router::new()
            .route("/logout", post(logout))
            .route("/auth/session", get(convert_cookie))
            .route("/auth/key", get(convert_key))
            .route("/auth/refresh", post(refresh_token))
            .route("/public-key", get(get_public_key))
            .route("/users/:account_name", get(get_user))
            .route("/users/:account_name/:account_tier", post(post_user))
            .route("/users/reset-api-key", put(put_user_reset_key))
            .route_layer(from_extractor::<Metrics>())
            .layer(
                TraceLayer::new(|request| {
                    request_span!(
                        request,
                        request.params.account_name = field::Empty,
                        request.params.account_tier = field::Empty
                    )
                })
                .with_propagation()
                .build(),
            );

        Self {
            router,
            pool: None,
            session_layer: None,
        }
    }

    pub fn with_sqlite_pool(mut self, pool: SqlitePool) -> Self {
        self.pool = Some(pool);
        self
    }

    pub fn with_sessions(mut self) -> Self {
        let store = MemoryStore::new();
        let mut secret = [0u8; 128];
        rand::thread_rng().fill_bytes(&mut secret[..]);
        self.session_layer = Some(
            SessionLayer::new(store, &secret)
                .with_cookie_name("shuttle.sid")
                .with_session_ttl(Some(COOKIE_EXPIRATION))
                .with_secure(true),
        );

        self
    }

    pub fn into_router(self) -> Router {
        let pool = self.pool.expect("an sqlite pool is required");
        let session_layer = self.session_layer.expect("a session layer is required");

        let user_manager = UserManager { pool };
        let key_manager = EdDsaManager::new();

        let state = RouterState {
            user_manager: Arc::new(Box::new(user_manager)),
            key_manager: Arc::new(Box::new(key_manager)),
        };

        self.router.layer(session_layer).with_state(state)
    }
}

pub async fn serve(router: Router, address: SocketAddr) {
    Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address: {}", address));
}
