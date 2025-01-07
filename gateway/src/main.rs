use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use shuttle_backends::client::{permit, PermissionsDal};
use shuttle_backends::trace::setup_tracing;
use shuttle_common::log::Backend;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::{Sqlite, SqlitePool};
use tracing::{debug, error, info, info_span, trace, warn, Instrument};

use shuttle_gateway::acme::{AcmeClient, CustomDomain};
use shuttle_gateway::api::latest::{ApiBuilder, SVC_DEGRADED_THRESHOLD};
use shuttle_gateway::args::{Args, Commands, UseTls};
use shuttle_gateway::args::{StartArgs, SyncArgs};
use shuttle_gateway::proxy::UserServiceBuilder;
use shuttle_gateway::service::{GatewayService, MIGRATIONS};
use shuttle_gateway::tls::make_tls_acceptor;
use shuttle_gateway::worker::{Worker, WORKER_QUEUE_SIZE};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    setup_tracing(tracing_subscriber::registry(), Backend::Gateway);

    let args = Args::parse();
    trace!(args = ?args, "parsed args");

    let db_path = args.state.join("gateway.sqlite");
    let db_uri = db_path.to_str().unwrap();

    info!("Using state db: {}", db_uri);
    if !db_path.exists() {
        info!("Creating new state db");
        Sqlite::create_database(db_uri).await.unwrap();
    }

    let sqlite_options = SqliteConnectOptions::from_str(db_uri)
        .unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        // Set the ulid0 extension for generating ULID's in migrations.
        // This uses the ulid0.so file in the crate root, with the
        // LD_LIBRARY_PATH env set in build.rs.
        .extension("ulid0");

    info!("Connecting and migrating db...");
    let db = SqlitePool::connect_with(sqlite_options).await.unwrap();
    MIGRATIONS.run(&db).await.unwrap();

    match args.command {
        Commands::Start(start_args) => start(db, args.state, start_args).await,
        Commands::Sync(sync_args) => sync_permit_projects(db, sync_args).await,
    }
}

async fn sync_permit_projects(db: SqlitePool, args: SyncArgs) {
    let client = permit::Client::new(
        args.permit.permit_api_uri.to_string(),
        args.permit.permit_pdp_uri.to_string(),
        "default".to_owned(),
        args.permit.permit_env,
        args.permit.permit_api_key,
    );

    let projects: Vec<(String, String)> =
        sqlx::query_as("SELECT user_id, project_id FROM projects")
            .fetch_all(&db)
            .await
            .unwrap();
    let mut projects_by_user = BTreeMap::<String, Vec<String>>::new();
    for (uid, pid) in projects {
        let v = projects_by_user.entry(uid).or_default();
        v.push(pid);
    }

    for (uid, pids) in projects_by_user {
        println!("syncing {uid} projects");
        match client.get_personal_projects(&uid).await {
            Ok(projs) => {
                for pid in pids {
                    if !projs.iter().any(|p| *p == pid) {
                        println!("creating project link {uid} <-> {pid}");
                        client.create_project(&uid, &pid).await.unwrap();
                    } else {
                        println!("project link exists {uid} <-> {pid}");
                    }
                }
            }
            Err(e) => {
                println!("failed to get projects for {uid}. skipping. error: {e:?}");
            }
        }
    }
}

async fn start(db: SqlitePool, state_dir: PathBuf, args: StartArgs) {
    let gateway = Arc::new(
        GatewayService::init(
            args.context.clone(),
            db,
            state_dir,
            Box::new(permit::Client::new(
                args.permit.permit_api_uri.to_string(),
                args.permit.permit_pdp_uri.to_string(),
                "default".to_owned(),
                args.permit.permit_env,
                args.permit.permit_api_key,
            )),
        )
        .await
        .unwrap(),
    );

    let worker = Worker::new();

    let sender = worker.sender();

    let worker_handle = tokio::spawn(worker.start());

    // Every 60 secs go over all `::Ready` projects and check their health.
    // Also syncs the state of all projects on startup
    let ambulance_handle = tokio::spawn({
        let gateway = gateway.clone();
        let sender = sender.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            // Don't try to catch up missed ticks since there is no point running a burst of checks
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                interval.tick().await;

                if sender.capacity() < WORKER_QUEUE_SIZE - SVC_DEGRADED_THRESHOLD {
                    // If degraded, don't stack more health checks.
                    warn!(
                        sender.capacity = sender.capacity(),
                        "skipping health checks"
                    );
                    continue;
                }

                if let Ok(projects) = gateway.iter_projects_ready().await {
                    let span = info_span!(
                        "running health checks",
                        healthcheck.active_projects = projects.len(),
                    );

                    let gateway = gateway.clone();
                    let sender = sender.clone();
                    async move {
                        for (project_name, _) in projects {
                            if let Ok(handle) =
                                gateway.new_task().project(project_name).send(&sender).await
                            {
                                // We wait for the check to be done before
                                // queuing up the next one.
                                handle.await
                            }
                        }
                    }
                    .instrument(span)
                    .await;
                }
            }
        }
    });

    let acme_client = AcmeClient::new();

    let mut api_builder = ApiBuilder::new()
        .with_service(Arc::clone(&gateway))
        .with_sender(sender.clone())
        .binding_to(args.control);

    let mut user_builder = UserServiceBuilder::new()
        .with_service(Arc::clone(&gateway))
        .with_task_sender(sender)
        .with_public(args.context.proxy_fqdn.clone())
        .with_user_proxy_binding_to(args.user)
        .with_bouncer(args.bouncer);

    if let UseTls::Enable = args.use_tls {
        let (resolver, tls_acceptor) = make_tls_acceptor();

        user_builder = user_builder
            .with_acme(acme_client.clone())
            .with_tls(tls_acceptor);

        api_builder = api_builder.with_acme(acme_client.clone(), resolver.clone());

        for CustomDomain {
            fqdn,
            certificate,
            private_key,
            ..
        } in gateway.iter_custom_domains().await.unwrap()
        {
            let mut buf = Vec::new();
            buf.extend(certificate.as_bytes());
            buf.extend(private_key.as_bytes());
            resolver
                .serve_pem(&fqdn.to_string(), Cursor::new(buf))
                .await
                .unwrap();
        }

        tokio::spawn(async move {
            // Make sure we have a certificate for ourselves.
            let certs = gateway
                .fetch_certificate(&acme_client, gateway.credentials())
                .await;
            resolver
                .serve_default_der(certs)
                .await
                .expect("failed to set certs to be served as default");
        });
    } else {
        warn!("TLS is disabled in the proxy service. This is only acceptable in testing, and should *never* be used in deployments.");
    };

    let api_handle = api_builder
        .with_default_routes()
        .with_auth_service(args.context.auth_uri, args.context.admin_key)
        .with_default_traces()
        .with_cors(&args.cors_origin)
        .serve();

    let user_handle = user_builder.serve();

    debug!("starting up all services");

    tokio::select!(
        _ = worker_handle => info!("worker handle finished"),
        _ = api_handle => error!("api handle finished"),
        _ = user_handle => error!("user handle finished"),
        _ = ambulance_handle => error!("ambulance handle finished"),
    );
}
