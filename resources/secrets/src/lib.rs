#![doc = include_str!("../README.md")]

use anyhow::anyhow;
use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use shuttle_service::{error::CustomError, Error};
use sqlx::postgres::PgArguments;
use sqlx::query::Query;
use sqlx::{
    ColumnIndex, Database, Decode, Encode, Execute, Executor, IntoArguments, Postgres, Row, Type,
};

fn check_and_lower_secret_key(key: &str) -> Result<String, Error> {
    lazy_static! {
        static ref VALID_KEY: Regex = Regex::new(r"[_a-zA-Z][_a-zA-Z0-9]*").unwrap();
    }
    VALID_KEY
        .is_match(key)
        .then(|| key.to_lowercase())
        .ok_or_else(|| Error::Custom(anyhow!("invalid secret key name '{}'", key)))
}

/// Abstraction over a simple key/value 'secret' store. This may be used for any number of
/// purposes, such as storing API keys. Note that secrets are not encrypted and are stored directly
/// in a table in the database (meaning they can be accessed via SQL rather than this abstraction
/// should you prefer). The table in question is created if it is found to not exist every time
/// either [`SecretStore::get_secret`] or [`SecretStore::set_secret`] is called.
#[async_trait]
pub trait SecretStore<DB, Args>
where
    DB: Database<Arguments = Args>,
    Args: for<'q> IntoArguments<'q, DB>,
    for<'c> &'c Self: Executor<'c, Database = DB>,
    for<'c> String: Decode<'c, DB> + Encode<'c, DB> + Type<DB>,
    for<'c> &'c str: Decode<'c, DB> + Encode<'c, DB> + Type<DB>,
    for<'c> usize: ColumnIndex<<DB as Database>::Row>,
    for<'c> Query<'c, Postgres, PgArguments>: Execute<'c, DB>,
{
    // TODO: Don't restrict to Postgres types above.

    const GET_QUERY: &'static str;
    const SET_QUERY: &'static str;
    const CREATE_TABLE_QUERY: &'static str;

    /// Read the secret with the given key from the database. Will error if a secret with the
    /// given key does not exist or otherwise could not be accessed.
    async fn get_secret(&self, key: &str) -> Result<String, Error> {
        self.execute(sqlx::query(Self::CREATE_TABLE_QUERY))
            .await
            .map_err(CustomError::new)
            .map_err(Error::from)?;

        let key = check_and_lower_secret_key(key)?;
        let query = sqlx::query(Self::GET_QUERY).bind(&key);

        self.fetch_optional(query)
            .await
            .map_err(CustomError::new)
            .map_err(Error::from)
            .and_then(|m_one| {
                m_one.ok_or_else(|| {
                    Error::Secret(format!(
                        "Secret `{key}` not found in service environment. If you have made changes to your `Secrets.toml` file recently, try deploying again to make sure changes are applied correctly."
                    ))
                })
            })
            .map(|one| one.get(0))
    }

    /// Create (or overwrite if already present) a key/value secret in the database. Will error if
    /// the database could not be accessed or execution of the query otherwise failed.
    async fn set_secret(&self, key: &str, val: &str) -> Result<(), Error> {
        self.execute(sqlx::query(Self::CREATE_TABLE_QUERY))
            .await
            .map_err(CustomError::new)
            .map_err(Error::from)?;

        let key = check_and_lower_secret_key(key)?;
        let query = sqlx::query(Self::SET_QUERY).bind(key).bind(val);

        self.execute(query)
            .await
            .map_err(CustomError::new)
            .map_err(Error::from)?;

        Ok(())
    }
}

#[async_trait]
impl SecretStore<sqlx::Postgres, PgArguments> for sqlx::PgPool {
    const GET_QUERY: &'static str = "SELECT value FROM secrets WHERE key = $1";
    const SET_QUERY: &'static str = "INSERT INTO secrets (key, value) VALUES ($1, $2)
                                     ON CONFLICT (key) DO UPDATE SET value = $2";
    const CREATE_TABLE_QUERY: &'static str = "
        CREATE TABLE IF NOT EXISTS secrets (
            key TEXT UNIQUE NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY (key)
        );
    ";
}
