pub mod args;
pub mod client;
pub mod config;

use tracing::trace;

use crate::{
    args::{Args, Command},
    client::Client,
    config::get_api_key,
};

pub async fn run(args: Args) {
    trace!(?args, "starting with args");

    let api_key = get_api_key();
    let client = Client::new(args.api_url.clone(), api_key, args.client_timeout);

    match args.command {
        Command::ChangeProjectOwner { .. } => {
            unimplemented!();
        }
        Command::RenewCerts => {
            let certs = client.get_old_certificates().await.unwrap();
            eprintln!("Starting renewals of {} certs in 5 seconds...", certs.len());
            tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
            for (cert_id, _) in certs {
                println!("{:?}", client.renew_certificate(&cert_id).await);
                // prevent api rate limiting
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
        }
        Command::UpdateCompute {
            project_id,
            compute_tier,
        } => {
            let res = client
                .update_project_compute_tier(&project_id, compute_tier)
                .await
                .unwrap();
            println!("{res:?}");
        }
        Command::Gc {
            days,
            stop_deployments,
            limit,
        } => {
            let project_ids = client.gc_free_tier(days).await.unwrap();
            gc(client, project_ids, stop_deployments, limit).await;
        }
        Command::GcShuttlings {
            minutes,
            stop_deployments,
            limit,
        } => {
            let project_ids = client.gc_shuttlings(minutes).await.unwrap();
            gc(client, project_ids, stop_deployments, limit).await;
        }
    };
}

async fn gc(client: Client, mut project_ids: Vec<String>, stop_deployments: bool, limit: u32) {
    if !stop_deployments {
        for pid in &project_ids {
            println!("{pid}");
        }
        eprintln!("({} projects)", project_ids.len());
        return;
    }

    project_ids.truncate(limit as usize);
    eprintln!(
        "Starting GC of {} projects in 5 seconds...",
        project_ids.len()
    );
    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
    for pid in project_ids {
        println!("{}", client.inner.stop_service(&pid).await.unwrap());
        // prevent api rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
