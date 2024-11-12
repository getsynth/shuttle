use shuttle_admin::{
    args::{AcmeCommand, Args, Command, StatsCommand},
    client::Client,
    config::get_api_key,
};
use shuttle_backends::project_name::ProjectName;
use tracing::trace;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args: Args = clap::Parser::parse();

    trace!(?args, "starting with args");

    let api_key = get_api_key();
    let client = Client::new(args.api_url.clone(), api_key);

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
        Command::IdleCch => {
            client.idle_cch().await.expect("cch projects to be idled");
            println!("Idled CCH projects")
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
    };
}
