use crate::deployment::deploy_layer::{self, LogRecorder, LogType};
use crate::deployment::{DeploymentInfo, Log, State};
use crate::error::Result;

use std::path::Path;

use serde_json::json;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{Sqlite, SqlitePool};
use tokio::sync::mpsc::{self, UnboundedSender};

const DB_PATH: &str = "deployer.sqlite";

#[derive(Clone)]
pub struct Persistence {
    pool: SqlitePool,
    log_send: UnboundedSender<deploy_layer::Log>,
}

impl Persistence {
    /// Creates a persistent storage solution (i.e., SQL database). This
    /// function creates all necessary tables and sets up a database connection
    /// pool - new connections should be made by cloning [`Persistence`] rather
    /// than repeatedly calling [`Persistence::new`].
    pub async fn new() -> Self {
        if !Path::new(DB_PATH).exists() {
            Sqlite::create_database(DB_PATH).await.unwrap();
        }

        let pool = SqlitePool::connect(DB_PATH).await.unwrap();
        Self::from_pool(pool).await
    }

    #[allow(dead_code)]
    async fn new_in_memory() -> Self {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        Self::from_pool(pool).await
    }

    async fn from_pool(pool: SqlitePool) -> Self {
        sqlx::query("
            CREATE TABLE IF NOT EXISTS deployments (
                name TEXT PRIMARY KEY, -- Name of the service being deployed.
                state INTEGER          -- Enum indicating the current state of the deployment.
            );

            CREATE TABLE IF NOT EXISTS logs (
                name TEXT,         -- The service that this log line pertains to.
                timestamp INTEGER, -- Unix eopch timestamp.
                state INTEGER,     -- The state of the deployment at the time at which the log text was produced.
                level TEXT,        -- The log level
                fields TEXT        -- Log fields object.
            );
        ").execute(&pool).await.unwrap();

        let (log_send, mut log_recv) = mpsc::unbounded_channel();

        let me = Self { pool, log_send };
        let me_clone = me.clone();

        // The logs are received on a non-async thread.
        // This moves them to an async thread
        tokio::spawn(async move {
            while let Some(log) = log_recv.recv().await {
                match log.r#type {
                    LogType::Event => me_clone.insert_log(log).await.unwrap(),
                    LogType::State => {
                        me_clone
                            .insert_log(Log {
                                name: log.name.clone(),
                                timestamp: log.timestamp.clone(),
                                state: log.state.clone(),
                                level: log.level.clone(),
                                fields: json!("NEW STATE"),
                            })
                            .await
                            .unwrap();
                        me_clone.update_deployment(log).await.unwrap();
                    }
                };
            }
        });

        me
    }

    pub async fn update_deployment(&self, info: impl Into<DeploymentInfo>) -> Result<()> {
        let info = info.into();

        // TODO: Handle moving to 'active_deployments' table for State::Running.

        sqlx::query("INSERT OR REPLACE INTO deployments (name, state) VALUES (?, ?)")
            .bind(info.name)
            .bind(info.state)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn get_deployment(&self, name: &str) -> Result<Option<DeploymentInfo>> {
        sqlx::query_as("SELECT * FROM deployments WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_deployment(&self, name: &str) -> Result<Option<DeploymentInfo>> {
        let info = self.get_deployment(name).await?;

        let _ = sqlx::query("DELETE FROM deployments WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await;

        Ok(info)
    }

    pub async fn get_all_deployments(&self) -> Result<Vec<DeploymentInfo>> {
        sqlx::query_as("SELECT * FROM deployments")
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn get_all_runnable_deployments(&self) -> Result<Vec<DeploymentInfo>> {
        sqlx::query_as("SELECT * FROM deployments WHERE state = ? OR state = ?")
            .bind(State::Built)
            .bind(State::Running)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn insert_log(&self, log: impl Into<Log>) -> Result<()> {
        let log = log.into();

        sqlx::query(
            "INSERT INTO logs (name, timestamp, state, level, fields) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(log.name)
        .bind(log.timestamp)
        .bind(log.state)
        .bind(log.level)
        .bind(log.fields)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    async fn get_deployment_logs(&self, name: &str) -> Result<Vec<Log>> {
        sqlx::query_as("SELECT * FROM logs WHERE name = ?")
            .bind(name)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
}

impl LogRecorder for Persistence {
    fn record(&self, log: deploy_layer::Log) {
        self.log_send
            .send(log)
            .expect("failed to move log to async thread");
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;
    use serde_json::json;

    use super::*;
    use crate::deployment::{log::Level, Built};

    #[tokio::test]
    async fn deployment_updates() {
        let p = Persistence::new_in_memory().await;

        let mut info = DeploymentInfo {
            name: "abc".to_string(),
            state: State::Queued,
        };

        p.update_deployment(info.clone()).await.unwrap();
        assert_eq!(p.get_deployment("abc").await.unwrap().unwrap(), info);

        p.update_deployment(&Built {
            name: "abc".to_string(),
        })
        .await
        .unwrap();
        info.state = State::Built;
        assert_eq!(p.get_deployment("abc").await.unwrap().unwrap(), info);
    }

    #[tokio::test]
    async fn fetching_runnable_deployments() {
        let p = Persistence::new_in_memory().await;

        for info in [
            DeploymentInfo {
                name: "abc".to_string(),
                state: State::Queued,
            },
            DeploymentInfo {
                name: "foo".to_string(),
                state: State::Built,
            },
            DeploymentInfo {
                name: "bar".to_string(),
                state: State::Running,
            },
            DeploymentInfo {
                name: "def".to_string(),
                state: State::Building,
            },
        ] {
            p.update_deployment(info).await.unwrap();
        }

        let runnable = p.get_all_runnable_deployments().await.unwrap();
        assert!(!runnable.iter().any(|x| x.name == "abc"));
        assert!(runnable.iter().any(|x| x.name == "foo"));
        assert!(runnable.iter().any(|x| x.name == "bar"));
        assert!(!runnable.iter().any(|x| x.name == "def"));
    }

    #[tokio::test]
    async fn deployment_deletion() {
        let p = Persistence::new_in_memory().await;

        p.update_deployment(DeploymentInfo {
            name: "x".to_string(),
            state: State::Running,
        })
        .await
        .unwrap();
        assert!(p.get_deployment("x").await.unwrap().is_some());
        p.delete_deployment("x").await.unwrap();
        assert!(p.get_deployment("x").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn log_insert() {
        let p = Persistence::new_in_memory().await;
        let log = Log {
            name: "a".to_string(),
            timestamp: Utc::now(),
            state: State::Queued,
            level: Level::Info,
            fields: json!({"message": "job queued"}),
        };

        p.insert_log(log.clone()).await.unwrap();

        let logs = p.get_deployment_logs("a").await.unwrap();
        assert!(!logs.is_empty(), "there should be one log");

        assert_eq!(logs.first().unwrap(), &log);
    }

    #[tokio::test]
    async fn logs_for_deployment() {
        let p = Persistence::new_in_memory().await;
        let log_a1 = Log {
            name: "a".to_string(),
            timestamp: Utc::now(),
            state: State::Queued,
            level: Level::Info,
            fields: json!({"message": "job queued"}),
        };
        let log_b = Log {
            name: "b".to_string(),
            timestamp: Utc::now(),
            state: State::Queued,
            level: Level::Info,
            fields: json!({"message": "job queued"}),
        };
        let log_a2 = Log {
            name: "a".to_string(),
            timestamp: Utc::now(),
            state: State::Building,
            level: Level::Warn,
            fields: json!({"message": "unused Result"}),
        };

        p.insert_log(log_a1.clone()).await.unwrap();
        p.insert_log(log_b).await.unwrap();
        p.insert_log(log_a2.clone()).await.unwrap();

        let logs = p.get_deployment_logs("a").await.unwrap();
        assert!(!logs.is_empty(), "there should be three logs");

        assert_eq!(logs, vec![log_a1, log_a2]);
    }

    #[tokio::test]
    async fn log_recorder_event() {
        let p = Persistence::new_in_memory().await;
        let event = deploy_layer::Log {
            name: "x".to_string(),
            timestamp: Utc::now(),
            state: State::Queued,
            level: Level::Info,
            fields: json!({"message": "job queued"}),
            r#type: deploy_layer::LogType::Event,
        };

        p.record(event);

        tokio::time::sleep(Duration::from_micros(200)).await;
        let logs = p.get_deployment_logs("a").await.unwrap();

        assert!(!logs.is_empty(), "there should be one log");

        let log = logs.first().unwrap();
        assert_eq!(log.name, "x");
        assert_eq!(log.state, State::Queued);
        assert_eq!(log.level, Level::Info);
        assert_eq!(log.fields, json!({"message": "job queued"}));
    }

    #[tokio::test]
    async fn log_recorder_state() {
        tracing_subscriber::fmt::init();

        let p = Persistence::new_in_memory().await;
        let state = deploy_layer::Log {
            name: "z".to_string(),
            timestamp: Utc::now(),
            state: State::Running,
            level: Level::Info,
            fields: serde_json::Value::Null,
            r#type: deploy_layer::LogType::State,
        };

        p.record(state);

        tokio::time::sleep(Duration::from_micros(200)).await;
        let logs = p.get_deployment_logs("a").await.unwrap();

        assert!(!logs.is_empty(), "state change should be logged");

        let log = logs.first().unwrap();
        assert_eq!(log.name, "z");
        assert_eq!(log.state, State::Running);
        assert_eq!(log.level, Level::Info);
        assert_eq!(log.fields, json!("NEW STATE"));

        assert_eq!(
            p.get_deployment("z").await.unwrap().unwrap(),
            DeploymentInfo {
                name: "z".to_string(),
                state: State::Running
            }
        );
    }
}
