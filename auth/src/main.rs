use std::io;

use clap::Parser;
use shuttle_common::{backends::tracing::setup_tracing, log::Backend};
use sqlx::migrate::Migrator;
use tracing::{info, trace};

use shuttle_auth::{init, sqlite_init, start, AccountTier, Args, Commands};

pub static MIGRATIONS: Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    trace!(args = ?args, "parsed args");

    setup_tracing(tracing_subscriber::registry(), Backend::Auth);

    let db_path = args.state.join("authentication.sqlite");

    let db_uri = db_path.to_str().unwrap();

    let pool = sqlite_init(db_uri).await;

    info!(
        "state db: {}",
        std::fs::canonicalize(&args.state)
            .unwrap()
            .to_string_lossy()
    );

    match args.command {
        Commands::Start(args) => start(pool, args).await,
        Commands::InitAdmin(args) => init(pool, args, AccountTier::Admin).await,
        Commands::InitDeployer(args) => init(pool, args, AccountTier::Deployer).await,
    }
}
