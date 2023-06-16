use std::{fmt, path::Path, str::FromStr};

use crate::{
    session::SessionToken,
    user::{AccountName, AccountTier, User},
    Error,
};
use async_trait::async_trait;
use shuttle_common::ApiKey;
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    Row, SqlitePool,
};
use tracing::{error, info};

pub static MIGRATIONS: Migrator = sqlx::migrate!("./migrations");

#[derive(Error, Debug)]
pub enum DalError {
    Sqlx(#[from] sqlx::Error),
    UserNotFound,
}

// We are not using the `thiserror`'s `#[error]` syntax to prevent sensitive details from bubbling up to the users.
// Instead we are logging it as an error which we can inspect.
impl fmt::Display for DalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            DalError::Sqlx(error) => {
                error!(error = error.to_string(), "database request failed");
                "failed to interact with auth database"
            }
            DalError::UserNotFound => "user not found",
        };

        write!(f, "{msg}")
    }
}

#[async_trait]
pub trait Dal {
    /// Create a new account
    async fn create_user(
        &self,
        name: AccountName,
        key: ApiKey,
        tier: AccountTier,
    ) -> Result<User, DalError>;

    /// Delete a session by the session token.
    async fn delete_session(&self, session_token: &[u8]) -> Result<(), DalError>;

    /// Get an account by [AccountName]
    async fn get_user_by_name(&self, name: AccountName) -> Result<User, DalError>;

    /// Get an account by [ApiKey]
    async fn get_user_by_key(&self, key: ApiKey) -> Result<User, DalError>;

    /// Reset an account's [ApiKey]
    async fn update_api_key(&self, name: &AccountName, new_key: ApiKey) -> Result<(), DalError>;

    /// Insert a new session.
    async fn insert_session(
        &self,
        expiration: i64,
        name: &AccountName,
        session_token: SessionToken,
    ) -> Result<(), DalError>;

    /// Get a user by session token.
    async fn get_user_by_session_token(&self, session_token: &[u8]) -> Result<User, DalError>;
}

#[derive(Clone)]
pub struct Sqlite {
    pool: SqlitePool,
}

impl Sqlite {
    /// This function creates all necessary tables and sets up a database connection pool.
    pub async fn new(path: &str) -> Self {
        if !Path::new(path).exists() {
            sqlx::Sqlite::create_database(path).await.unwrap();
        }

        info!(
            "state db: {}",
            std::fs::canonicalize(path).unwrap().to_string_lossy()
        );

        // We have found in the past that setting synchronous to anything other than the default (full) breaks the
        // broadcast channel in deployer. The broken symptoms are that the ws socket connections won't get any logs
        // from the broadcast channel and would then close. When users did deploys, this would make it seem like the
        // deploy is done (while it is still building for most of the time) and the status of the previous deployment
        // would be returned to the user.
        //
        // If you want to activate a faster synchronous mode, then also do proper testing to confirm this bug is no
        // longer present.
        let sqlite_options = SqliteConnectOptions::from_str(path)
            .unwrap()
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePool::connect_with(sqlite_options).await.unwrap();

        Self::from_pool(pool).await
    }

    /// A utility for creating a migrating an in-memory database for testing.
    /// Currently only used for integration tests so the compiler thinks it is
    /// dead code.
    #[allow(dead_code)]
    pub async fn new_in_memory() -> Self {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        Self::from_pool(pool).await
    }

    async fn from_pool(pool: SqlitePool) -> Self {
        MIGRATIONS.run(&pool).await.unwrap();

        Self { pool }
    }
}

#[async_trait]
impl Dal for Sqlite {
    async fn create_user(
        &self,
        name: AccountName,
        key: ApiKey,
        tier: AccountTier,
    ) -> Result<User, DalError> {
        sqlx::query("INSERT INTO users (account_name, key, account_tier) VALUES (?1, ?2, ?3)")
            .bind(&name)
            .bind(&key)
            .bind(tier)
            .execute(&self.pool)
            .await?;

        Ok(User::new(name, key, tier))
    }

    async fn get_user_by_name(&self, name: AccountName) -> Result<User, DalError> {
        sqlx::query("SELECT account_name, key, account_tier FROM users WHERE account_name = ?1")
            .bind(&name)
            .fetch_optional(&self.pool)
            .await?
            .map(|row| User {
                name,
                key: row.try_get("key").expect("a user should always have a key"),
                account_tier: row
                    .try_get("account_tier")
                    .expect("a user should always have an account tier"),
            })
            .ok_or(DalError::UserNotFound)
    }

    async fn get_user_by_key(&self, key: ApiKey) -> Result<User, DalError> {
        sqlx::query("SELECT account_name, key, account_tier FROM users WHERE key = ?1")
            .bind(&key)
            .fetch_optional(&self.pool)
            .await?
            .map(|row| User {
                name: row
                    .try_get("account_name")
                    .expect("a user should always have an account name"),
                key,
                account_tier: row
                    .try_get("account_tier")
                    .expect("a user should always have an account tier"),
            })
            .ok_or(DalError::UserNotFound)
    }

    async fn update_api_key(&self, name: &AccountName, new_key: ApiKey) -> Result<(), DalError> {
        let rows_affected = sqlx::query("UPDATE users SET key = ?1 WHERE account_name = ?2")
            .bind(&new_key)
            .bind(name)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if rows_affected > 0 {
            Ok(())
        } else {
            Err(DalError::UserNotFound)
        }
    }

    async fn insert_session(
        &self,
        expiration: i64,
        name: &AccountName,
        session_token: SessionToken,
    ) -> Result<(), DalError> {
        sqlx::query(
            "INSERT INTO sessions (session_token, account_name, expiration) VALUES (?1, ?2, ?3)",
        )
        .bind(&session_token.into_database_value())
        .bind(name)
        .bind(expiration)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get user by session token if the session has not expired.
    async fn get_user_by_session_token(&self, token: &[u8]) -> Result<User, DalError> {
        let now = cookie::time::OffsetDateTime::now_utc().unix_timestamp();

        sqlx::query(
            r#"
            SELECT users.account_name, key, account_tier 
            FROM users 
            JOIN sessions ON sessions.account_name = users.account_name 
            WHERE session_token = ?1 AND expiration > ?2
        "#,
        )
        .bind(token)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| User {
            name: row
                .try_get("account_name")
                .expect("a user should always have an account name"),
            key: row.try_get("key").expect("a user should always have a key"),
            account_tier: row
                .try_get("account_tier")
                .expect("a user should always have an account tier"),
        })
        .ok_or(DalError::UserNotFound)
    }

    async fn delete_session(&self, token: &[u8]) -> Result<(), DalError> {
        sqlx::query("DELETE FROM sessions WHERE session_token = ?1")
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
