mod api;
mod args;
mod error;
mod secrets;
mod user;

use anyhow::Result;
use args::{CopyPermitEnvArgs, StartArgs, SyncArgs};
use http::StatusCode;
use shuttle_backends::{
    client::{
        permit::{self, Error, ResponseContent},
        PermissionsDal,
    },
    key::ApiKey,
};
use shuttle_common::models::user::AccountTier;
use sqlx::{query, PgPool};
use tracing::info;
pub use user::User;

use crate::api::serve;
pub use api::ApiBuilder;
pub use args::{Args, Commands, InitArgs};

pub async fn start(pool: PgPool, args: StartArgs) {
    let router = api::ApiBuilder::new()
        .with_pg_pool(pool)
        .with_stripe_client(stripe::Client::new(args.stripe_secret_key))
        .with_permissions_client(permit::Client::new(
            args.permit.permit_api_uri.to_string(),
            args.permit.permit_pdp_uri.to_string(),
            "default".to_string(),
            args.permit.permit_env,
            args.permit.permit_api_key,
        ))
        .with_jwt_signing_private_key(args.jwt_signing_private_key)
        .into_router();

    info!(address=%args.address, "Binding to and listening at address");

    serve(router, args.address).await;
}

pub async fn sync(pool: PgPool, args: SyncArgs) -> Result<()> {
    let users: Vec<User> = sqlx::query_as("SELECT * FROM users")
        .fetch_all(&pool)
        .await?;

    let permit_client = permit::Client::new(
        args.permit.permit_api_uri.to_string(),
        args.permit.permit_pdp_uri.to_string(),
        "default".to_string(),
        args.permit.permit_env,
        args.permit.permit_api_key,
    );

    for user in users {
        match permit_client.get_user(&user.id).await {
            Ok(p_user) => {
                // Update tier if out of sync
                let wanted_tier = user.account_tier.as_permit_account_tier();
                if !p_user
                    .roles
                    .is_some_and(|rs| rs.iter().any(|r| r.role == wanted_tier.to_string()))
                {
                    println!("updating tier for user: {}", user.id);
                    match wanted_tier {
                        AccountTier::Basic => {
                            permit_client.make_basic(&user.id).await?;
                        }
                        AccountTier::Pro => {
                            permit_client.make_pro(&user.id).await?;
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Err(Error::ResponseError(ResponseContent {
                status: StatusCode::NOT_FOUND,
                ..
            })) => {
                // Add users that are not in permit
                println!("creating user: {}", user.id);

                permit_client.new_user(&user.id).await?;
                if user.account_tier.as_permit_account_tier() == AccountTier::Pro {
                    permit_client.make_pro(&user.id).await?;
                }
            }
            Err(e) => {
                println!("failed to fetch user {}. skipping. error: {e}", user.id);
            }
        }
    }

    Ok(())
}

pub async fn copy_environment(args: CopyPermitEnvArgs) -> Result<()> {
    let client = permit::Client::new(
        args.permit.permit_api_uri.to_string(),
        args.permit.permit_pdp_uri.to_string(),
        "default".to_string(),
        args.permit.permit_env,
        args.permit.permit_api_key,
    );

    Ok(client.copy_environment(&args.target).await?)
}

pub async fn init(pool: PgPool, args: InitArgs, tier: AccountTier) -> Result<()> {
    let key = match args.key {
        Some(ref key) => ApiKey::parse(key).unwrap(),
        None => ApiKey::generate(),
    };

    query("INSERT INTO users (account_name, key, account_tier, user_id) VALUES ($1, $2, $3, $4)")
        .bind("")
        .bind(key.as_ref())
        .bind(tier.to_string())
        .bind(&args.user_id)
        .execute(&pool)
        .await?;

    println!(
        "`{}` created as {} with key: {}",
        args.user_id,
        tier,
        key.as_ref()
    );
    Ok(())
}

/// Initialize the connection pool to a Postgres database at the given URI.
pub async fn pgpool_init(db_uri: &str, migrate: bool) -> Result<PgPool> {
    let opts = db_uri.parse()?;
    let pool = PgPool::connect_with(opts).await?;
    if migrate {
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    }

    Ok(pool)
}
