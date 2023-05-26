//! Shuttle resource providing a SQLite database. The database will be created in-process by the shuttle runtime.
//!
//! ## Example
//! ```rust
//! TODO: SIMPLE EXAMPLE
//! ```
//! ## Configuration
//! The database can be configured using `SQLiteConnOpts` which mirrors `sqlx::sqlite::SQLiteConnectOptions` for the
//! options it exposes. Shuttle does currently not support the `collation`, `thread_name`, `log_settings`, `pragma`,
//! `extension` options.
//! Construction of the full connection string is handled internally for security reasons and defaults to creating a
//! file-based database named `default_db.sqlite` with `create_if_missing == true`. Use the `filename` and/or
//! `in_memory` methods to configure the type of database created.
//! See the [`sqlx::sqlite::SQLiteConnectOptions`](https://docs.rs/sqlx/latest/sqlx/sqlite/struct.SQLiteConnectOptions.html)
//! docs for all other default settings.
//!
//! ```rust
//! TODO: CONFIG EXAMPLE
//! ```
use async_trait::async_trait;
use serde::Serialize;
use shuttle_service::{Factory, ResourceBuilder, Type};

pub use sqlx::SqlitePool;

pub mod conn_opts;
pub use conn_opts::*;

#[derive(Serialize)]
pub struct SQLite {
    opts: SQLiteConnOpts,
}

impl SQLite {
    pub fn opts(mut self, opts: SQLiteConnOpts) -> Self {
        self.opts = opts;
        self
    }

    pub fn in_memory(mut self, on: bool) -> Self {
        self.opts.in_memory = on;
        self
    }
}

#[async_trait]
impl ResourceBuilder<sqlx::SqlitePool> for SQLite {
    /// The type of resource this creates
    const TYPE: Type = Type::EmbeddedDatabase;

    /// The internal config being constructed by this builder. This will be used to find cached [Self::Output].
    type Config = Self;

    /// The output type used to build this resource later
    type Output = SQLiteConnOpts;

    /// Create a new instance of this resource builder
    fn new() -> Self {
        Self {
            opts: SQLiteConnOpts::default(),
        }
    }

    /// Get the internal config state of the builder
    ///
    /// If the exact same config was returned by a previous deployement that used this resource, then [Self::output()]
    /// will not be called to get the builder output again. Rather the output state of the previous deployment
    /// will be passed to [Self::build()].
    fn config(&self) -> &Self::Config {
        &self
    }

    /// Get the config output of this builder
    ///
    /// This method is where the actual resource provisioning should take place and is expected to take the longest. It
    /// can at times even take minutes. That is why the output of this method is cached and calling this method can be
    /// skipped as explained in [Self::config()].
    async fn output(
        mut self,
        factory: &mut dyn Factory,
    ) -> Result<Self::Output, shuttle_service::Error> {
        // We construct an absolute path using `storage_path` to prevent user access to other parts of the file system.
        let storage_path = factory.get_storage_path()?;
        let db_path = storage_path.join(&self.opts.filename);

        let conn_str = match self.opts.in_memory {
            true => "sqlite::memory:".to_string(),
            false => format!("sqlite:///{}", db_path.display()),
        };

        self.opts.conn_str = conn_str;

        Ok(self.opts)
    }

    /// Build this resource from its config output
    async fn build(build_data: &Self::Output) -> Result<sqlx::SqlitePool, shuttle_service::Error> {
        let pool = sqlx::SqlitePool::connect_with(build_data.into())
            .await
            .expect("Failed to create sqlite database");

        Ok(pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //
    }
}
