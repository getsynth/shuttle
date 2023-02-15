use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use tracing::instrument;

use crate::{
    api::builder::RouterState,
    error::Error,
    user::{User, UserManagement, UserName},
};

#[debug_handler]
#[instrument(skip(user_manager))]
pub(crate) async fn get_user(
    State(RouterState { user_manager }): State<RouterState>,
    Path(user_name): Path<UserName>,
) -> Result<Json<User>, Error> {
    let user = user_manager.get_user(user_name).await?;

    Ok(Json(user))
}

#[debug_handler]
#[instrument(skip(user_manager))]
pub(crate) async fn post_user(
    State(RouterState { user_manager }): State<RouterState>,
    Path(user_name): Path<UserName>,
) -> Result<Json<User>, Error> {
    let user = user_manager.create_user(user_name).await?;

    Ok(Json(user))
}

pub(crate) async fn login() {}

pub(crate) async fn logout() {}

pub(crate) async fn convert_cookie() {}

pub(crate) async fn convert_key() {}

pub(crate) async fn refresh_token() {}

pub(crate) async fn get_public_key() {}
