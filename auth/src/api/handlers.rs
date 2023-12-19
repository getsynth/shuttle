use crate::{
    error::Error,
    user::{AccountName, Admin, Key, User},
    NewSubscriptionItemExtractor,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use axum_sessions::extractors::{ReadableSession, WritableSession};
use http::StatusCode;
use shuttle_common::{
    claims::{AccountTier, Claim},
    models::user,
};
use stripe::CheckoutSession;
use tracing::{error, instrument};

use super::{builder::KeyManagerState, RouterState, UserManagerState};

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

#[instrument(skip(user_manager))]
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

#[instrument(skip(claim, user_manager), fields(account_name = claim.sub, account_tier = %claim.tier))]
pub(crate) async fn add_subscription_items(
    Extension(claim): Extension<Claim>,
    State(user_manager): State<UserManagerState>,
    NewSubscriptionItemExtractor(update_subscription_items): NewSubscriptionItemExtractor,
) -> Result<(), Error> {
    // Fetching the user will also sync their subscription. This means we can verify that the
    // caller still has the required tier after the sync.
    let user = user_manager.get_user(AccountName::from(claim.sub)).await?;

    let Some(ref subscription_id) = user.subscription_id else {
        return Err(Error::MissingSubscriptionId);
    };

    if !matches![user.account_tier, AccountTier::Pro | AccountTier::Admin] {
        error!("account was downgraded from pro in sync, denying the addition of new items");
        return Err(Error::Unauthorized);
    }

    user_manager
        .add_subscription_items(subscription_id, update_subscription_items)
        .await?;

    Ok(())
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

    let claim = Claim::new(
        account_name,
        account_tier.into(),
        account_tier,
        account_tier,
    );

    let token = claim.into_token(key_manager.private_key())?;

    let response = shuttle_common::backends::auth::ConvertResponse { token };

    Ok(Json(response))
}

/// Convert a valid API-key bearer token to a JWT.
pub(crate) async fn convert_key(
    State(RouterState {
        key_manager,
        user_manager,
        ..
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
