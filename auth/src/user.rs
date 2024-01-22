use std::{fmt::Formatter, io::ErrorKind, str::FromStr};

use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    headers::{authorization::Bearer, Authorization, HeaderMapExt},
    http::request::Parts,
    TypedHeader,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use shuttle_common::{backends::headers::XShuttleAdminSecret, claims::AccountTier, ApiKey, Secret};
use sqlx::{postgres::PgRow, query, FromRow, PgPool, Row};
use strum::EnumString;
use tracing::{debug, error, trace, Span};

use crate::{api::UserManagerState, error::Error};
use stripe::{
    CheckoutSession, CheckoutSessionStatus, Expandable, SubscriptionId, SubscriptionStatus,
};

#[async_trait]
pub trait UserManagement: Send + Sync {
    async fn create_user(&self, name: AccountName, tier: AccountTier) -> Result<User, Error>;
    async fn upgrade_to_pro(
        &self,
        name: &AccountName,
        checkout_session_metadata: CheckoutSession,
    ) -> Result<(), Error>;
    async fn update_tier(&self, name: &AccountName, tier: AccountTier) -> Result<(), Error>;
    async fn get_user(&self, name: AccountName) -> Result<User, Error>;
    async fn get_user_by_key(&self, key: ApiKey) -> Result<User, Error>;
    async fn reset_key(&self, name: AccountName) -> Result<(), Error>;
    async fn insert_subscription(
        &self,
        name: AccountName,
        r#type: ShuttleSubscriptionType,
    ) -> Result<(), Error>;
    async fn increment_subscription_quantity(
        &self,
        name: AccountName,
        r#type: ShuttleSubscriptionType,
    ) -> Result<(), Error>;
    // TODO: add delete sub method.
}

#[derive(Clone)]
pub struct UserManager {
    pub pool: PgPool,
    pub stripe_client: stripe::Client,
}

#[async_trait]
impl UserManagement for UserManager {
    async fn create_user(&self, name: AccountName, tier: AccountTier) -> Result<User, Error> {
        let key = ApiKey::generate();

        query("INSERT INTO users (account_name, key, account_tier) VALUES ($1, $2, $3)")
            .bind(&name)
            .bind(&key)
            .bind(tier.to_string())
            .execute(&self.pool)
            .await?;

        Ok(User::new(name, key, tier, vec![]))
    }

    // Update user tier to pro and update the subscription id.
    async fn upgrade_to_pro(
        &self,
        name: &AccountName,
        checkout_session_metadata: CheckoutSession,
    ) -> Result<(), Error> {
        // Update the user tier and store the subscription id. We expect the checkout session to be
        // completed when it is sent. In case of incomplete checkout sessions, auth backend will not
        // fulfill the request.
        if checkout_session_metadata
            .status
            .filter(|inner| inner == &CheckoutSessionStatus::Complete)
            .is_some()
        {
            // Extract the checkout session status if any, otherwise return with error.
            let subscription_id = checkout_session_metadata
                .subscription
                .map(|s| match s {
                    Expandable::Id(id) => id.to_string(),
                    Expandable::Object(obj) => obj.id.to_string(),
                })
                .ok_or(Error::MissingSubscriptionId)?;

            // Update the user account tier and insert or update their pro subscription.
            let mut transaction = self.pool.begin().await?;

            let rows_affected = query("UPDATE users SET account_tier = $1 WHERE account_name = $2")
                .bind(AccountTier::Pro.to_string())
                .bind(name)
                .execute(&mut *transaction)
                .await?
                .rows_affected();

            // Insert a new pro subscription. If a pro subscription already exists, update the
            // subscription id.
            // NOTE: we do not increase the quantity if a pro subscription exists, because it
            // should never be increased.
            query(
                r#"INSERT INTO subscriptions (subscription_id, account_name, type)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (account_name, type)
                    DO UPDATE SET subscription_id = EXCLUDED.subscription_id
                "#,
            )
            .bind(&subscription_id)
            .bind(name)
            .bind(ShuttleSubscriptionType::Pro.to_string())
            .execute(&mut *transaction)
            .await?;

            transaction.commit().await?;

            // In case no rows were updated, this means the account doesn't exist.
            if rows_affected > 0 {
                Ok(())
            } else {
                Err(Error::UserNotFound)
            }
        } else {
            Err(Error::IncompleteCheckoutSession)
        }
    }

    // Update tier leaving the subscription_id untouched.
    async fn update_tier(&self, name: &AccountName, tier: AccountTier) -> Result<(), Error> {
        let rows_affected = query("UPDATE users SET account_tier = $1 WHERE account_name = $2")
            .bind(tier.to_string())
            .bind(name)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if rows_affected > 0 {
            Ok(())
        } else {
            Err(Error::UserNotFound)
        }
    }

    async fn get_user(&self, name: AccountName) -> Result<User, Error> {
        let mut user: User = sqlx::query_as(
            "SELECT account_name, key, account_tier FROM users WHERE account_name = $1",
        )
        .bind(&name)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(Error::UserNotFound)?;

        let subscriptions: Vec<Subscription> = sqlx::query_as(
            "SELECT subscription_id, type, quantity, updated_at FROM subscriptions WHERE account_name = $1",
        )
        .bind(&user.name.to_string())
        .fetch_all(&self.pool)
        .await?;

        if !subscriptions.is_empty() {
            user.subscriptions = subscriptions;
        }

        // Sync the user tier based on the subscription validity, if any.
        if let Err(err) = user.sync_tier(self).await {
            error!("failed syncing account");
            return Err(err);
        } else {
            debug!("synced account");
        }

        Ok(user)
    }

    async fn get_user_by_key(&self, key: ApiKey) -> Result<User, Error> {
        let mut user: User =
            sqlx::query_as("SELECT account_name, key, account_tier FROM users WHERE key = $1")
                .bind(&key)
                .fetch_optional(&self.pool)
                .await?
                .ok_or(Error::UserNotFound)?;

        let subscriptions: Vec<Subscription> = sqlx::query_as(
            "SELECT subscription_id, type, quantity, updated_at FROM subscriptions WHERE account_name = $1",
        )
        .bind(&user.name.to_string())
        .fetch_all(&self.pool)
        .await?;

        if !subscriptions.is_empty() {
            user.subscriptions = subscriptions;
        }

        // Sync the user tier based on the subscription validity, if any.
        if user.sync_tier(self).await? {
            debug!("synced account");
        }

        Ok(user)
    }

    async fn reset_key(&self, name: AccountName) -> Result<(), Error> {
        let key = ApiKey::generate();

        let rows_affected = query("UPDATE users SET key = $1 WHERE account_name = $2")
            .bind(&key)
            .bind(&name)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if rows_affected > 0 {
            Ok(())
        } else {
            Err(Error::UserNotFound)
        }
    }

    async fn insert_subscription(
        &self,
        name: AccountName,
        r#type: ShuttleSubscriptionType,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn increment_subscription_quantity(
        &self,
        name: AccountName,
        r#type: ShuttleSubscriptionType,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub name: AccountName,
    pub key: Secret<ApiKey>,
    pub account_tier: AccountTier,
    pub subscriptions: Vec<Subscription>,
}

#[derive(Clone, Debug)]
pub struct Subscription {
    pub id: stripe::SubscriptionId,
    pub r#type: ShuttleSubscriptionType,
    pub quantity: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, EnumString, strum::Display, Deserialize, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum ShuttleSubscriptionType {
    Pro,
    Rds,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.account_tier == AccountTier::Admin
    }

    pub fn pro_subscription_id(&self) -> Option<&stripe::SubscriptionId> {
        self.subscriptions
            .iter()
            .find(|sub| matches!(sub.r#type, ShuttleSubscriptionType::Pro))
            .map(|sub| &sub.id)
    }

    pub fn new(
        name: AccountName,
        key: ApiKey,
        account_tier: AccountTier,
        subscriptions: Vec<Subscription>,
    ) -> Self {
        Self {
            name,
            key: Secret::new(key),
            account_tier,
            subscriptions,
        }
    }

    /// In case of an existing subscription, check if valid.
    async fn subscription_is_valid(&self, client: &stripe::Client) -> Result<bool, Error> {
        if let Some(subscription_id) = self.pro_subscription_id() {
            let subscription = stripe::Subscription::retrieve(client, subscription_id, &[]).await?;
            debug!("subscription: {:#?}", subscription);
            return Ok(subscription.status == SubscriptionStatus::Active
                || subscription.status == SubscriptionStatus::Trialing);
        }

        Ok(false)
    }

    // Synchronize the tiers with the subscription validity.
    async fn sync_tier(&mut self, user_manager: &UserManager) -> Result<bool, Error> {
        let has_pro_access = self.account_tier == AccountTier::Pro
            || self.account_tier == AccountTier::CancelledPro
            || self.account_tier == AccountTier::PendingPaymentPro;

        if !has_pro_access {
            return Ok(false);
        }

        let subscription_is_valid = self
            .subscription_is_valid(&user_manager.stripe_client)
            .await?;

        if self.account_tier == AccountTier::CancelledPro && !subscription_is_valid {
            self.account_tier = AccountTier::Basic;
            user_manager
                .update_tier(&self.name, self.account_tier)
                .await?;
            return Ok(true);
        }

        if self.account_tier == AccountTier::Pro && !subscription_is_valid {
            self.account_tier = AccountTier::PendingPaymentPro;
            user_manager
                .update_tier(&self.name, self.account_tier)
                .await?;
            return Ok(true);
        }

        if self.account_tier == AccountTier::PendingPaymentPro && subscription_is_valid {
            self.account_tier = AccountTier::Pro;
            user_manager
                .update_tier(&self.name, self.account_tier)
                .await?;
            return Ok(true);
        }

        Ok(false)
    }
}

impl FromRow<'_, PgRow> for User {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(User {
            name: row.try_get("account_name").unwrap(),
            key: Secret::new(row.try_get("key").unwrap()),
            account_tier: AccountTier::from_str(row.try_get("account_tier").unwrap()).map_err(
                |err| sqlx::Error::ColumnDecode {
                    index: "account_tier".to_string(),
                    source: Box::new(std::io::Error::new(ErrorKind::Other, err.to_string())),
                },
            )?,
            subscriptions: vec![],
        })
    }
}

impl FromRow<'_, PgRow> for Subscription {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Subscription {
            id: row
                .try_get("subscription_id")
                .ok()
                .and_then(|inner| SubscriptionId::from_str(inner).ok())
                .unwrap(),
            r#type: ShuttleSubscriptionType::from_str(row.try_get("type").unwrap()).map_err(
                |err| sqlx::Error::ColumnDecode {
                    index: "type".to_string(),
                    source: Box::new(std::io::Error::new(ErrorKind::Other, err.to_string())),
                },
            )?,
            quantity: row.try_get("quantity").unwrap(),
            updated_at: row.try_get("updated_at").unwrap(),
        })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    UserManagerState: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let key = Key::from_request_parts(parts, state).await?;

        let user_manager: UserManagerState = UserManagerState::from_ref(state);

        let user = user_manager
            .get_user_by_key(key.into())
            .await
            // Absorb any error into `Unauthorized`
            .map_err(|_| Error::Unauthorized)?;

        // Record current account name for tracing purposes
        Span::current().record("account.name", &user.name.to_string());

        Ok(user)
    }
}

impl From<User> for shuttle_common::models::user::Response {
    fn from(user: User) -> Self {
        Self {
            name: user.name.to_string(),
            key: user.key.expose().as_ref().to_owned(),
            account_tier: user.account_tier.to_string(),
            // TODO: this just returns what was always returned, the id of the pro subscription.
            // We will need to update this when we want to also return the rds subscription. We
            // can return a vec of IDs, but it will be a breaking change for the console (I don't believe it is used anywhere else).
            subscription_id: user.pro_subscription_id().map(|id| id.to_string()),
        }
    }
}

/// A wrapper around [ApiKey] so we can implement [FromRequestParts] for it.
pub struct Key(ApiKey);

impl From<Key> for ApiKey {
    fn from(key: Key) -> Self {
        key.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Key
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let key = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::KeyMissing)
            .and_then(|TypedHeader(Authorization(bearer))| {
                let bearer = bearer.token().trim();
                ApiKey::parse(bearer).map_err(|error| {
                    debug!(error = ?error, "received a malformed api-key");
                    Self::Rejection::Unauthorized
                })
            })?;

        trace!("got bearer key");

        Ok(Key(key))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type, Serialize)]
#[sqlx(transparent)]
pub struct AccountName(String);

impl From<String> for AccountName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl FromStr for AccountName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.to_string().into())
    }
}

impl std::fmt::Display for AccountName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for AccountName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

pub struct Admin {
    pub user: User,
}

#[async_trait]
impl<S> FromRequestParts<S> for Admin
where
    S: Send + Sync,
    UserManagerState: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = User::from_request_parts(parts, state).await?;
        if user.is_admin() {
            return Ok(Self { user });
        }

        match parts.headers.typed_try_get::<XShuttleAdminSecret>() {
            Ok(Some(secret)) => {
                let user_manager = UserManagerState::from_ref(state);
                // For this particular case, we expect the secret to be an admin API key.
                let key = ApiKey::parse(&secret.0).map_err(|_| Error::Unauthorized)?;
                let admin_user = user_manager
                    .get_user_by_key(key)
                    .await
                    .map_err(|_| Error::Unauthorized)?;
                if admin_user.is_admin() {
                    Ok(Self { user: admin_user })
                } else {
                    Err(Error::Unauthorized)
                }
            }
            Ok(_) => Err(Error::Unauthorized),
            // Returning forbidden for the cases where we don't understand why we can not authorize.
            Err(_) => Err(Error::Forbidden),
        }
    }
}

#[cfg(test)]
mod tests {
    mod convert_tiers {
        use shuttle_common::claims::Scope;

        use crate::user::AccountTier;

        #[test]
        fn basic() {
            let scopes: Vec<Scope> = AccountTier::Basic.into();

            assert_eq!(
                scopes,
                vec![
                    Scope::Deployment,
                    Scope::DeploymentPush,
                    Scope::Logs,
                    Scope::Service,
                    Scope::ServiceCreate,
                    Scope::Project,
                    Scope::ProjectWrite,
                    Scope::Resources,
                    Scope::ResourcesWrite,
                    Scope::Secret,
                    Scope::SecretWrite,
                ]
            );
        }

        #[test]
        fn pending_payment_pro() {
            let scopes: Vec<Scope> = AccountTier::PendingPaymentPro.into();

            assert_eq!(
                scopes,
                vec![
                    Scope::Deployment,
                    Scope::DeploymentPush,
                    Scope::Logs,
                    Scope::Service,
                    Scope::ServiceCreate,
                    Scope::Project,
                    Scope::ProjectWrite,
                    Scope::Resources,
                    Scope::ResourcesWrite,
                    Scope::Secret,
                    Scope::SecretWrite,
                ]
            );
        }

        #[test]
        fn pro() {
            let scopes: Vec<Scope> = AccountTier::Pro.into();

            assert_eq!(
                scopes,
                vec![
                    Scope::Deployment,
                    Scope::DeploymentPush,
                    Scope::Logs,
                    Scope::Service,
                    Scope::ServiceCreate,
                    Scope::Project,
                    Scope::ProjectWrite,
                    Scope::Resources,
                    Scope::ResourcesWrite,
                    Scope::Secret,
                    Scope::SecretWrite,
                    Scope::ExtraProjects,
                ]
            );
        }

        #[test]
        fn team() {
            let scopes: Vec<Scope> = AccountTier::Team.into();

            assert_eq!(
                scopes,
                vec![
                    Scope::Deployment,
                    Scope::DeploymentPush,
                    Scope::Logs,
                    Scope::Service,
                    Scope::ServiceCreate,
                    Scope::Project,
                    Scope::ProjectWrite,
                    Scope::Resources,
                    Scope::ResourcesWrite,
                    Scope::Secret,
                    Scope::SecretWrite,
                ]
            );
        }

        #[test]
        fn admin() {
            let scopes: Vec<Scope> = AccountTier::Admin.into();

            assert_eq!(
                scopes,
                vec![
                    Scope::Deployment,
                    Scope::DeploymentPush,
                    Scope::Logs,
                    Scope::Service,
                    Scope::ServiceCreate,
                    Scope::Project,
                    Scope::ProjectWrite,
                    Scope::Resources,
                    Scope::ResourcesWrite,
                    Scope::Secret,
                    Scope::SecretWrite,
                    Scope::User,
                    Scope::UserCreate,
                    Scope::AcmeCreate,
                    Scope::CustomDomainCreate,
                    Scope::CustomDomainCertificateRenew,
                    Scope::GatewayCertificateRenew,
                    Scope::Admin,
                ]
            );
        }

        #[test]
        fn deployer_machine() {
            let scopes: Vec<Scope> = AccountTier::Deployer.into();

            assert_eq!(
                scopes,
                vec![
                    Scope::DeploymentPush,
                    Scope::Resources,
                    Scope::Service,
                    Scope::ResourcesWrite
                ]
            );
        }
    }
}
