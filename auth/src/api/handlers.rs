use crate::{
    error::Error,
    user::{AccountName, Admin, Key, User},
};
use axum::{
    extract::{Path, State},
    Json,
};
use http::StatusCode;
use shuttle_common::{
    claims::{AccountTier, Claim},
    models::user,
};
use stripe::CheckoutSession;
use tracing::instrument;

use super::{
    builder::{KeyManagerState, UserManagerState},
    RouterState,
};

#[instrument(skip_all, fields(account.name = %account_name))]
pub(crate) async fn get_user(
    _: Admin,
    State(user_manager): State<UserManagerState>,
    Path(account_name): Path<AccountName>,
) -> Result<Json<user::Response>, Error> {
    let user = user_manager.get_user(account_name).await?;

    Ok(Json(user.into()))
}

#[instrument(skip_all, fields(account.name = %account_name, account.tier = %account_tier))]
pub(crate) async fn post_user(
    _: Admin,
    State(user_manager): State<UserManagerState>,
    Path((account_name, account_tier)): Path<(AccountName, AccountTier)>,
) -> Result<Json<user::Response>, Error> {
    let user = user_manager.create_user(account_name, account_tier).await?;

    Ok(Json(user.into()))
}

#[instrument(skip(user_manager, account_name, account_tier), fields(account.name = %account_name, account.tier = %account_tier))]
pub(crate) async fn update_user_tier(
    _: Admin,
    State(user_manager): State<UserManagerState>,
    Path((account_name, account_tier)): Path<(AccountName, AccountTier)>,
    payload: Option<Json<CheckoutSession>>,
) -> Result<(), Error> {
    if account_tier == AccountTier::Pro {
        match payload {
            Some(Json(checkout_session)) => {
                user_manager
                    .upgrade_to_pro(&account_name, checkout_session)
                    .await?;
            }
            None => return Err(Error::MissingCheckoutSession),
        }
    } else {
        user_manager
            .update_tier(&account_name, account_tier)
            .await?;
    };

    Ok(())
}

pub(crate) async fn put_user_reset_key(
    State(user_manager): State<UserManagerState>,
    key: Key,
) -> Result<(), Error> {
    let account_name = user_manager.get_user_by_key(key.into()).await?.name;

    user_manager.reset_key(account_name).await
}

// Dummy health-check returning 200 if the auth server is up.
pub(crate) async fn health_check() -> Result<(), Error> {
    Ok(())
}

/// Convert a valid API-key bearer token to a JWT.
pub(crate) async fn convert_key(
    _: Admin,
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

    let claim = Claim::new(
        name.to_string(),
        account_tier.into(),
        account_tier,
        account_tier,
    );

    let token = claim.into_token(key_manager.private_key())?;

    let response = shuttle_common::backends::auth::ConvertResponse { token };

    Ok(Json(response))
}

pub(crate) async fn refresh_token() {}

pub(crate) async fn get_public_key(State(key_manager): State<KeyManagerState>) -> Vec<u8> {
    key_manager.public_key().to_vec()
}
