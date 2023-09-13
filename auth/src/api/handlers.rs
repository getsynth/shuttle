use crate::{
    error::Error,
    user::{AccountName, AccountTier, Admin, Key, User},
};
use axum::{
    extract::{Path, State},
    Json,
};
use axum_sessions::extractors::{ReadableSession, WritableSession};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use shuttle_common::{claims::Claim, models::user};
use tracing::instrument;

use super::{
    builder::{KeyManagerState, UserManagerState},
    RouterState,
};

#[instrument(skip(user_manager))]
pub(crate) async fn get_user(
    _: Admin,
    State(user_manager): State<UserManagerState>,
    Path(account_name): Path<AccountName>,
) -> Result<Json<user::Response>, Error> {
    let user = user_manager.get_user(account_name).await?;

    Ok(Json(user.into()))
}

#[instrument(skip(user_manager))]
pub(crate) async fn post_user(
    _: Admin,
    State(user_manager): State<UserManagerState>,
    Path((account_name, account_tier)): Path<(AccountName, AccountTier)>,
) -> Result<Json<user::Response>, Error> {
    let user = user_manager.create_user(account_name, account_tier).await?;

    Ok(Json(user.into()))
}

pub(crate) async fn put_user_reset_key(
    session: ReadableSession,
    State(user_manager): State<UserManagerState>,
    key: Option<Key>,
) -> Result<(), Error> {
    let account_name = match session.get::<String>("account_name") {
        Some(account_name) => account_name.into(),

        None => match key {
            Some(key) => user_manager.get_user_by_key(key.into()).await?.name,
            None => return Err(Error::Unauthorized),
        },
    };

    user_manager.reset_key(account_name).await
}

pub(crate) async fn logout(mut session: WritableSession) {
    session.destroy();
}

// Dummy health-check returning 200 if the auth server is up.
pub(crate) async fn health_check() -> Result<(), Error> {
    Ok(())
}

pub(crate) async fn convert_cookie(
    session: ReadableSession,
    State(key_manager): State<KeyManagerState>,
) -> Result<Json<shuttle_common::backends::auth::ConvertResponse>, StatusCode> {
    let account_name = session
        .get::<String>("account_name")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let account_tier = session
        .get::<AccountTier>("account_tier")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claim = Claim::new(account_name, account_tier.into());

    let token = claim.into_token(key_manager.private_key())?;

    let response = shuttle_common::backends::auth::ConvertResponse { token };

    Ok(Json(response))
}

/// Convert a valid API-key bearer token to a JWT.
pub(crate) async fn convert_key(
    State(RouterState {
        key_manager,
        user_manager,
    }): State<RouterState>,
    key: Key,
) -> Result<Json<shuttle_common::backends::auth::ConvertResponse>, StatusCode> {
    let User {
        name, account_tier, ..
    } = user_manager
        .get_user_by_key(key.into())
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let claim = Claim::new(name.to_string(), account_tier.into());

    let token = claim.into_token(key_manager.private_key())?;

    let response = shuttle_common::backends::auth::ConvertResponse { token };

    Ok(Json(response))
}

pub(crate) async fn refresh_token() {}

pub(crate) async fn get_public_key(State(key_manager): State<KeyManagerState>) -> Vec<u8> {
    key_manager.public_key().to_vec()
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    account_name: AccountName,
}
