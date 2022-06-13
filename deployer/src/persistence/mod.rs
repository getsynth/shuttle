// Handle database interactions.

use sqlx::sqlite::SqlitePool;

const DB_PATH: &str = "deployer.sqlite";

#[derive(Clone)]
pub struct Persistence {
    pool: SqlitePool,
}

impl Persistence {
    pub async fn new() -> Self {
        let pool = SqlitePool::connect(DB_PATH).await.unwrap();

        sqlx::query("
            CREATE TABLE IF NOT EXISTS deploying (
                state INTEGER -- Enum indicating the current state of the deployment.
            );

            CREATE TABLE IF NOT EXISTS active_deployments (
                id
            );

            CREATE TABLE IF NOT EXISTS logs (
                text TEXT,         -- Log line(s).
                state INTEGER,     -- The state of the deployment at the time at which the log text was produced.
                timestamp INTEGER, -- Unix eopch timestamp.
                id
            );
        ").execute(&pool).await.unwrap();

        Persistence { pool }
    }
}
