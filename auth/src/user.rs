use std::{io::ErrorKind, str::FromStr};

use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    headers::{authorization::Bearer, Authorization, HeaderMapExt},
    http::request::Parts,
    TypedHeader,
};
use chrono::{DateTime, Utc};
use shuttle_backends::{client::PermissionsDal, headers::XShuttleAdminSecret, key::ApiKey};
use shuttle_common::{
    limits::Limits,
    models::{
        self,
        user::{AccountTier, UserId},
    },
    Secret,
};
use sqlx::{postgres::PgRow, query, FromRow, PgPool, Row};
use stripe::{SubscriptionId, SubscriptionStatus};
use tracing::{debug, error, trace, Span};

use crate::{api::UserManagerState, error::Error};

#[async_trait]
pub trait UserManagement: Send + Sync {
    /// Create a user with the given auth0 name and tier
    async fn create_user(&self, account_name: String, tier: AccountTier) -> Result<User, Error>;

    async fn update_tier(&self, user_id: &UserId, tier: AccountTier) -> Result<(), Error>;
    async fn get_user_by_name(&self, account_name: &str) -> Result<User, Error>;
    async fn get_user(&self, user_id: UserId) -> Result<User, Error>;
    async fn get_user_by_key(&self, key: ApiKey) -> Result<User, Error>;
    async fn reset_key(&self, user_id: UserId) -> Result<(), Error>;
    async fn insert_subscription(
        &self,
        user_id: &UserId,
        subscription_id: &str,
        subscription_type: &models::user::SubscriptionType,
        subscription_quantity: i32,
    ) -> Result<(), Error>;
    async fn delete_subscription(
        &self,
        user_id: &UserId,
        subscription_id: &str,
    ) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct UserManager<P> {
    pub pool: PgPool,
    pub stripe_client: stripe::Client,
    pub permissions_client: P,
}

impl<P> UserManager<P>
where
    P: PermissionsDal + Send + Sync,
{
    /// Add subscriptions to and sync the tier of a user
    async fn complete_user(&self, mut user: User) -> Result<User, Error> {
        let subscriptions: Vec<Subscription> = sqlx::query_as(
            "SELECT subscription_id, type, quantity, created_at, updated_at FROM subscriptions WHERE user_id = $1",
        )
        .bind(&user.id)
        .fetch_all(&self.pool)
        .await?;

        if !subscriptions.is_empty() {
            user.subscriptions = subscriptions;
        }

        // Sync the user tier based on the subscription validity, if any.
        if let Err(err) = user.sync_tier(self).await {
            error!(
                error = &err as &dyn std::error::Error,
                "failed syncing account"
            );
            return Err(err);
        }
        debug!("synced account");

        Ok(user)
    }
}

#[async_trait]
impl<P> UserManagement for UserManager<P>
where
    P: PermissionsDal + Send + Sync,
{
    async fn create_user(&self, account_name: String, tier: AccountTier) -> Result<User, Error> {
        let user = User::new(account_name, ApiKey::generate(), tier, vec![], false);

        query(
            "INSERT INTO users (account_name, key, account_tier, user_id, has_access_to_beta) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&user.name)
        .bind(user.key.expose().as_ref())
        .bind(user.account_tier.to_string())
        .bind(&user.id)
        .bind(false)
        .execute(&self.pool)
        .await?;

        self.permissions_client.new_user(&user.id).await?;

        if tier == AccountTier::Pro {
            self.permissions_client.make_pro(&user.id).await?;
        }

        Ok(user)
    }

    // Update tier leaving the subscription_id untouched.
    async fn update_tier(&self, user_id: &UserId, tier: AccountTier) -> Result<(), Error> {
        let rows_affected = query("UPDATE users SET account_tier = $1 WHERE user_id = $2")
            .bind(tier.to_string())
            .bind(user_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if tier == AccountTier::Pro {
            self.permissions_client.make_pro(user_id).await?;
        } else {
            self.permissions_client.make_basic(user_id).await?;
        }

        if rows_affected > 0 {
            Ok(())
        } else {
            Err(Error::UserNotFound)
        }
    }

    async fn get_user_by_name(&self, account_name: &str) -> Result<User, Error> {
        let user: User = sqlx::query_as("SELECT * FROM users WHERE account_name = $1")
            .bind(account_name)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(Error::UserNotFound)?;

        self.complete_user(user).await
    }

    async fn get_user(&self, user_id: UserId) -> Result<User, Error> {
        let user: User = sqlx::query_as("SELECT * FROM users WHERE user_id = $1")
            .bind(&user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(Error::UserNotFound)?;

        self.complete_user(user).await
    }

    async fn get_user_by_key(&self, key: ApiKey) -> Result<User, Error> {
        let mut user: User = sqlx::query_as("SELECT * FROM users WHERE key = $1")
            .bind(key.as_ref())
            .fetch_optional(&self.pool)
            .await?
            .ok_or(Error::UserNotFound)?;

        let subscriptions: Vec<Subscription> = sqlx::query_as(
            "SELECT subscription_id, type, quantity, created_at, updated_at FROM subscriptions WHERE user_id = $1",
        )
        .bind(&user.id)
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

    async fn reset_key(&self, user_id: UserId) -> Result<(), Error> {
        let key = ApiKey::generate();

        let rows_affected = query("UPDATE users SET key = $1 WHERE user_id = $2")
            .bind(key.as_ref())
            .bind(&user_id)
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
        user_id: &UserId,
        subscription_id: &str,
        subscription_type: &models::user::SubscriptionType,
        subscription_quantity: i32,
    ) -> Result<(), Error> {
        let mut transaction = self.pool.begin().await?;

        if *subscription_type == models::user::SubscriptionType::Pro {
            query("UPDATE users SET account_tier = $1 WHERE user_id = $2")
                .bind(AccountTier::Pro.to_string())
                .bind(user_id)
                .execute(&mut *transaction)
                .await?;

            self.permissions_client.make_pro(user_id).await?;
        }

        // Insert a new subscription. If the same type of subscription already exists, update the
        // subscription id and quantity.
        let rows_affected = query(
            r#"INSERT INTO subscriptions (subscription_id, user_id, type, quantity)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, type)
            DO UPDATE SET subscription_id = EXCLUDED.subscription_id, quantity = EXCLUDED.quantity
        "#,
        )
        .bind(subscription_id)
        .bind(user_id)
        .bind(subscription_type.to_string())
        .bind(subscription_quantity)
        .execute(&mut *transaction)
        .await?
        .rows_affected();

        transaction.commit().await?;

        // In case no rows were updated, this means the account doesn't exist.
        if rows_affected > 0 {
            Ok(())
        } else {
            Err(Error::UserNotFound)
        }
    }

    async fn delete_subscription(
        &self,
        user_id: &UserId,
        subscription_id: &str,
    ) -> Result<(), Error> {
        let subscription: Subscription = sqlx::query_as(
            "SELECT subscription_id, type, quantity, created_at, updated_at FROM subscriptions WHERE user_id = $1 AND subscription_id = $2",
        )
        .bind(user_id)
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        if subscription.r#type == models::user::SubscriptionType::Pro {
            self.update_tier(user_id, AccountTier::CancelledPro).await?;

            self.permissions_client.make_basic(user_id).await?;
        } else {
            query(
                r#"DELETE FROM subscriptions
                WHERE subscription_id = $1 AND user_id = $2
            "#,
            )
            .bind(subscription_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub name: UserId,
    pub id: String,
    pub key: Secret<ApiKey>,
    pub account_tier: AccountTier,
    pub subscriptions: Vec<Subscription>,
    pub has_access_to_beta: bool,
}

#[derive(Clone, Debug)]
pub struct Subscription {
    pub id: stripe::SubscriptionId,
    pub r#type: models::user::SubscriptionType,
    pub quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.account_tier == AccountTier::Admin
    }

    fn pro_subscription_id(&self) -> Option<&stripe::SubscriptionId> {
        self.subscriptions
            .iter()
            .find(|sub| matches!(sub.r#type, models::user::SubscriptionType::Pro))
            .map(|sub| &sub.id)
    }

    pub fn new_user_id() -> String {
        format!("user_{}", ulid::Ulid::new())
    }

    fn new(
        name: UserId,
        key: ApiKey,
        account_tier: AccountTier,
        subscriptions: Vec<Subscription>,
        has_access_to_beta: bool,
    ) -> Self {
        Self {
            name,
            id: Self::new_user_id(),
            key: Secret::new(key),
            account_tier,
            subscriptions,
            has_access_to_beta,
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
    async fn sync_tier<P: PermissionsDal + Send + Sync>(
        &mut self,
        user_manager: &UserManager<P>,
    ) -> Result<bool, Error> {
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
                .update_tier(&self.id, self.account_tier)
                .await?;
            return Ok(true);
        }

        if self.account_tier == AccountTier::Pro && !subscription_is_valid {
            self.account_tier = AccountTier::PendingPaymentPro;
            user_manager
                .update_tier(&self.id, self.account_tier)
                .await?;
            return Ok(true);
        }

        if self.account_tier == AccountTier::PendingPaymentPro && subscription_is_valid {
            self.account_tier = AccountTier::Pro;
            user_manager
                .update_tier(&self.id, self.account_tier)
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
            id: row.try_get("user_id").unwrap(),
            key: Secret::new(ApiKey::parse(row.try_get("key").unwrap()).map_err(|err| {
                sqlx::Error::ColumnDecode {
                    index: "key".to_string(),
                    source: Box::new(std::io::Error::new(ErrorKind::Other, err.to_string())),
                }
            })?),
            account_tier: AccountTier::from_str(row.try_get("account_tier").unwrap()).map_err(
                |err| sqlx::Error::ColumnDecode {
                    index: "account_tier".to_string(),
                    source: Box::new(std::io::Error::new(ErrorKind::Other, err.to_string())),
                },
            )?,
            subscriptions: vec![],
            has_access_to_beta: row.try_get("has_access_to_beta").unwrap(),
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
            r#type: models::user::SubscriptionType::from_str(row.try_get("type").unwrap())
                .map_err(|err| sqlx::Error::ColumnDecode {
                    index: "type".to_string(),
                    source: Box::new(std::io::Error::new(ErrorKind::Other, err.to_string())),
                })?,
            quantity: row.try_get("quantity").unwrap(),
            created_at: row.try_get("created_at").unwrap(),
            updated_at: row.try_get("updated_at").unwrap(),
        })
    }
}

impl From<User> for Limits {
    fn from(user: User) -> Self {
        let mut limits: Limits = user.account_tier.into();

        let rds_quota = user
            .subscriptions
            .iter()
            .find(|sub| matches!(sub.r#type, models::user::SubscriptionType::Rds))
            .map(|sub| sub.quantity as u32)
            .unwrap_or(0);

        limits.set_rds_quota(rds_quota);

        limits
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
        Span::current().record("account.user_id", &user.id);

        Ok(user)
    }
}

impl From<User> for models::user::UserResponse {
    fn from(user: User) -> Self {
        Self {
            name: user.name.to_string(),
            id: user.id,
            key: user.key.expose().as_ref().to_owned(),
            account_tier: user.account_tier,
            subscriptions: user.subscriptions.into_iter().map(Into::into).collect(),
            has_access_to_beta: Some(user.has_access_to_beta),
        }
    }
}

impl From<Subscription> for models::user::Subscription {
    fn from(subscription: Subscription) -> Self {
        Self {
            id: subscription.id.to_string(),
            r#type: subscription.r#type,
            quantity: subscription.quantity,
            created_at: subscription.created_at,
            updated_at: subscription.updated_at,
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

pub struct Admin {
    pub _user: User,
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
            return Ok(Self { _user: user });
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
                    Ok(Self { _user: admin_user })
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
