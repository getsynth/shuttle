pub mod args;
pub mod client;
pub mod config;

use shuttle_backends::project_name::ProjectName;
use tracing::trace;

use crate::{
    args::{AcmeCommand, Args, Command, StatsCommand},
    client::Client,
    config::get_api_key,
};

pub async fn run(args: Args) {
    trace!(?args, "starting with args");

    let api_key = get_api_key();
    let client = Client::new(args.api_url.clone(), api_key, args.client_timeout);

    match args.command {
        Command::Revive => {
            let s = client.revive().await.expect("revive to succeed");
            println!("{s}");
        }
        Command::Destroy => {
            let s = client.destroy().await.expect("destroy to succeed");
            println!("{s}");
        }
        Command::Acme(AcmeCommand::CreateAccount { email, acme_server }) => {
            let account = client
                .acme_account_create(&email, acme_server)
                .await
                .expect("to create ACME account");

            println!("Details of ACME account are as follow. Keep this safe as it will be needed to create certificates in the future");
            println!("{}", serde_json::to_string_pretty(&account).unwrap());
        }
        Command::Acme(AcmeCommand::Request {
            fqdn,
            project,
            credentials,
        }) => {
            let s = client
                .acme_request_certificate(&fqdn, &project, &credentials)
                .await
                .expect("to get a certificate challenge response");
            println!("{s}");
        }
        Command::Acme(AcmeCommand::RenewCustomDomain {
            fqdn,
            project,
            credentials,
        }) => {
            let s = client
                .acme_renew_custom_domain_certificate(&fqdn, &project, &credentials)
                .await
                .expect("to get a certificate challenge response");
            println!("{s}");
        }
        Command::Acme(AcmeCommand::RenewGateway { credentials }) => {
            let s = client
                .acme_renew_gateway_certificate(&credentials)
                .await
                .expect("to get a certificate challenge response");
            println!("{s}");
        }
        Command::ProjectNames => {
            let projects = client
                .get_projects()
                .await
                .expect("to get list of projects");
            for p in projects {
                if !ProjectName::is_valid(&p.project_name) {
                    println!("{}", p.project_name);
                }
            }
        }
        Command::Stats(StatsCommand::Load { clear }) => {
            let resp = if clear {
                client.clear_load().await.expect("to delete load stats")
            } else {
                client.get_load().await.expect("to get load stats")
            };

            let has_capacity = if resp.has_capacity { "a" } else { "no" };

            println!(
                "Currently {} builds are running and there is {} capacity for new builds",
                resp.builds_count, has_capacity
            )
        }
        Command::ChangeProjectOwner {
            project_name,
            new_user_id,
        } => {
            client
                .change_project_owner(&project_name, &new_user_id)
                .await
                .unwrap();
            println!("Changed project owner: {project_name} -> {new_user_id}")
        }
        Command::SetBetaAccess { user_id } => {
            client.set_beta_access(&user_id, true).await.unwrap();
            println!("Set user {user_id} beta access");
        }
        Command::UnsetBetaAccess { user_id } => {
            client.set_beta_access(&user_id, false).await.unwrap();
            println!("Unset user {user_id} beta access");
        }
        Command::RenewCerts => {
            let res = client.renew_old_certificates().await.unwrap();
            println!("{res}");
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
        println!("{}", client.inner.stop_service_beta(&pid).await.unwrap());
        // prevent api rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
