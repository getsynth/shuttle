mod args;
pub mod config;
mod init;
mod provisioner_server;
mod suggestions;
mod util;

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs::{read_to_string, File};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use clap::{parser::ValueSource, CommandFactory, FromArgMatches};
use crossterm::style::Stylize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use flate2::write::GzEncoder;
use flate2::Compression;
use futures::{SinkExt, StreamExt, TryFutureExt};
use git2::Repository;
use globset::{Glob, GlobSetBuilder};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use indicatif::ProgressBar;
use indoc::{formatdoc, printdoc};
use reqwest::header::HeaderMap;
use shuttle_api_client::ShuttleApiClient;
use shuttle_common::{
    constants::{
        headers::X_CARGO_SHUTTLE_VERSION, API_URL_DEFAULT, API_URL_DEFAULT_BETA,
        DEFAULT_IDLE_MINUTES, EXAMPLES_REPO, EXECUTABLE_DIRNAME, RESOURCE_SCHEMA_VERSION,
        RUNTIME_NAME, SHUTTLE_IDLE_DOCS_URL, SHUTTLE_LEGACY_NEW_PROJECT, STORAGE_DIRNAME,
        TEMPLATES_SCHEMA_VERSION,
    },
    deployment::{DeploymentStateBeta, DEPLOYER_END_MESSAGES_BAD, DEPLOYER_END_MESSAGES_GOOD},
    log::LogsRange,
    models::{
        auth::{KeyMessage, TokenMessage},
        deployment::{
            deployments_table_beta, get_deployments_table, BuildArgsBeta, BuildArgsRustBeta,
            BuildMetaBeta, DeploymentRequest, DeploymentRequestBeta,
            DeploymentRequestBuildArchiveBeta, DeploymentRequestImageBeta, DeploymentResponseBeta,
            CREATE_SERVICE_BODY_LIMIT, GIT_STRINGS_MAX_LENGTH,
        },
        error::ApiError,
        project::{self, ProjectUpdateRequestBeta},
        resource::{get_certificates_table_beta, get_resource_tables, get_resource_tables_beta},
    },
    resource::{self, ResourceInput, ShuttleResourceOutput},
    semvers_are_compatible, DatabaseResource, DbInput, LogItem, LogItemBeta, VersionInfo,
};
use shuttle_proto::{
    provisioner::{provisioner_server::Provisioner, DatabaseRequest},
    runtime::{self, LoadRequest, StartRequest, StopRequest},
};
use shuttle_service::{
    builder::{async_cargo_metadata, build_workspace, find_shuttle_packages, BuiltService},
    runner, Environment,
};
use strum::{EnumMessage, VariantArray};
use tar::Builder;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::Message;
use tonic::{Request, Status};
use tracing::{debug, error, info, trace};
use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};
use uuid::Uuid;
use zip::write::FileOptions;

use crate::args::{
    CertificateCommand, ConfirmationArgs, DeployArgs, DeploymentCommand, GenerateCommand, InitArgs,
    LoginArgs, LogoutArgs, LogsArgs, ProjectCommand, ProjectStartArgs, ProjectUpdateCommand,
    ResourceCommand, SecretsArgs, TableArgs, TemplateLocation,
};
pub use crate::args::{Command, ProjectArgs, RunArgs, ShuttleArgs};
use crate::config::RequestContext;
use crate::provisioner_server::beta::{ProvApiState, ProvisionerServerBeta};
use crate::provisioner_server::LocalProvisioner;
use crate::util::{
    check_and_warn_runtime_version, generate_completions, generate_manpage, get_templates_schema,
    is_dirty, open_gh_issue, read_ws_until_text, update_cargo_shuttle,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the args and whether the PATH arg of the init command was explicitly given
pub fn parse_args() -> (ShuttleArgs, bool) {
    let matches = ShuttleArgs::command().get_matches();
    let args =
        ShuttleArgs::from_arg_matches(&matches).expect("args to already be parsed successfully");
    let provided_path_to_init = matches
        .subcommand_matches("init")
        .is_some_and(|init_matches| {
            init_matches.value_source("path") == Some(ValueSource::CommandLine)
        });

    (args, provided_path_to_init)
}

pub fn setup_tracing(debug: bool) {
    registry()
        .with(fmt::layer())
        .with(
            // let user set RUST_LOG if they want to
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                if debug {
                    EnvFilter::new("info,cargo_shuttle=trace,shuttle=trace")
                } else {
                    EnvFilter::default()
                }
            }),
        )
        .init();
}

#[derive(PartialEq)]
pub enum Binary {
    CargoShuttle,
    Shuttle,
}

impl Binary {
    pub fn name(&self) -> String {
        match self {
            Self::CargoShuttle => "cargo-shuttle".to_owned(),
            Self::Shuttle => "shuttle".to_owned(),
        }
    }
}

pub struct Shuttle {
    ctx: RequestContext,
    client: Option<ShuttleApiClient>,
    version_info: Option<VersionInfo>,
    /// Strings to print at the end of command execution
    version_warnings: Vec<String>,
    /// Alter behaviour to interact with the new platform
    beta: bool,
    /// Alter behaviour based on which CLI is used
    bin: Binary,
}

impl Shuttle {
    pub fn new(bin: Binary) -> Result<Self> {
        let ctx = RequestContext::load_global()?;
        Ok(Self {
            ctx,
            client: None,
            version_info: None,
            version_warnings: vec![],
            beta: false,
            bin,
        })
    }

    pub async fn run(mut self, mut args: ShuttleArgs, provided_path_to_init: bool) -> Result<()> {
        if self.bin == Binary::Shuttle {
            // beta is always enabled in `shuttle`
            args.beta = true;
        }
        self.beta = args.beta;
        if self.beta {
            if matches!(args.cmd, Command::Project(ProjectCommand::Restart { .. })) {
                bail!("This command is discontinued on the NEW platform (shuttle.dev). Deploy to start a new deployment.");
            }
            if matches!(args.cmd, Command::Status) {
                bail!("This command is discontinued on the NEW platform (shuttle.dev). Use `deployment status` instead.");
            }
            if matches!(
                args.cmd,
                Command::Stop | Command::Project(ProjectCommand::Stop { .. })
            ) {
                bail!("This command is discontinued on the NEW platform (shuttle.dev). Use `deployment stop` instead.");
            }
            if matches!(args.cmd, Command::Clean) {
                bail!("This command is discontinued on the NEW platform (shuttle.dev).");
            }
            if matches!(args.cmd, Command::Resource(ResourceCommand::Dump { .. })) {
                bail!("This command is not yet supported on the NEW platform (shuttle.dev).");
            }
        } else if matches!(
            args.cmd,
            Command::Deployment(DeploymentCommand::Stop)
                | Command::Deployment(DeploymentCommand::Redeploy { .. })
                | Command::Account
                | Command::Project(ProjectCommand::Link)
                | Command::Project(ProjectCommand::Update(..))
        ) {
            bail!("This command is not supported on the OLD platform (shuttle.rs).");
        }

        if !matches!(
            args.cmd,
            // commands that don't differ in behavior in any way between .rs/.dev
            Command::Feedback | Command::Generate(_) | Command::Upgrade { .. }
        ) {
            if self.beta {
                eprintln!("{}", "INFO: Using NEW platform API (shuttle.dev)".green());
            } else {
                eprintln!("{}", "INFO: Using OLD platform API (shuttle.rs)".blue());
            }
        }
        if let Some(ref url) = args.api_url {
            if (!self.beta && url != API_URL_DEFAULT) || (self.beta && url != API_URL_DEFAULT_BETA)
            {
                eprintln!(
                    "{}",
                    format!("INFO: Targeting non-default API: {url}").yellow(),
                );
            }
            if url.ends_with('/') {
                eprintln!("WARNING: API URL is probably incorrect. Ends with '/': {url}");
            }
        }
        self.ctx.set_api_url(args.api_url);

        // All commands that call the API
        if matches!(
            args.cmd,
            Command::Init(..)
                | Command::Deploy(..)
                | Command::Status
                | Command::Logs { .. }
                | Command::Account
                | Command::Login(..)
                | Command::Logout(..)
                | Command::Deployment(..)
                | Command::Resource(..)
                | Command::Certificate(..)
                | Command::Stop
                | Command::Clean
                | Command::Project(..)
        ) || (
            // project linking on beta requires api client
            // TODO: refactor so that beta local run does not need to know project id / always uses crate name ???
            self.beta && matches!(args.cmd, Command::Run(..))
        ) {
            let client = ShuttleApiClient::new(
                self.ctx.api_url(self.beta),
                self.ctx.api_key().ok(),
                Some(
                    HeaderMap::try_from(&HashMap::from([(
                        X_CARGO_SHUTTLE_VERSION.clone(),
                        crate::VERSION.to_owned(),
                    )]))
                    .unwrap(),
                ),
            );
            self.client = Some(client);
            if !args.offline && !self.beta {
                self.check_api_versions().await?;
            }
        }

        // All commands that need to know which project is being handled
        if matches!(
            args.cmd,
            Command::Deploy(..)
                | Command::Deployment(..)
                | Command::Resource(..)
                | Command::Certificate(..)
                | Command::Project(
                    // ProjectCommand::List does not need to know which project we are in
                    ProjectCommand::Start { .. }
                        | ProjectCommand::Update(..)
                        | ProjectCommand::Status { .. }
                        | ProjectCommand::Stop { .. }
                        | ProjectCommand::Restart { .. }
                        | ProjectCommand::Delete { .. }
                        | ProjectCommand::Link
                )
                | Command::Stop
                | Command::Clean
                | Command::Status
                | Command::Logs { .. }
        ) {
            // Command::Run only uses load_local (below) instead of load_project since it does not target a project in the API
            self.load_project(
                &args.project_args,
                matches!(args.cmd, Command::Project(ProjectCommand::Link)),
                // only deploy should create a project if the provided name is not found in the project list.
                // (project start should always make the POST call, it's an upsert operation)
                matches!(args.cmd, Command::Deploy(..)),
            )
            .await?;
        }

        let res = match args.cmd {
            Command::Init(init_args) => {
                self.init(
                    init_args,
                    args.project_args,
                    provided_path_to_init,
                    args.offline,
                )
                .await
            }
            Command::Generate(cmd) => match cmd {
                GenerateCommand::Manpage => generate_manpage(),
                GenerateCommand::Shell { shell, output } => {
                    generate_completions(self.bin, shell, output)
                }
            },
            Command::Account => self.account().await,
            Command::Login(login_args) => self.login(login_args, args.offline).await,
            Command::Logout(logout_args) => self.logout(logout_args).await,
            Command::Feedback => open_gh_issue(),
            Command::Run(run_args) => {
                self.ctx.load_local(&args.project_args)?;
                if self.beta {
                    self.local_run_beta(run_args, args.debug).await
                } else {
                    self.local_run(run_args).await
                }
            }
            Command::Deploy(deploy_args) => self.deploy(deploy_args).await,
            Command::Status => self.status().await,
            Command::Logs(logs_args) => {
                if self.beta {
                    self.logs_beta(logs_args).await
                } else {
                    self.logs(logs_args).await
                }
            }
            Command::Deployment(cmd) => match cmd {
                DeploymentCommand::List { page, limit, table } => {
                    self.deployments_list(page, limit, table).await
                }
                DeploymentCommand::Status { id } => self.deployment_get(id).await,
                DeploymentCommand::Redeploy { id } => self.deployment_redeploy(id).await,
                DeploymentCommand::Stop => self.stop_beta().await,
            },
            Command::Stop => self.stop().await,
            Command::Clean => self.clean().await,
            Command::Resource(cmd) => match cmd {
                ResourceCommand::List {
                    table,
                    show_secrets,
                } => {
                    if self.beta {
                        self.resources_list_beta(table, show_secrets).await
                    } else {
                        self.resources_list(table, show_secrets).await
                    }
                }
                ResourceCommand::Delete {
                    resource_type,
                    confirmation: ConfirmationArgs { yes },
                } => self.resource_delete(&resource_type, yes).await,
                ResourceCommand::Dump { resource_type } => self.resource_dump(&resource_type).await,
            },
            Command::Certificate(cmd) => match cmd {
                CertificateCommand::Add { domain } => self.add_certificate(domain).await,
                CertificateCommand::List { table } => self.list_certificates(table).await,
                CertificateCommand::Delete {
                    domain,
                    confirmation: ConfirmationArgs { yes },
                } => self.delete_certificate(domain, yes).await,
            },
            Command::Project(cmd) => match cmd {
                ProjectCommand::Start(ProjectStartArgs { idle_minutes }) => {
                    if self.beta {
                        self.project_create_beta().await
                    } else {
                        self.project_start(idle_minutes).await
                    }
                }
                ProjectCommand::Update(cmd) => match cmd {
                    ProjectUpdateCommand::Name { name } => self.project_rename_beta(name).await,
                },
                ProjectCommand::Restart(ProjectStartArgs { idle_minutes }) => {
                    self.project_restart(idle_minutes).await
                }
                ProjectCommand::Status { follow } => {
                    if self.beta {
                        self.project_status_beta().await
                    } else {
                        self.project_status(follow).await
                    }
                }
                ProjectCommand::List { table, .. } => self.projects_list(table).await,
                ProjectCommand::Stop => self.project_stop().await,
                ProjectCommand::Delete(ConfirmationArgs { yes }) => {
                    if self.beta {
                        self.project_delete_beta(yes).await
                    } else {
                        self.project_delete(yes).await
                    }
                }
                ProjectCommand::Link => Ok(()), // logic is done in `load_local`
            },
            Command::Upgrade { preview } => update_cargo_shuttle(preview).await,
        };

        for w in self.version_warnings {
            println!("{w}");
        }

        res
    }

    async fn check_api_versions(&mut self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        debug!("Checking API versions");
        if let Ok(versions) = client.get_api_versions().await {
            debug!("Got API versions: {versions:?}");
            self.version_info = Some(versions);

            // check cargo-shuttle version
            // should always be a valid semver
            let my_version = &semver::Version::from_str(VERSION).unwrap();
            let latest_version = &self.version_info.as_ref().unwrap().cargo_shuttle;
            if my_version != latest_version {
                let newer_version_exists = my_version < latest_version;
                let string = if semvers_are_compatible(my_version, latest_version) {
                    newer_version_exists.then(|| {
                        format!("Info: A newer version of cargo-shuttle exists ({latest_version}).")
                    })
                    // Having a newer but compatible version does not show warning
                } else {
                    newer_version_exists.then(||
                        formatdoc! {"
                            Warning:
                                A newer version of cargo-shuttle exists ({latest_version}).
                                It is recommended to upgrade.
                                Refer to the upgrading docs: https://docs.shuttle.rs/configuration/shuttle-versions#upgrading-shuttle-version"
                        }
                    ).or_else(||
                        Some(formatdoc! {"
                            Warning:
                                Your version of cargo-shuttle ({my_version}) is newer than what the API expects ({latest_version}).
                                This means a new release is likely underway!
                                Unexpected behavior can occur until the API is updated."
                        })
                    )
                };
                if let Some(s) = string {
                    self.version_warnings.push(s.yellow().to_string());
                }
            }
        } else {
            debug!("Failed to get API version info");
        }

        Ok(())
    }

    /// Log in, initialize a project and potentially create the Shuttle environment for it.
    ///
    /// If project name, template, and path are passed as arguments, it will run without any extra
    /// interaction.
    async fn init(
        &mut self,
        args: InitArgs,
        mut project_args: ProjectArgs,
        provided_path_to_init: bool,
        offline: bool,
    ) -> Result<()> {
        // Turns the template or git args (if present) to a repo+folder.
        let git_template = args.git_template()?;
        let no_git = args.no_git;

        let needs_name = project_args.name_or_id.is_none();
        let needs_template = git_template.is_none();
        let needs_path = !provided_path_to_init;
        let needs_login = self.ctx.api_key().is_err() && args.login_args.api_key.is_none();
        let interactive = needs_name || needs_template || needs_path || needs_login;

        let theme = ColorfulTheme::default();

        // 1. Log in (if not logged in yet)
        if needs_login {
            println!("First, let's log in to your Shuttle account.");
            self.login(args.login_args.clone(), offline).await?;
            println!();
        } else if args.login_args.api_key.is_some() {
            self.login(args.login_args.clone(), offline).await?;
        }

        // 2. Ask for project name or validate the given one
        if needs_name && !self.beta {
            printdoc! {"
                What do you want to name your project?
                It will be hosted at ${{project_name}}.shuttleapp.rs, so choose something unique!
                "
            };
        }
        let mut prev_name: Option<String> = None;
        loop {
            // prompt if interactive
            let name: String = if let Some(name) = project_args.name_or_id.clone() {
                name
            } else {
                // not using `validate_with` due to being blocking.
                Input::with_theme(&theme)
                    .with_prompt("Project name")
                    .interact()?
            };
            let force_name = args.force_name
                || (needs_name && prev_name.as_ref().is_some_and(|prev| prev == &name));
            if force_name {
                project_args.name_or_id = Some(name);
                break;
            }
            // validate and take action based on result
            if self
                .check_project_name(&mut project_args, name.clone())
                .await
            {
                // success
                break;
            } else if needs_name {
                // try again
                println!(r#"Type the same name again to use "{}" anyways."#, name);
                prev_name = Some(name);
            } else {
                // don't continue if non-interactive
                bail!(
                    "Invalid or unavailable project name. Use `--force-name` to use this project name anyways."
                );
            }
        }
        if needs_name {
            println!();
        }

        // 3. Confirm the project directory
        let path = if needs_path {
            let path = args.path.join(
                project_args
                    .name_or_id
                    .as_ref()
                    .expect("name should be set"),
            );

            loop {
                println!("Where should we create this project?");

                let directory_str: String = Input::with_theme(&theme)
                    .with_prompt("Directory")
                    .default(format!("{}", path.display()))
                    .interact()?;
                println!();

                let path = args::create_and_parse_path(OsString::from(directory_str))?;

                if std::fs::read_dir(&path)
                    .expect("init dir to exist and list entries")
                    .count()
                    > 0
                    && !Confirm::with_theme(&theme)
                        .with_prompt("Target directory is not empty. Are you sure?")
                        .default(true)
                        .interact()?
                {
                    println!();
                    continue;
                }

                break path;
            }
        } else {
            args.path.clone()
        };

        // 4. Ask for the template
        let template = match git_template {
            Some(git_template) => git_template,
            None => {
                // Try to present choices from our up-to-date examples.
                // Fall back to the internal (potentially outdated) list.
                let schema = if offline {
                    None
                } else {
                    get_templates_schema()
                        .await
                        .map_err(|e| {
                            error!(err = %e, "Failed to get templates");
                            println!(
                                "{}",
                                "Failed to look up template list. Falling back to internal list."
                                    .yellow()
                            )
                        })
                        .ok()
                        .and_then(|s| {
                            if s.version == TEMPLATES_SCHEMA_VERSION {
                                return Some(s);
                            }
                            println!(
                                "{}",
                                "Template list with incompatible version found. Consider updating cargo-shuttle. Falling back to internal list."
                                    .yellow()
                            );

                            None
                        })
                };
                if let Some(schema) = schema {
                    println!("What type of project template would you like to start from?");
                    let i = Select::with_theme(&theme)
                        .items(&[
                            "A Hello World app in a supported framework",
                            "Browse our full library of templates", // TODO(when templates page is live): Add link to it?
                        ])
                        .clear(false)
                        .default(0)
                        .interact()?;
                    println!();
                    if i == 0 {
                        // Use a Hello world starter
                        let mut starters = schema.starters.into_values().collect::<Vec<_>>();
                        starters.sort_by_key(|t| {
                            // Make the "No templates" appear last in the list
                            if t.title.starts_with("No") {
                                "zzz".to_owned()
                            } else {
                                t.title.clone()
                            }
                        });
                        let starter_strings = starters
                            .iter()
                            .map(|t| {
                                format!("{} - {}", t.title.clone().bold(), t.description.clone())
                            })
                            .collect::<Vec<_>>();
                        let index = Select::with_theme(&theme)
                            .with_prompt("Select template")
                            .items(&starter_strings)
                            .default(0)
                            .interact()?;
                        println!();
                        let path = starters[index]
                            .path
                            .clone()
                            .expect("starter to have a path");

                        TemplateLocation {
                            auto_path: EXAMPLES_REPO.into(),
                            subfolder: Some(path),
                        }
                    } else {
                        // Browse all non-starter templates
                        let mut templates = schema.templates.into_values().collect::<Vec<_>>();
                        templates.sort_by_key(|t| t.title.clone());
                        let template_strings = templates
                            .iter()
                            .map(|t| {
                                format!(
                                    "{} - {}{}",
                                    t.title.clone().bold(),
                                    t.description.clone(),
                                    t.tags
                                        .first()
                                        .map(|tag| format!(" ({tag})").dim().to_string())
                                        .unwrap_or_default(),
                                )
                            })
                            .collect::<Vec<_>>();
                        let index = Select::with_theme(&theme)
                            .with_prompt("Select template")
                            .items(&template_strings)
                            .default(0)
                            .interact()?;
                        println!();
                        let path = templates[index]
                            .path
                            .clone()
                            .expect("template to have a path");

                        TemplateLocation {
                            auto_path: EXAMPLES_REPO.into(),
                            subfolder: Some(path),
                        }
                    }
                } else {
                    println!("Shuttle works with many frameworks. Which one do you want to use?");
                    let frameworks = args::InitTemplateArg::VARIANTS;
                    let framework_strings = frameworks
                        .iter()
                        .map(|t| {
                            t.get_documentation()
                                .expect("all template variants to have docs")
                        })
                        .collect::<Vec<_>>();
                    let index = Select::with_theme(&theme)
                        .items(&framework_strings)
                        .default(0)
                        .interact()?;
                    println!();
                    frameworks[index].template()
                }
            }
        };

        // 5. Initialize locally
        crate::init::generate_project(
            path.clone(),
            project_args
                .name_or_id
                .as_ref()
                .expect("to have a project name provided"),
            &template,
            no_git,
        )?;
        println!();

        // 6. Confirm that the user wants to create the project environment on Shuttle
        let should_create_environment = if !interactive {
            args.create_env
        } else if args.create_env {
            true
        } else {
            let name = project_args
                .name_or_id
                .as_ref()
                .expect("to have a project name provided");

            let should_create = Confirm::with_theme(&theme)
                .with_prompt(if self.beta {
                    format!(r#"Create a project on Shuttle with the name "{name}"?"#)
                } else {
                    format!(
                        r#"Claim the project name "{name}" by starting a project container on Shuttle?"#
                    )
                })
                .default(true)
                .interact()?;
            if !should_create && !self.beta {
                println!(
                    "Note: The project name will not be claimed until \
                    you start the project with `cargo shuttle project start`."
                )
            }
            println!();
            should_create
        };

        if should_create_environment {
            // Set the project working directory path to the init path,
            // so `load_project` is ran with the correct project path
            project_args.working_directory.clone_from(&path);

            self.load_project(&project_args, true, true).await?;
            if !self.beta {
                self.project_start(DEFAULT_IDLE_MINUTES).await?;
            }
        }

        if std::env::current_dir().is_ok_and(|d| d != path) {
            println!("You can `cd` to the directory, then:");
        }
        if self.beta {
            println!("Run `shuttle run` to run the app locally.");
        } else {
            println!("Run `cargo shuttle run` to run the app locally.");
        }
        if !should_create_environment {
            if self.beta {
                println!("Run `shuttle deploy` to deploy it to Shuttle.");
            } else {
                println!(
                    "Run `cargo shuttle project start` to create a project environment on Shuttle."
                );
                let serenity_idle_hint = template
                    .subfolder
                    .as_ref()
                    .is_some_and(|s| s.contains("serenity") || s.contains("poise"));
                if serenity_idle_hint {
                    printdoc!(
                        "
                        Hint: Discord bots might want to use `--idle-minutes 0` when starting the
                        project so that they don't go offline: {SHUTTLE_IDLE_DOCS_URL}
                        "
                    );
                }
            }
        }

        Ok(())
    }

    /// true -> success/neutral. false -> try again.
    async fn check_project_name(&self, project_args: &mut ProjectArgs, name: String) -> bool {
        let client = self.client.as_ref().unwrap();
        match if self.beta {
            client.check_project_name_beta(&name).await
        } else {
            client.check_project_name(&name).await
        } {
            Ok(true) => {
                // inner value is inverted on beta
                if self.beta {
                    project_args.name_or_id = Some(name);
                    return true;
                }

                println!("{} {}", "Project name already taken:".red(), name);
                println!("{}", "Try a different name.".yellow());

                false
            }
            // not possible on beta
            Ok(false) => {
                project_args.name_or_id = Some(name);

                true
            }
            Err(e) => {
                // If API error contains message regarding format of error name, print that error and prompt again
                if let Ok(api_error) = e.downcast::<ApiError>() {
                    // If the returned error string changes, this could break
                    if api_error.message.contains("Invalid project name") {
                        println!("{}", api_error.message.yellow());
                        println!("{}", "Try a different name.".yellow());
                        return false;
                    }
                }
                // Else, the API error was about something else.
                // Ignore and keep going to not prevent the flow of the init command.
                project_args.name_or_id = Some(name);
                println!(
                    "{}",
                    "Failed to check if project name is available.".yellow()
                );

                true
            }
        }
    }

    pub async fn load_project(
        &mut self,
        project_args: &ProjectArgs,
        do_linking: bool,
        create_missing_beta_project: bool,
    ) -> Result<()> {
        trace!("project arguments: {project_args:?}");

        self.ctx.load_local(project_args)?;
        if self.beta {
            // load project id from file if exists
            self.ctx.load_local_internal(project_args)?;
            if let Some(name) = project_args.name_or_id.as_ref() {
                // uppercase project id
                if let Some(suffix) = name.strip_prefix("proj_") {
                    // Soft (dumb) validation of ULID format in the id (ULIDs are 26 chars)
                    if suffix.len() == 26 {
                        let proj_id_uppercase = format!("proj_{}", suffix.to_ascii_uppercase());
                        if *name != proj_id_uppercase {
                            eprintln!("INFO: Converted project id to '{}'", proj_id_uppercase);
                            self.ctx.set_project_id(proj_id_uppercase);
                        }
                    }
                }
                // translate project name to project id if a name was given
                if !name.starts_with("proj_") {
                    trace!("unprefixed project id found, assuming it's a project name");
                    let client = self.client.as_ref().unwrap();
                    trace!(%name, "looking up project id from project name");
                    if let Some(proj) = client
                        .get_projects_list_beta()
                        .await?
                        .projects
                        .into_iter()
                        .find(|p| p.name == *name)
                    {
                        trace!("found project by name");
                        self.ctx.set_project_id(proj.id);
                    } else {
                        trace!("did not find project by name");
                        if create_missing_beta_project {
                            trace!("creating project since it was not found");
                            let proj = client.create_project_beta(name).await?;
                            eprintln!("Created project '{}' with id {}", proj.name, proj.id);
                            self.ctx.set_project_id(proj.id);
                        }
                    }
                }
                // if called from Link command, command-line override is saved to file
                if do_linking {
                    eprintln!("Linking to project {}", self.ctx.project_id());
                    self.ctx.save_local_internal()?;
                    return Ok(());
                }
            }
            // if project id is still not known or an explicit linking is wanted, start the linking prompt
            if !self.ctx.project_id_found() || do_linking {
                self.project_link(None).await?;
            }
        }

        Ok(())
    }

    async fn project_link(&mut self, id_or_name: Option<String>) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let projs = client.get_projects_list_beta().await?.projects;

        let theme = ColorfulTheme::default();

        let proj = if let Some(id_or_name) = id_or_name {
            projs
                .into_iter()
                .find(|p| p.id == id_or_name || p.name == id_or_name)
                .ok_or(anyhow!("Did not find project '{id_or_name}'."))?
        } else {
            let selected_project = if projs.is_empty() {
                eprintln!("Create a new project to link to this directory:");

                None
            } else {
                eprintln!("Which project do you want to link this directory to?");

                let mut items = projs.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
                items.extend_from_slice(&["[CREATE NEW]".to_string()]);
                let index = Select::with_theme(&theme)
                    .items(&items)
                    .default(0)
                    .interact()?;

                if index == projs.len() {
                    // last item selected (create new)
                    None
                } else {
                    Some(projs[index].clone())
                }
            };

            match selected_project {
                Some(proj) => proj,
                None => {
                    let name: String = Input::with_theme(&theme)
                        .with_prompt("Project name")
                        .interact()?;

                    let proj = client.create_project_beta(&name).await?;
                    eprintln!("Created project '{}' with id {}", proj.name, proj.id);

                    proj
                }
            }
        };

        eprintln!("Linking to project '{}' with id {}", proj.name, proj.id);
        self.ctx.set_project_id(proj.id);
        self.ctx.save_local_internal()?;

        Ok(())
    }

    async fn account(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let user = client.get_current_user_beta().await?;
        print!("{}", user.to_string_colored());

        Ok(())
    }

    /// Log in with the given API key or after prompting the user for one.
    async fn login(&mut self, login_args: LoginArgs, offline: bool) -> Result<()> {
        let api_key = match login_args.api_key {
            Some(api_key) => api_key,
            None => {
                if login_args.prompt || !self.beta {
                    // manual input requested (always the case on shuttle.rs)

                    if !login_args.prompt && !self.beta {
                        // if !beta, open console
                        let url = SHUTTLE_LEGACY_NEW_PROJECT;
                        let _ = webbrowser::open(url);
                        println!("If your browser did not automatically open, go to {url}");
                    }

                    Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("API key")
                        .validate_with(|input: &String| {
                            if input.is_empty() {
                                return Err("Empty API key was provided");
                            }
                            Ok(())
                        })
                        .interact()?
                } else {
                    // device auth flow via Shuttle Console
                    self.device_auth(login_args.console_url).await?
                }
            }
        };

        self.ctx.set_api_key(api_key.clone())?;

        if let Some(client) = self.client.as_mut() {
            client.api_key = Some(api_key);

            if self.beta {
                if offline {
                    eprintln!("INFO: Skipping API key verification");
                } else {
                    let u = client
                        .get_current_user_beta()
                        .await
                        .context("failed to check API key validity")?;
                    println!("Logged in as {} ({})", u.name.bold(), u.id.bold());
                }
            }
        }

        Ok(())
    }

    async fn device_auth(&self, console_url: String) -> Result<String> {
        let client = self.client.as_ref().unwrap();

        // should not have trailing slash
        if console_url.ends_with('/') {
            eprintln!("WARNING: Console URL is probably incorrect. Ends with '/': {console_url}");
        }

        let (mut tx, mut rx) = client.get_device_auth_ws().await?.split();

        // keep the socket alive with ping/pong
        let pinger = tokio::spawn(async move {
            loop {
                if let Err(e) = tx.send(Message::Ping(Vec::new())).await {
                    error!(error = %e, "Error when pinging websocket");
                    break;
                };
                sleep(Duration::from_secs(20)).await;
            }
        });

        let token = read_ws_until_text(&mut rx).await?;
        let Some(token) = token else {
            bail!("Did not receive device auth token over websocket");
        };
        let token = serde_json::from_str::<TokenMessage>(&token)?.token;

        let url = &format!("{}/device-auth?token={}", console_url, token);
        let _ = webbrowser::open(url);
        println!("Complete login in Shuttle Console to authenticate CLI.");
        println!("If your browser did not automatically open, go to {url}");
        println!();
        println!("{}", format!("Token: {token}").bold());
        println!();

        let key = read_ws_until_text(&mut rx).await?;
        let Some(key) = key else {
            bail!("Failed to receive API key over websocket");
        };
        let key = serde_json::from_str::<KeyMessage>(&key)?.api_key;

        pinger.abort();

        Ok(key)
    }

    async fn logout(&mut self, logout_args: LogoutArgs) -> Result<()> {
        if logout_args.reset_api_key {
            self.reset_api_key()
                .await
                .map_err(suggestions::api_key::reset_api_key_failed)?;
            println!("Successfully reset the API key.");
            if self.beta {
                println!(" -> Use `shuttle login` to get a new one.")
            } else {
                println!(" -> Go to {SHUTTLE_LEGACY_NEW_PROJECT} to get a new one.");
            }
            println!();
        }
        self.ctx.clear_api_key()?;
        println!("Successfully logged out.");

        Ok(())
    }

    async fn reset_api_key(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        client.reset_api_key().await.and_then(|res| {
            if res.status().is_success() {
                Ok(())
            } else {
                Err(anyhow!("Resetting API key failed."))
            }
        })
    }

    async fn stop_beta(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let pid = self.ctx.project_id();
        let res = client.stop_service_beta(pid).await?;
        println!("{res}");
        wait_with_spinner(2000, |_, pb| async move {
            let deployment = client.get_current_deployment_beta(pid).await?;

            let get_cleanup = |d: Option<DeploymentResponseBeta>| {
                move || {
                    if let Some(d) = d {
                        println!("{}", d.to_string_colored());
                    }
                }
            };
            let Some(deployment) = deployment else {
                return Ok(Some(get_cleanup(None)));
            };

            let state = deployment.state.clone();
            pb.set_message(deployment.to_string_summary_colored());
            let cleanup = get_cleanup(Some(deployment));
            match state {
                    DeploymentStateBeta::Pending
                    | DeploymentStateBeta::Stopping
                    | DeploymentStateBeta::InProgress
                    | DeploymentStateBeta::Running => Ok(None),
                    DeploymentStateBeta::Building // a building deployment should take it back to InProgress then Running, so don't follow that sequence
                    | DeploymentStateBeta::Failed
                    | DeploymentStateBeta::Stopped
                    | DeploymentStateBeta::Unknown => Ok(Some(cleanup)),
                }
        })
        .await?;

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let p = self.ctx.project_name();
        wait_with_spinner(500, |i, pb| async move {
            let service = if i == 0 {
                client.stop_service(p).await?
            } else {
                client.get_service(p).await?
            };

            let service_str = format!("{service}");
            let cleanup = move || {
                println!("{}", "Successfully stopped service".bold());
                println!("{service_str}");
                println!("Run `cargo shuttle deploy` to re-deploy your service.");
            };

            let Some(ref deployment) = service.deployment else {
                return Ok(Some(cleanup));
            };

            pb.set_message(format!("Stopping {}", deployment.id));

            if deployment.state == shuttle_common::deployment::State::Stopped {
                Ok(Some(cleanup))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(suggestions::deployment::stop_deployment_failure)?;

        Ok(())
    }

    async fn status(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let summary = client.get_service(self.ctx.project_name()).await?;

        println!("{summary}");

        Ok(())
    }

    async fn clean(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let message = client
            .clean_project(self.ctx.project_name())
            .await
            .map_err(|err| {
                suggestions::project::project_request_failure(
                    err,
                    "Project clean failed",
                    true,
                    "cleaning your project or checking its status fail repeatedly",
                )
            })?;
        println!("{message}");

        Ok(())
    }

    async fn logs_beta(&self, args: LogsArgs) -> Result<()> {
        if args.follow {
            eprintln!("Streamed logs are not yet supported on the new platform.");
            return Ok(());
        }
        // TODO: implement logs range
        let client = self.client.as_ref().unwrap();
        let pid = self.ctx.project_id();
        let logs = if args.all_deployments {
            client.get_project_logs_beta(pid).await?.logs
        } else {
            let id = if args.latest {
                // Find latest deployment (not always an active one)
                let deployments = client.get_deployments_beta(pid, 1, 1).await?.deployments;
                let Some(most_recent) = deployments.first() else {
                    println!("No deployments found");
                    return Ok(());
                };
                eprintln!("Getting logs from: {}", most_recent.id);
                most_recent.id.to_string()
            } else if let Some(id) = args.id {
                id
            } else {
                let Some(current) = client.get_current_deployment_beta(pid).await? else {
                    println!("No deployments found");
                    return Ok(());
                };
                eprintln!("Getting logs from: {}", current.id);
                current.id
            };
            client.get_deployment_logs_beta(pid, &id).await?.logs
        };
        for log in logs {
            if args.raw {
                println!("{}", log.line);
            } else {
                println!("{log}");
            }
        }

        Ok(())
    }

    async fn logs(&self, args: LogsArgs) -> Result<()> {
        let range = match (args.head, args.tail, args.all) {
            (Some(num), _, _) => LogsRange::Head(num),
            (_, Some(num), _) => LogsRange::Tail(num),
            (_, _, true) => LogsRange::All,
            _ => LogsRange::Tail(1000),
        };
        let client = self.client.as_ref().unwrap();
        let id = if let Some(id) = args.id {
            id
        } else {
            let proj_name = self.ctx.project_name();

            if args.latest {
                // Find latest deployment (not always an active one)
                let deployments = client
                    .get_deployments(proj_name, 0, 1)
                    .await
                    .map_err(|err| {
                        suggestions::logs::get_logs_failure(
                            err,
                            "Fetching the latest deployment failed",
                        )
                    })?;
                let most_recent = deployments.first().context(format!(
                    "Could not find any deployments for '{proj_name}'. Try passing a deployment ID manually",
                ))?;

                most_recent.id.to_string()
            } else if let Some(deployment) = client.get_service(proj_name).await?.deployment {
                // Active deployment
                deployment.id.to_string()
            } else {
                bail!(
                    "Could not find a running deployment for '{proj_name}'. \
                    Try with '--latest', or pass a deployment ID manually"
                );
            }
        };

        if args.follow {
            let mut stream = client
                .get_logs_ws(self.ctx.project_name(), &id, range)
                .await
                .map_err(|err| {
                    suggestions::logs::get_logs_failure(err, "Connecting to the logs stream failed")
                })?;

            while let Ok(Some(s)) = read_ws_until_text(&mut stream).await {
                match serde_json::from_str::<LogItem>(&s) {
                    Ok(log) => {
                        if args.raw {
                            println!("{}", log.get_raw_line())
                        } else {
                            println!("{log}")
                        }
                    }
                    Err(err) => {
                        debug!(error = %err, "failed to parse message into log item");

                        let message = if let Ok(err) = serde_json::from_str::<ApiError>(&s) {
                            err.to_string()
                        } else {
                            "failed to parse logs, is your cargo-shuttle outdated?".to_string()
                        };

                        bail!(message);
                    }
                }
            }
        } else {
            let logs = client
                .get_logs(self.ctx.project_name(), &id, range)
                .await
                .map_err(|err| {
                    suggestions::logs::get_logs_failure(err, "Fetching the deployment failed")
                })?;

            for log in logs.into_iter() {
                if args.raw {
                    println!("{}", log.get_raw_line())
                } else {
                    println!("{log}")
                }
            }
        }

        Ok(())
    }

    async fn deployments_list(&self, page: u32, limit: u32, table_args: TableArgs) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        if limit == 0 {
            println!();
            return Ok(());
        }
        let limit = limit + 1;

        let proj_name = self.ctx.project_name();

        if self.beta {
            let mut deployments = client
                .get_deployments_beta(self.ctx.project_id(), page as i32, limit as i32)
                .await?
                .deployments;
            let page_hint = if deployments.len() == limit as usize {
                deployments.pop();
                true
            } else {
                false
            };
            let table = deployments_table_beta(&deployments, table_args.raw);

            println!(
                "{}",
                format!("Deployments in project '{}'", proj_name).bold()
            );
            println!("{table}");
            if page_hint {
                println!("View the next page using `--page {}`", page + 1);
            }
        } else {
            let mut deployments = client
                .get_deployments(proj_name, page, limit)
                .await
                .map_err(suggestions::deployment::get_deployments_list_failure)?;
            let page_hint = if deployments.len() == limit as usize {
                deployments.pop();
                true
            } else {
                false
            };
            let table =
                get_deployments_table(&deployments, proj_name, page, table_args.raw, page_hint);
            println!("{table}");

            if deployments.is_empty() {
                println!("Run `cargo shuttle deploy` to deploy your project.");
            } else {
                println!("Run `cargo shuttle logs <id>` to get logs for a given deployment.");
            }
        };

        Ok(())
    }

    async fn deployment_get(&self, deployment_id: Option<String>) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        if self.beta {
            let pid = self.ctx.project_id();
            let deployment = match deployment_id {
                Some(id) => client.get_deployment_beta(pid, &id).await,
                None => {
                    let d = client.get_current_deployment_beta(pid).await?;
                    let Some(d) = d else {
                        println!("No deployment found");
                        return Ok(());
                    };
                    Ok(d)
                }
            }?;
            println!("{}", deployment.to_string_colored());
        } else {
            let deployment_id = deployment_id.expect("deployment id required on alpha platform");
            let deployment = client
                .get_deployment_details(
                    self.ctx.project_name(),
                    &Uuid::from_str(&deployment_id).map_err(|err| {
                        anyhow!("Provided deployment id is not a valid UUID: {err}")
                    })?,
                )
                .await
                .map_err(suggestions::deployment::get_deployment_status_failure)?;

            println!("{deployment}");
        }

        Ok(())
    }

    async fn deployment_redeploy(&self, deployment_id: String) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        let pid = self.ctx.project_id();
        let deployment = client.redeploy_beta(pid, &deployment_id).await?;

        // TODO?: Make it print logs on fail
        self.track_deployment_status_beta(pid, &deployment.id)
            .await?;

        Ok(())
    }

    async fn resources_list(&self, table_args: TableArgs, show_secrets: bool) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let resources = client
            .get_service_resources(self.ctx.project_name())
            .await
            .map_err(suggestions::resources::get_service_resources_failure)?;
        let table = get_resource_tables(
            &resources,
            self.ctx.project_name(),
            table_args.raw,
            show_secrets,
        );

        println!("{table}");

        Ok(())
    }

    async fn resources_list_beta(&self, table_args: TableArgs, show_secrets: bool) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let pid = self.ctx.project_id();
        let resources = client.get_service_resources_beta(pid).await?.resources;
        let table =
            get_resource_tables_beta(resources.as_slice(), pid, table_args.raw, show_secrets);

        println!("{table}");

        Ok(())
    }

    async fn resource_delete(
        &self,
        resource_type: &resource::Type,
        no_confirm: bool,
    ) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        if !no_confirm {
            println!(
                "{}",
                formatdoc!(
                    "
                WARNING:
                    Are you sure you want to delete this project's {}?
                    This action is permanent.",
                    resource_type
                )
                .bold()
                .red()
            );
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Are you sure?")
                .default(false)
                .interact()
                .unwrap()
            {
                return Ok(());
            }
        }

        if self.beta {
            let msg = client
                .delete_service_resource_beta(self.ctx.project_id(), resource_type)
                .await?;
            println!("{msg}");
        } else {
            client
                .delete_service_resource(self.ctx.project_name(), resource_type)
                .await?;
            println!("Deleted resource {resource_type}");
        }

        println!(
            "{}",
            formatdoc! {"
                Note:
                    Remember to remove the resource annotation from your #[shuttle_runtime::main] function.
                    Otherwise, it will be provisioned again during the next deployment."
            }
            .yellow(),
        );

        Ok(())
    }

    async fn resource_dump(&self, resource_type: &resource::Type) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        let bytes = client
            .dump_service_resource(self.ctx.project_name(), resource_type)
            .await?;

        std::io::stdout().write_all(&bytes).unwrap();

        Ok(())
    }

    async fn list_certificates(&self, table_args: TableArgs) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let certs = client
            .list_certificates_beta(self.ctx.project_id())
            .await?
            .certificates;

        let table = get_certificates_table_beta(certs.as_ref(), table_args.raw);
        println!("{}", table);

        Ok(())
    }
    async fn add_certificate(&self, domain: String) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let cert = client
            .add_certificate_beta(self.ctx.project_id(), domain.clone())
            .await?;

        println!("Added certificate for {}", cert.subject);

        Ok(())
    }
    async fn delete_certificate(&self, domain: String, no_confirm: bool) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        if !no_confirm {
            println!(
                "{}",
                formatdoc!(
                    "
                WARNING:
                    Delete the certificate for {}?",
                    domain
                )
                .bold()
                .red()
            );
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Are you sure?")
                .default(false)
                .interact()
                .unwrap()
            {
                return Ok(());
            }
        }

        let msg = client
            .delete_certificate_beta(self.ctx.project_id(), domain.clone())
            .await?;
        println!("{msg}");

        Ok(())
    }

    fn get_secrets(run_args: &RunArgs, service: &BuiltService) -> Result<HashMap<String, String>> {
        let secrets_file = run_args.secret_args.secrets.clone().or_else(|| {
            let crate_dir = service.crate_directory();
            // Prioritise crate-local prod secrets over workspace dev secrets (in the rare case that both exist)
            [
                crate_dir.join("Secrets.dev.toml"),
                crate_dir.join("Secrets.toml"),
                service.workspace_path.join("Secrets.dev.toml"),
                service.workspace_path.join("Secrets.toml"),
            ]
            .into_iter()
            .find(|f| f.exists() && f.is_file())
        });
        let secrets = if let Some(secrets_file) = secrets_file {
            trace!("Loading secrets from {}", secrets_file.display());
            if let Ok(secrets_str) = read_to_string(secrets_file) {
                let secrets = toml::from_str::<HashMap<String, String>>(&secrets_str)?;
                trace!(keys = ?secrets.keys(), "available secrets");
                secrets
            } else {
                trace!("No secrets were loaded");
                Default::default()
            }
        } else {
            trace!("No secrets file was found");
            Default::default()
        };

        Ok(secrets)
    }

    fn get_secrets_beta(
        args: &SecretsArgs,
        workspace_root: &Path,
    ) -> Result<Option<HashMap<String, String>>> {
        // Look for a secrets file, first in the command args, then in the root of the workspace.
        let secrets_file = args.secrets.clone().or_else(|| {
            let secrets_file = workspace_root.join("Secrets.toml");

            if secrets_file.exists() && secrets_file.is_file() {
                Some(secrets_file)
            } else {
                None
            }
        });

        Ok(if let Some(secrets_file) = secrets_file {
            trace!("Loading secrets from {}", secrets_file.display());
            if let Ok(secrets_str) = read_to_string(&secrets_file) {
                let secrets = toml::from_str::<HashMap<String, String>>(&secrets_str)?;

                trace!(keys = ?secrets.keys(), "available secrets");

                Some(secrets)
            } else {
                trace!("No secrets were loaded");
                None
            }
        } else {
            trace!("No secrets file was found");
            None
        })
    }

    async fn spin_local_runtime(
        run_args: &RunArgs,
        service: &BuiltService,
        idx: u16,
    ) -> Result<Option<(Child, runtime::Client)>> {
        let secrets = Shuttle::get_secrets(run_args, service)?;

        trace!(path = ?service.executable_path, "runtime executable");

        if let Some(warning) = check_and_warn_runtime_version(&service.executable_path).await? {
            eprint!("{}", warning);
        }

        let runtime_executable = service.executable_path.clone();
        let port =
            portpicker::pick_unused_port().expect("unable to find available port for gRPC server");
        // Child process and gRPC client for sending requests to it
        let (mut runtime, mut runtime_client) =
            runner::start(port, runtime_executable, service.workspace_path.as_path()).await?;

        let service_name = service.service_name()?;
        let deployment_id: Uuid = Default::default();

        let child_stdout = runtime
            .stdout
            .take()
            .context("child process did not have a handle to stdout")?;
        let mut reader = BufReader::new(child_stdout).lines();
        let service_name_clone = service_name.clone();
        let raw = run_args.raw;
        tokio::spawn(async move {
            while let Some(line) = reader.next_line().await.unwrap() {
                let log_item = LogItem::new(
                    deployment_id,
                    shuttle_common::log::Backend::Runtime(service_name_clone.clone()),
                    line,
                );

                if raw {
                    println!("{}", log_item.get_raw_line())
                } else {
                    println!("{log_item}")
                }
            }
        });

        //
        // LOADING PHASE
        //

        let load_request = tonic::Request::new(LoadRequest {
            project_name: service_name.to_string(),
            env: Environment::Local.to_string(),
            secrets: secrets.clone(),
            path: service
                .executable_path
                .clone()
                .into_os_string()
                .into_string()
                .expect("to convert path to string"),
            ..Default::default()
        });

        trace!("loading service");
        let response = runtime_client
            .load(load_request)
            .or_else(|err| async {
                runtime.kill().await?;
                Err(err)
            })
            .await?
            .into_inner();

        if !response.success {
            error!(error = response.message, "failed to load your service");
            return Ok(None);
        }

        //
        // PROVISIONING PHASE
        //

        let resources = response.resources;
        let (resources, mocked_responses) =
            Shuttle::local_provision_phase(service_name.as_str(), resources, secrets).await?;

        println!(
            "{}",
            get_resource_tables(&mocked_responses, service_name.as_str(), false, false,)
        );

        //
        // START PHASE
        //

        let addr = SocketAddr::new(
            if run_args.external {
                Ipv4Addr::UNSPECIFIED // 0.0.0.0
            } else {
                Ipv4Addr::LOCALHOST // 127.0.0.1
            }
            .into(),
            run_args.port + idx,
        );

        println!(
            "    {} {} on http://{}\n",
            "Starting".bold().green(),
            service_name,
            addr
        );

        let start_request = StartRequest {
            ip: addr.to_string(),
            resources,
        };

        trace!(?start_request, "starting service");
        let response = runtime_client
            .start(tonic::Request::new(start_request))
            .or_else(|err| async {
                runtime.kill().await?;
                Err(err)
            })
            .await?
            .into_inner();

        trace!(response = ?response,  "client response: ");
        Ok(Some((runtime, runtime_client)))
    }

    async fn local_provision_phase(
        project_name: &str,
        mut resources: Vec<Vec<u8>>,
        secrets: HashMap<String, String>,
    ) -> Result<(Vec<Vec<u8>>, Vec<resource::Response>)> {
        // for displaying the tables
        let mut mocked_responses: Vec<resource::Response> = Vec::new();
        let prov = LocalProvisioner::new()?;

        // Fail early if any bytes is invalid json
        let values = resources
            .iter()
            .map(|bytes| {
                serde_json::from_slice::<ResourceInput>(bytes)
                    .context("deserializing resource input")
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        for (bytes, shuttle_resource) in
            resources
                .iter_mut()
                .zip(values)
                // ignore non-Shuttle resource items
                .filter_map(|(bytes, value)| match value {
                    ResourceInput::Shuttle(shuttle_resource) => Some((bytes, shuttle_resource)),
                    ResourceInput::Custom(_) => None,
                })
                .map(|(bytes, shuttle_resource)| {
                    if shuttle_resource.version == RESOURCE_SCHEMA_VERSION {
                        Ok((bytes, shuttle_resource))
                    } else {
                        Err(anyhow!("
                            Shuttle resource request for {} with incompatible version found. Expected {}, found {}. \
                            Make sure that this deployer and the Shuttle resource are up to date.
                            ",
                            shuttle_resource.r#type,
                            RESOURCE_SCHEMA_VERSION,
                            shuttle_resource.version
                        ))
                    }
                }).collect::<anyhow::Result<Vec<_>>>()?.into_iter()
        {
            match shuttle_resource.r#type {
                resource::Type::Database(db_type) => {
                    let config: DbInput = serde_json::from_value(shuttle_resource.config)
                        .context("deserializing resource config")?;
                    let res = match config.local_uri {
                        Some(local_uri) => DatabaseResource::ConnectionString(local_uri),
                        None => DatabaseResource::Info(
                            prov.provision_database(Request::new(DatabaseRequest {
                                project_name: project_name.to_string(),
                                db_type: Some(db_type.into()),
                                db_name: config.db_name,
                            }))
                            .await
                            .context("Failed to start database container. Make sure that a Docker engine is running.")?
                            .into_inner()
                            .into(),
                        ),
                    };
                    mocked_responses.push(resource::Response {
                        r#type: shuttle_resource.r#type,
                        config: serde_json::Value::Null,
                        data: serde_json::to_value(&res).unwrap(),
                    });
                    *bytes = serde_json::to_vec(&ShuttleResourceOutput {
                        output: res,
                        custom: shuttle_resource.custom,
                    })
                    .unwrap();
                }
                resource::Type::Secrets => {
                    // We already know the secrets at this stage, they are not provisioned like other resources
                    mocked_responses.push(resource::Response {
                        r#type: shuttle_resource.r#type,
                        config: serde_json::Value::Null,
                        data: serde_json::to_value(secrets.clone()).unwrap(),
                    });
                    *bytes = serde_json::to_vec(&ShuttleResourceOutput {
                        output: secrets.clone(),
                        custom: shuttle_resource.custom,
                    })
                    .unwrap();
                }
                resource::Type::Persist => {
                    // only show that this resource is "connected"
                    mocked_responses.push(resource::Response {
                        r#type: shuttle_resource.r#type,
                        config: serde_json::Value::Null,
                        data: serde_json::Value::Null,
                    });
                }
                resource::Type::Container => {
                    let config = serde_json::from_value(shuttle_resource.config)
                        .context("deserializing resource config")?;
                    let res = prov.start_container(config).await.context("Failed to start Docker container. Make sure that a Docker engine is running.")?;
                    *bytes = serde_json::to_vec(&ShuttleResourceOutput {
                        output: res,
                        custom: shuttle_resource.custom,
                    })
                    .unwrap();
                }
            }
        }

        Ok((resources, mocked_responses))
    }

    async fn stop_runtime(
        runtime: &mut Child,
        runtime_client: &mut runtime::Client,
    ) -> Result<(), Status> {
        let stop_request = StopRequest {};
        trace!(?stop_request, "stopping service");
        let response = runtime_client
            .stop(tonic::Request::new(stop_request))
            .or_else(|err| async {
                runtime.kill().await?;
                trace!(status = ?err, "killed the runtime by force because stopping it errored out");
                Err(err)
            })
            .await?
            .into_inner();
        trace!(response = ?response, "client stop response: ");
        Ok(())
    }

    async fn add_runtime_info(
        runtime: Option<(Child, runtime::Client)>,
        existing_runtimes: &mut Vec<(Child, runtime::Client)>,
    ) -> Result<(), Status> {
        match runtime {
            Some(inner) => {
                trace!("Adding runtime PID: {:?}", inner.0.id());
                existing_runtimes.push(inner);
            }
            None => {
                trace!("Runtime error: No runtime process. Crashed during startup?");
                for rt_info in existing_runtimes {
                    let mut errored_out = false;
                    // Stopping all runtimes gracefully first, but if this errors out the function kills the runtime forcefully.
                    Shuttle::stop_runtime(&mut rt_info.0, &mut rt_info.1)
                        .await
                        .unwrap_or_else(|_| {
                            errored_out = true;
                        });

                    // If the runtime stopping is successful, we still need to kill it forcefully because we exit outside the loop
                    // and destructors will not be guaranteed to run.
                    if !errored_out {
                        rt_info.0.kill().await?;
                    }
                }
                exit(1);
            }
        };
        Ok(())
    }

    async fn pre_local_run(&self, run_args: &RunArgs) -> Result<Vec<BuiltService>> {
        trace!("starting a local run with args: {run_args:?}");

        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(256);
        tokio::task::spawn(async move {
            while let Some(line) = rx.recv().await {
                println!("{line}");
            }
        });

        let working_directory = self.ctx.working_directory();

        trace!("building project");
        println!(
            "{} {}",
            "    Building".bold().green(),
            working_directory.display()
        );

        build_workspace(working_directory, run_args.release, tx, false).await
    }

    fn find_available_port(run_args: &mut RunArgs, services_len: usize) {
        let default_port = run_args.port;
        'outer: for port in (run_args.port..=u16::MAX).step_by(services_len.max(10)) {
            for inner_port in port..(port + services_len as u16) {
                if !portpicker::is_free_tcp(inner_port) {
                    continue 'outer;
                }
            }
            run_args.port = port;
            break;
        }

        if run_args.port != default_port
            && !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Port {} is already in use. Would you like to continue on port {}?",
                    default_port, run_args.port
                ))
                .default(true)
                .interact()
                .unwrap()
        {
            exit(0);
        }
    }
    fn find_available_port_beta(run_args: &mut RunArgs) {
        let original_port = run_args.port;
        for port in (run_args.port..=u16::MAX).step_by(10) {
            if !portpicker::is_free_tcp(port) {
                continue;
            }
            run_args.port = port;
            break;
        }

        if run_args.port != original_port {
            eprintln!(
                "Port {} is already in use. Using port {}.",
                original_port, run_args.port,
            )
        };
    }

    #[cfg(target_family = "unix")]
    async fn local_run(&self, mut run_args: RunArgs) -> Result<()> {
        let services = self.pre_local_run(&run_args).await?;

        let mut sigterm_notif =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Can not get the SIGTERM signal receptor");
        let mut sigint_notif =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
                .expect("Can not get the SIGINT signal receptor");

        // Start all the services.
        let mut runtimes: Vec<(Child, runtime::Client)> = Vec::new();

        Shuttle::find_available_port(&mut run_args, services.len());

        let mut signal_received = false;
        for (i, service) in services.iter().enumerate() {
            // We must cover the case of starting multiple workspace services and receiving a signal in parallel.
            // This must stop all the existing runtimes and creating new ones.
            signal_received = tokio::select! {
                res = Shuttle::spin_local_runtime(&run_args, service, i as u16) => {
                    match res {
                        Ok(runtime) => {
                            Shuttle::add_runtime_info(runtime, &mut runtimes).await?;
                        },
                        Err(e) => println!("Error while starting service: {e:?}"),
                    }
                    false
                },
                _ = sigterm_notif.recv() => {
                    println!(
                        "cargo-shuttle received SIGTERM. Killing all the runtimes..."
                    );
                    true
                },
                _ = sigint_notif.recv() => {
                    println!(
                        "cargo-shuttle received SIGINT. Killing all the runtimes..."
                    );
                    true
                }
            };

            if signal_received {
                break;
            }
        }

        // If prior signal received is set to true we must stop all the existing runtimes and
        // exit the `local_run`.
        if signal_received {
            for (mut rt, mut rt_client) in runtimes {
                Shuttle::stop_runtime(&mut rt, &mut rt_client)
                    .await
                    .unwrap_or_else(|err| {
                        trace!(status = ?err, "stopping the runtime errored out");
                    });
            }
            return Ok(());
        }

        // If no signal was received during runtimes initialization, then we must handle each runtime until
        // completion and handle the signals during this time.
        for (mut rt, mut rt_client) in runtimes {
            // If we received a signal while waiting for any runtime we must stop the rest and exit
            // the waiting loop.
            if signal_received {
                Shuttle::stop_runtime(&mut rt, &mut rt_client)
                    .await
                    .unwrap_or_else(|err| {
                        trace!(status = ?err, "stopping the runtime errored out");
                    });
                continue;
            }

            // Receiving a signal will stop the current runtime we're waiting for.
            signal_received = tokio::select! {
                res = rt.wait() => {
                    println!(
                        "a service future completed with exit status: {:?}",
                        res.unwrap().code()
                    );
                    false
                },
                _ = sigterm_notif.recv() => {
                    println!(
                        "cargo-shuttle received SIGTERM. Killing all the runtimes..."
                    );
                    Shuttle::stop_runtime(&mut rt, &mut rt_client).await.unwrap_or_else(|err| {
                        trace!(status = ?err, "stopping the runtime errored out");
                    });
                    true
                },
                _ = sigint_notif.recv() => {
                    println!(
                        "cargo-shuttle received SIGINT. Killing all the runtimes..."
                    );
                    Shuttle::stop_runtime(&mut rt, &mut rt_client).await.unwrap_or_else(|err| {
                        trace!(status = ?err, "stopping the runtime errored out");
                    });
                    true
                }
            };
        }

        println!(
            "Run `cargo shuttle project start` to create a project environment on Shuttle.\n\
             Run `cargo shuttle deploy` to deploy your Shuttle service."
        );

        Ok(())
    }

    async fn local_run_beta(&self, mut run_args: RunArgs, debug: bool) -> Result<()> {
        let project_name = self.ctx.project_name().to_owned();
        let services = self.pre_local_run(&run_args).await?;
        let service = services
            .first()
            .expect("at least one shuttle service")
            .to_owned();

        trace!(path = ?service.executable_path, "runtime executable");

        let secrets = Shuttle::get_secrets(&run_args, &service)?;
        Shuttle::find_available_port_beta(&mut run_args);
        if let Some(warning) = check_and_warn_runtime_version(&service.executable_path).await? {
            eprint!("{}", warning);
        }

        let runtime_executable = service.executable_path.clone();
        let api_port = portpicker::pick_unused_port()
            .expect("failed to find available port for local provisioner server");
        let api_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), api_port);
        let ip = if run_args.external {
            Ipv4Addr::UNSPECIFIED
        } else {
            Ipv4Addr::LOCALHOST
        };

        let state = Arc::new(ProvApiState {
            project_name: project_name.clone(),
            secrets,
        });
        ProvisionerServerBeta::start(state, &api_addr);

        println!(
            "\n    {} {} on http://{}:{}\n",
            "Starting".bold().green(),
            service.package_name,
            ip,
            run_args.port,
        );

        info!(
            path = %runtime_executable.display(),
            "Spawning runtime process",
        );
        let mut runtime = tokio::process::Command::new(
            dunce::canonicalize(runtime_executable).context("canonicalize path of executable")?,
        )
        .current_dir(&service.workspace_path)
        .envs([
            ("SHUTTLE_BETA", "true"),
            ("SHUTTLE_PROJECT_ID", "proj_LOCAL"),
            ("SHUTTLE_PROJECT_NAME", project_name.as_str()),
            ("SHUTTLE_ENV", Environment::Local.to_string().as_str()),
            ("SHUTTLE_RUNTIME_IP", ip.to_string().as_str()),
            ("SHUTTLE_RUNTIME_PORT", run_args.port.to_string().as_str()),
            (
                "SHUTTLE_API",
                format!("http://127.0.0.1:{}", api_port).as_str(),
            ),
            (
                "RUST_LOG",
                if debug {
                    "info,shuttle=trace,reqwest=debug"
                } else {
                    "info"
                },
            ),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .context("spawning runtime process")?;

        let raw = run_args.raw;
        let mut stdout_reader = BufReader::new(
            runtime
                .stdout
                .take()
                .context("child process did not have a handle to stdout")?,
        )
        .lines();
        tokio::spawn(async move {
            while let Some(line) = stdout_reader.next_line().await.unwrap() {
                if raw {
                    println!("{}", line);
                } else {
                    let log_item = LogItemBeta::new(Utc::now(), "app".to_owned(), line);
                    println!("{log_item}");
                }
            }
        });
        let mut stderr_reader = BufReader::new(
            runtime
                .stderr
                .take()
                .context("child process did not have a handle to stderr")?,
        )
        .lines();
        tokio::spawn(async move {
            while let Some(line) = stderr_reader.next_line().await.unwrap() {
                if raw {
                    println!("{}", line);
                } else {
                    let log_item = LogItemBeta::new(Utc::now(), "app".to_owned(), line);
                    println!("{log_item}");
                }
            }
        });

        #[cfg(target_family = "unix")]
        let exit_result = {
            let mut sigterm_notif =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Can not get the SIGTERM signal receptor");
            let mut sigint_notif =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
                    .expect("Can not get the SIGINT signal receptor");
            tokio::select! {
                exit_result = runtime.wait() => {
                    Some(exit_result)
                }
                _ = sigterm_notif.recv() => {
                    eprintln!("cargo-shuttle received SIGTERM. Killing the runtime...");
                    None
                },
                _ = sigint_notif.recv() => {
                    eprintln!("cargo-shuttle received SIGINT. Killing the runtime...");
                    None
                }
            }
        };
        #[cfg(target_family = "windows")]
        let exit_result = {
            let mut ctrl_break_notif = tokio::signal::windows::ctrl_break()
                .expect("Can not get the CtrlBreak signal receptor");
            let mut ctrl_c_notif =
                tokio::signal::windows::ctrl_c().expect("Can not get the CtrlC signal receptor");
            let mut ctrl_close_notif = tokio::signal::windows::ctrl_close()
                .expect("Can not get the CtrlClose signal receptor");
            let mut ctrl_logoff_notif = tokio::signal::windows::ctrl_logoff()
                .expect("Can not get the CtrlLogoff signal receptor");
            let mut ctrl_shutdown_notif = tokio::signal::windows::ctrl_shutdown()
                .expect("Can not get the CtrlShutdown signal receptor");
            tokio::select! {
                exit_result = runtime.wait() => {
                    Some(exit_result)
                }
                _ = ctrl_break_notif.recv() => {
                    eprintln!("cargo-shuttle received ctrl-break.");
                    None
                },
                _ = ctrl_c_notif.recv() => {
                    eprintln!("cargo-shuttle received ctrl-c.");
                    None
                },
                _ = ctrl_close_notif.recv() => {
                    eprintln!("cargo-shuttle received ctrl-close.");
                    None
                },
                _ = ctrl_logoff_notif.recv() => {
                    eprintln!("cargo-shuttle received ctrl-logoff.");
                    None
                },
                _ = ctrl_shutdown_notif.recv() => {
                    eprintln!("cargo-shuttle received ctrl-shutdown.");
                    None
                }
            }
        };
        match exit_result {
            Some(Ok(exit_status)) => {
                bail!(
                    "Runtime process exited with code {}",
                    exit_status.code().unwrap_or_default()
                );
            }
            Some(Err(e)) => {
                bail!("Failed to wait for runtime process to exit: {e}");
            }
            None => {
                runtime.kill().await?;
            }
        }

        Ok(())
    }

    #[cfg(target_family = "windows")]
    async fn handle_signals() -> bool {
        let mut ctrl_break_notif = tokio::signal::windows::ctrl_break()
            .expect("Can not get the CtrlBreak signal receptor");
        let mut ctrl_c_notif =
            tokio::signal::windows::ctrl_c().expect("Can not get the CtrlC signal receptor");
        let mut ctrl_close_notif = tokio::signal::windows::ctrl_close()
            .expect("Can not get the CtrlClose signal receptor");
        let mut ctrl_logoff_notif = tokio::signal::windows::ctrl_logoff()
            .expect("Can not get the CtrlLogoff signal receptor");
        let mut ctrl_shutdown_notif = tokio::signal::windows::ctrl_shutdown()
            .expect("Can not get the CtrlShutdown signal receptor");

        tokio::select! {
            _ = ctrl_break_notif.recv() => {
                println!("cargo-shuttle received ctrl-break.");
                true
            },
            _ = ctrl_c_notif.recv() => {
                println!("cargo-shuttle received ctrl-c.");
                true
            },
            _ = ctrl_close_notif.recv() => {
                println!("cargo-shuttle received ctrl-close.");
                true
            },
            _ = ctrl_logoff_notif.recv() => {
                println!("cargo-shuttle received ctrl-logoff.");
                true
            },
            _ = ctrl_shutdown_notif.recv() => {
                println!("cargo-shuttle received ctrl-shutdown.");
                true
            }
            else => {
                false
            }
        }
    }

    #[cfg(target_family = "windows")]
    async fn local_run(&self, mut run_args: RunArgs) -> Result<()> {
        let services = self.pre_local_run(&run_args).await?;

        // Start all the services.
        let mut runtimes: Vec<(Child, runtime::Client)> = Vec::new();

        Shuttle::find_available_port(&mut run_args, services.len());

        let mut signal_received = false;
        for (i, service) in services.iter().enumerate() {
            signal_received = tokio::select! {
                res = Shuttle::spin_local_runtime(&run_args, service, i as u16) => {
                    Shuttle::add_runtime_info(res.unwrap(), &mut runtimes).await?;
                    false
                },
                _ = Shuttle::handle_signals() => {
                    println!(
                        "Killing all the runtimes..."
                    );
                    true
                }
            };

            if signal_received {
                break;
            }
        }

        // If prior signal received is set to true we must stop all the existing runtimes and
        // exit the `local_run`.
        if signal_received {
            for (mut rt, mut rt_client) in runtimes {
                Shuttle::stop_runtime(&mut rt, &mut rt_client)
                    .await
                    .unwrap_or_else(|err| {
                        trace!(status = ?err, "stopping the runtime errored out");
                    });
            }
            return Ok(());
        }

        // If no signal was received during runtimes initialization, then we must handle each runtime until
        // completion and handle the signals during this time.
        for (mut rt, mut rt_client) in runtimes {
            // If we received a signal while waiting for any runtime we must stop the rest and exit
            // the waiting loop.
            if signal_received {
                Shuttle::stop_runtime(&mut rt, &mut rt_client)
                    .await
                    .unwrap_or_else(|err| {
                        trace!(status = ?err, "stopping the runtime errored out");
                    });
                continue;
            }

            // Receiving a signal will stop the current runtime we're waiting for.
            signal_received = tokio::select! {
                res = rt.wait() => {
                    println!(
                        "a service future completed with exit status: {:?}",
                        res.unwrap().code()
                    );
                    false
                },
                _ = Shuttle::handle_signals() => {
                    println!(
                        "Killing all the runtimes..."
                    );
                    Shuttle::stop_runtime(&mut rt, &mut rt_client).await.unwrap_or_else(|err| {
                        trace!(status = ?err, "stopping the runtime errored out");
                    });
                    true
                }
            };
        }

        println!(
            "Run `cargo shuttle project start` to create a project environment on Shuttle.\n\
             Run `cargo shuttle deploy` to deploy your Shuttle service."
        );

        Ok(())
    }

    async fn deploy(&mut self, args: DeployArgs) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let working_directory = self.ctx.working_directory();
        let manifest_path = working_directory.join("Cargo.toml");
        let project_name = self.ctx.project_name();

        let secrets = if self.beta {
            Shuttle::get_secrets_beta(&args.secret_args, working_directory)?
        } else {
            None
        };

        // Beta: Image deployment mode
        if self.beta {
            if let Some(image) = args.image {
                let pid = self.ctx.project_id();
                let deployment_req_image_beta = DeploymentRequestImageBeta { image, secrets };

                let deployment = client
                    .deploy_beta(pid, DeploymentRequestBeta::Image(deployment_req_image_beta))
                    .await?;

                if args.no_follow {
                    println!("{}", deployment.to_string_colored());
                    return Ok(());
                }

                // TODO?: Make it print logs on fail
                self.track_deployment_status_beta(pid, &deployment.id)
                    .await?;

                return Ok(());
            }
        }

        // Alpha and beta: Build archive deployment mode
        let mut deployment_req = DeploymentRequest {
            no_test: args.no_test,
            ..Default::default()
        };
        let mut deployment_req_beta = DeploymentRequestBuildArchiveBeta {
            secrets,
            ..Default::default()
        };

        if self.beta {
            let mut rust_build_args = BuildArgsRustBeta::default();

            let metadata = async_cargo_metadata(manifest_path.as_path()).await?;
            let packages = find_shuttle_packages(&metadata)?;
            // TODO: support overriding this
            let package = packages
                .first()
                .expect("Expected at least one crate with shuttle-runtime in the workspace");
            let package_name = package.name.to_owned();
            rust_build_args.package_name = Some(package_name);

            // activate shuttle feature if present
            let (no_default_features, features) = if package.features.contains_key("shuttle") {
                (true, Some(vec!["shuttle".to_owned()]))
            } else {
                (false, None)
            };
            rust_build_args.no_default_features = no_default_features;
            rust_build_args.features = features.map(|v| v.join(","));

            rust_build_args.shuttle_runtime_version = package
                .dependencies
                .iter()
                .find(|dependency| dependency.name == RUNTIME_NAME)
                .expect("shuttle package to have runtime dependency")
                .req
                .comparators
                .first()
                // is "^0.X.0" when `shuttle-runtime = "0.X.0"` is in Cargo.toml
                .and_then(|c| c.to_string().strip_prefix('^').map(ToOwned::to_owned));

            // TODO: determine which (one) binary to build

            deployment_req_beta.build_args = Some(BuildArgsBeta::Rust(rust_build_args));

            // TODO: have all of the above be configurable in CLI and Shuttle.toml
        }

        if let Ok(repo) = Repository::discover(working_directory) {
            let repo_path = repo
                .workdir()
                .context("getting working directory of repository")?;
            let repo_path = dunce::canonicalize(repo_path)?;
            trace!(?repo_path, "found git repository");

            let dirty = is_dirty(&repo);
            deployment_req.git_dirty = Some(dirty.is_err());

            let check_dirty = !self.beta || self.ctx.deny_dirty().is_some_and(|d| d);
            if check_dirty && !args.allow_dirty && dirty.is_err() {
                bail!(dirty.unwrap_err());
            }

            if let Ok(head) = repo.head() {
                // This is typically the name of the current branch
                // It is "HEAD" when head detached, for example when a tag is checked out
                deployment_req.git_branch = head
                    .shorthand()
                    .map(|s| s.chars().take(GIT_STRINGS_MAX_LENGTH).collect());
                if let Ok(commit) = head.peel_to_commit() {
                    deployment_req.git_commit_id = Some(commit.id().to_string());
                    // Summary is None if error or invalid utf-8
                    deployment_req.git_commit_msg = commit
                        .summary()
                        .map(|s| s.chars().take(GIT_STRINGS_MAX_LENGTH).collect());
                }
            }
        }

        if self.beta {
            eprintln!("Packing files...");
        }
        let archive = self.make_archive(args.secret_args.secrets.clone(), self.beta)?;

        if let Some(path) = args.output_archive {
            eprintln!("Writing archive to {}", path.display());
            std::fs::write(path, archive).context("writing archive")?;

            return Ok(());
        }

        if !self.beta && archive.len() > CREATE_SERVICE_BODY_LIMIT {
            bail!(
                r#"The project is too large - the limit is {} MB. \
                Your project archive is {:.1} MB. \
                Run with `cargo shuttle --debug` to see which files are being packed."#,
                CREATE_SERVICE_BODY_LIMIT / 1_000_000,
                archive.len() as f32 / 1_000_000f32,
            );
        }

        // End early for beta
        if self.beta {
            // TODO: upload secrets separately

            let pid = self.ctx.project_id();

            eprintln!("Uploading code...");
            let arch = client.upload_archive_beta(pid, archive).await?;
            deployment_req_beta.archive_version_id = arch.archive_version_id;
            deployment_req_beta.build_meta = Some(BuildMetaBeta {
                git_commit_id: deployment_req.git_commit_id,
                git_commit_msg: deployment_req.git_commit_msg,
                git_branch: deployment_req.git_branch,
                git_dirty: deployment_req.git_dirty,
            });

            eprintln!("Creating deployment...");
            let deployment = client
                .deploy_beta(
                    pid,
                    DeploymentRequestBeta::BuildArchive(deployment_req_beta),
                )
                .await?;

            if args.no_follow {
                println!("{}", deployment.to_string_colored());
                return Ok(());
            }

            if self
                .track_deployment_status_beta(pid, &deployment.id)
                .await?
            {
                for log in client
                    .get_deployment_logs_beta(pid, &deployment.id)
                    .await?
                    .logs
                {
                    if args.raw {
                        println!("{}", log.line);
                    } else {
                        println!("{log}");
                    }
                }
            }

            return Ok(());
        }

        deployment_req.data = archive;
        let deployment = client
            .deploy(project_name, deployment_req)
            .await
            .map_err(suggestions::deploy::deploy_request_failure)?;

        let mut stream = client
            .get_logs_ws(project_name, &deployment.id.to_string(), LogsRange::All)
            .await
            .map_err(|err| {
                suggestions::deploy::deployment_setup_failure(
                    err,
                    "Connecting to the deployment logs failed",
                )
            })?;

        let mut deployer_version_checked = false;
        let mut runtime_version_checked = false;
        while let Ok(Some(s)) = read_ws_until_text(&mut stream).await {
            let log_item = match serde_json::from_str::<LogItem>(&s) {
                Ok(log_item) => log_item,
                Err(err) => {
                    debug!(error = %err, "failed to parse message into log item");

                    let message = if let Ok(err) = serde_json::from_str::<ApiError>(&s) {
                        err.to_string()
                    } else {
                        "failed to parse logs, is your cargo-shuttle outdated?".to_string()
                    };

                    bail!(message);
                }
            };

            if args.raw {
                println!("{}", log_item.get_raw_line())
            } else {
                println!("{log_item}")
            }

            // Detect versions of deployer and runtime, and print warnings of outdated.
            if !deployer_version_checked
                && self.version_info.is_some()
                && log_item.line.contains("Deployer version: ")
            {
                deployer_version_checked = true;
                let my_version = &log_item
                    .line
                    .split_once("Deployer version: ")
                    .unwrap()
                    .1
                    .parse::<semver::Version>()
                    .context("parsing deployer version in log stream")?;
                let latest_version = &self.version_info.as_ref().unwrap().deployer;
                if latest_version > my_version {
                    self.version_warnings.push(
                                formatdoc! {"
                                    Warning:
                                        A newer version of shuttle-deployer is available ({latest_version}).
                                        Use `cargo shuttle project restart` to upgrade."
                                }
                                .yellow()
                                .to_string(),
                            )
                }
            }
            if !runtime_version_checked
                && self.version_info.is_some()
                && log_item
                    .line
                    .contains("shuttle-runtime executable started (version ")
            {
                runtime_version_checked = true;
                let my_version = &log_item
                    .line
                    .split_once("shuttle-runtime executable started (version ")
                    .unwrap()
                    .1
                    .split_once(')')
                    .unwrap()
                    .0
                    .parse::<semver::Version>()
                    .context("parsing runtime version in log stream")?;
                let latest_version = &self.version_info.as_ref().unwrap().runtime;
                if latest_version > my_version {
                    self.version_warnings.push(
                                formatdoc! {"
                                    Warning:
                                        A newer version of shuttle-runtime is available ({latest_version}).
                                        Update it and any other shuttle dependencies in Cargo.toml."
                                }
                                .yellow()
                                .to_string(),
                            )
                }
            }

            // Determine when to stop listening to the log stream
            if DEPLOYER_END_MESSAGES_BAD
                .iter()
                .any(|m| log_item.line.contains(m))
            {
                println!();
                println!("{}", "Deployment crashed".red());
                println!();
                println!("Run the following for more details");
                println!();
                println!("cargo shuttle logs {}", &deployment.id);

                bail!("");
            }
            if DEPLOYER_END_MESSAGES_GOOD
                .iter()
                .any(|m| log_item.line.contains(m))
            {
                debug!("received end message, breaking deployment stream");
                break;
            }
        }

        // Temporary fix.
        // TODO: Make get_service_summary endpoint wait for a bit and see if it entered Running/Crashed state.
        // Note: Will otherwise be possible when health checks are supported
        sleep(Duration::from_millis(500)).await;

        let deployment = client
            .get_deployment_details(project_name, &deployment.id)
            .await
            .map_err(|err| {
                suggestions::deploy::deployment_setup_failure(
                    err,
                    "Assessing deployment state failed",
                )
            })?;

        // A deployment will only exist if there is currently one in the running state
        if deployment.state != shuttle_common::deployment::State::Running {
            println!("{}", "Deployment has not entered the running state".red());
            println!();

            match deployment.state {
                shuttle_common::deployment::State::Stopped => {
                    println!("State: Stopped - Deployment was running, but has been stopped by the user.")
                }
                shuttle_common::deployment::State::Completed => {
                    println!("State: Completed - Deployment was running, but stopped running all by itself.")
                }
                shuttle_common::deployment::State::Unknown => {
                    println!("State: Unknown - Deployment was in an unknown state. We never expect this state and entering this state should be considered a bug.")
                }
                shuttle_common::deployment::State::Crashed => {
                    println!(
                        "{}",
                        "State: Crashed - Deployment crashed after startup.".red()
                    );
                }
                state => {
                    debug!("deployment logs stream received state: {state} when it expected to receive running state");
                    println!(
                        "Deployment entered an unexpected state - Please create a ticket to report this."
                    );
                }
            }

            println!();
            println!("Run the following for more details");
            println!();
            println!("cargo shuttle logs {}", &deployment.id);

            bail!("");
        }

        let service = client.get_service(project_name).await?;
        let resources = client.get_service_resources(project_name).await?;
        let resources = get_resource_tables(&resources, project_name, false, false);

        println!("{resources}{service}");

        Ok(())
    }

    /// Returns true if the deployment failed
    async fn track_deployment_status_beta(&self, pid: &str, id: &str) -> Result<bool> {
        let client = self.client.as_ref().unwrap();
        let failed = wait_with_spinner(2000, |_, pb| async move {
            let deployment = client.get_deployment_beta(pid, id).await?;

            let state = deployment.state.clone();
            pb.set_message(deployment.to_string_summary_colored());
            let failed = state == DeploymentStateBeta::Failed;
            let cleanup = move || {
                println!("{}", deployment.to_string_colored());
                failed
            };
            match state {
                DeploymentStateBeta::Pending
                | DeploymentStateBeta::Building
                | DeploymentStateBeta::InProgress => Ok(None),
                DeploymentStateBeta::Running
                | DeploymentStateBeta::Stopped
                | DeploymentStateBeta::Stopping
                | DeploymentStateBeta::Unknown
                | DeploymentStateBeta::Failed => Ok(Some(cleanup)),
            }
        })
        .await?;

        Ok(failed)
    }

    async fn project_start(&self, idle_minutes: u64) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let config = &project::Config { idle_minutes };

        let p = self.ctx.project_name();
        wait_with_spinner(500, |i, pb| async move {
            let project = if i == 0 {
                client.create_project(p, config).await?
            } else {
                client.get_project(p).await?
            };
            pb.set_message(format!("{project}"));

            let done = [
                project::State::Ready,
                project::State::Errored {
                    message: Default::default(),
                },
            ]
            .contains(&project.state);

            if done {
                Ok(Some(move || {
                    println!("{project}");
                }))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|err| {
            suggestions::project::project_request_failure(
                err,
                "Project creation failed",
                true,
                "the project creation or retrieving the status fails repeatedly",
            )
        })?;

        if idle_minutes > 0 && !self.beta {
            let idle_msg = format!(
                "Your project will sleep if it is idle for {} minutes.",
                idle_minutes
            );
            println!("{}", idle_msg.yellow());
            println!("To change the idle time refer to the docs: {SHUTTLE_IDLE_DOCS_URL}");
            println!();
        }

        println!("Run `cargo shuttle deploy --allow-dirty` to deploy your Shuttle service.");

        Ok(())
    }

    async fn project_create_beta(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let name = self.ctx.project_name();
        let project = client.create_project_beta(name).await?;

        println!("Created project '{}' with id {}", project.name, project.id);

        Ok(())
    }
    async fn project_rename_beta(&self, name: String) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        let project = client
            .update_project_beta(
                self.ctx.project_id(),
                ProjectUpdateRequestBeta {
                    name: Some(name),
                    ..Default::default()
                },
            )
            .await?;

        println!("Renamed project {} to {}", project.id, project.name);

        Ok(())
    }

    async fn project_restart(&self, idle_minutes: u64) -> Result<()> {
        self.project_stop()
            .await
            .map_err(suggestions::project::project_restart_failure)?;
        self.project_start(idle_minutes)
            .await
            .map_err(suggestions::project::project_restart_failure)?;

        Ok(())
    }

    async fn projects_list(&self, table_args: TableArgs) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        let projects_table = if self.beta {
            project::get_projects_table_beta(
                &client.get_projects_list_beta().await?.projects,
                table_args.raw,
            )
        } else {
            project::get_projects_table(
                &client.get_projects_list().await.map_err(|err| {
                    suggestions::project::project_request_failure(
                        err,
                        "Getting projects list failed",
                        false,
                        "getting the projects list fails repeatedly",
                    )
                })?,
                table_args.raw,
            )
        };

        println!("{}", "Personal Projects".bold());
        println!("{projects_table}\n");

        if !self.beta {
            let teams = client.get_teams_list().await?;

            for team in teams {
                let team_projects = client.get_team_projects_list(&team.id).await?;
                let team_projects_table =
                    project::get_projects_table(&team_projects, table_args.raw);

                println!("{}", format!("{}'s Projects", team.display_name).bold());
                println!("{team_projects_table}\n");
            }
        }

        Ok(())
    }

    async fn project_status(&self, follow: bool) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        if follow {
            let p = self.ctx.project_name();
            wait_with_spinner(500, |_, pb| async move {
                let project = client.get_project(p).await?;
                pb.set_message(format!("{project}"));

                let done = [
                    project::State::Ready,
                    project::State::Destroyed,
                    project::State::Errored {
                        message: Default::default(),
                    },
                ]
                .contains(&project.state);

                if done {
                    Ok(Some(move || {
                        println!("{project}");
                    }))
                } else {
                    Ok(None)
                }
            })
            .await?;
        } else {
            let project = client
                .get_project(self.ctx.project_name())
                .await
                .map_err(|err| {
                    suggestions::project::project_request_failure(
                        err,
                        "Getting project status failed",
                        false,
                        "getting project status failed repeatedly",
                    )
                })?;
            println!(
                "{project}\nIdle minutes: {}",
                project
                    .idle_minutes
                    .map(|i| i.to_string())
                    .unwrap_or("<unknown>".to_owned())
            );
        }

        Ok(())
    }
    async fn project_status_beta(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let project = client.get_project_beta(self.ctx.project_id()).await?;
        print!("{}", project.to_string_colored());

        Ok(())
    }

    async fn project_stop(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        let p = self.ctx.project_name();
        wait_with_spinner(500, |i, pb| async move {
            let project = if i == 0 {
                client.stop_project(p).await?
            } else {
                client.get_project(p).await?
            };
            pb.set_message(format!("{project}"));

            let done = [
                project::State::Destroyed,
                project::State::Errored {
                    message: Default::default(),
                },
            ]
            .contains(&project.state);

            if done {
                Ok(Some(move || {
                    println!("{project}");
                }))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|err| {
            suggestions::project::project_request_failure(
                err,
                "Project stop failed",
                true,
                "stopping the project or getting project status fails repeatedly",
            )
        })?;
        println!("Run `cargo shuttle project start` to recreate project environment on Shuttle.");

        Ok(())
    }

    async fn project_delete_beta(&self, no_confirm: bool) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let pid = self.ctx.project_id();

        if !no_confirm {
            println!(
                "{}",
                formatdoc!(
                    r#"
                    WARNING:
                        Are you sure you want to delete "{pid}"?
                        This will...
                        - Shut down you service.
                        - Delete any databases and secrets in this project.
                        - Delete any custom domains linked to this project.
                        This action is permanent."#
                )
                .bold()
                .red()
            );
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Are you sure?")
                .default(false)
                .interact()
                .unwrap()
            {
                return Ok(());
            }
        }

        let res = client.delete_project_beta(pid).await?;

        println!("{res}");

        Ok(())
    }

    async fn project_delete(&self, no_confirm: bool) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        if !no_confirm {
            println!(
                "{}",
                formatdoc!(
                    r#"
                    WARNING:
                        Are you sure you want to delete "{}"?
                        This will...
                        - Delete any databases, secrets, and shuttle-persist data in this project.
                        - Delete any custom domains linked to this project.
                        - Release the project name from your account.
                        This action is permanent."#,
                    self.ctx.project_name()
                )
                .bold()
                .red()
            );
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Are you sure?")
                .default(false)
                .interact()
                .unwrap()
            {
                return Ok(());
            }
        }

        client
            .delete_project(self.ctx.project_name())
            .await
            .map_err(|err| {
                suggestions::project::project_request_failure(
                    err,
                    "Project delete failed",
                    true,
                    "deleting the project or getting project status fails repeatedly",
                )
            })?;

        println!("Deleted project");

        Ok(())
    }

    fn make_archive(&self, secrets_file: Option<PathBuf>, zip: bool) -> Result<Vec<u8>> {
        let include_patterns = self.ctx.include();

        let working_directory = self.ctx.working_directory();

        //
        // Mixing include and exclude overrides messes up the .ignore and .gitignore etc,
        // therefore these "ignore" walk and the "include" walk are separate.
        //
        let mut entries = Vec::new();

        // Default excludes
        let ignore_overrides = OverrideBuilder::new(working_directory)
            .add("!.git/")
            .context("adding override `!.git/`")?
            .add("!target/")
            .context("adding override `!target/`")?
            // these should always be ignored when unpacked in deployment, so ignore them here as well
            .add(&format!("!{EXECUTABLE_DIRNAME}/"))
            .context(format!("adding override `!{EXECUTABLE_DIRNAME}/`"))?
            .add(&format!("!{STORAGE_DIRNAME}/"))
            .context(format!("adding override `!{STORAGE_DIRNAME}/`"))?
            .build()
            .context("building archive override rules")?;
        for r in WalkBuilder::new(working_directory)
            .hidden(false)
            .overrides(ignore_overrides)
            .build()
        {
            entries.push(r.context("list dir entry")?.into_path())
        }

        let mut globs = GlobSetBuilder::new();

        if let Some(secrets_file) = secrets_file.clone() {
            entries.push(secrets_file);
        } else {
            // Default: Include all Secrets.toml files
            globs.add(Glob::new("**/Secrets.toml").unwrap());
        }

        // User provided includes
        if let Some(rules) = include_patterns {
            for r in rules {
                globs.add(Glob::new(r.as_str()).context(format!("parsing glob pattern {:?}", r))?);
            }
        }

        // Find the files
        let globs = globs.build().context("glob glob")?;
        for entry in walkdir::WalkDir::new(working_directory) {
            let path = entry.context("list dir")?.into_path();
            if globs.is_match(
                path.strip_prefix(working_directory)
                    .context("strip prefix of path")?,
            ) {
                entries.push(path);
            }
        }

        let mut archive_files = BTreeMap::new();
        for path in entries {
            // It's not possible to add a directory to an archive
            // and symlinks == chaos
            if path.is_dir() || path.is_symlink() {
                trace!("Skipping {:?}", path);
                continue;
            }

            // zip file puts all files in root, tar puts all files nested in a dir at root level
            let prefix = if zip {
                working_directory
            } else {
                working_directory.parent().context("get parent dir")?
            };
            let mut name = path
                .strip_prefix(prefix)
                .context("strip prefix of path")?
                .to_owned();

            // if this is the custom secrets file, rename it to Secrets.toml
            if secrets_file.as_ref().is_some_and(|sf| sf == &path) {
                name.pop();
                name.push("Secrets.toml");
            }

            archive_files.insert(path, name);
        }

        if archive_files.is_empty() {
            error!("No files included in upload. Aborting...");
            bail!("No files included in upload.");
        }

        let bytes = if zip {
            debug!("making zip archive");
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            for (path, name) in archive_files {
                debug!("Packing {path:?}");

                // windows things
                let name = name.to_str().expect("valid filename").replace('\\', "/");
                zip.start_file(name, FileOptions::<()>::default())?;

                let mut b = Vec::new();
                File::open(path)?.read_to_end(&mut b)?;
                zip.write_all(&b)?;
            }
            let r = zip.finish().context("finish encoding zip archive")?;

            r.into_inner()
        } else {
            debug!("making tar archive");
            let encoder = GzEncoder::new(Vec::new(), Compression::new(3));
            let mut tar = Builder::new(encoder);
            for (path, name) in archive_files {
                debug!("Packing {path:?}");
                tar.append_path_with_name(path, name)?;
            }
            let encoder = tar.into_inner().context("get encoder from tar archive")?;

            encoder.finish().context("finish encoding tar archive")?
        };
        debug!("Archive size: {} bytes", bytes.len());

        Ok(bytes)
    }
}

/// Calls async function `f` in a loop with `millis` sleep between iterations,
/// providing iteration count and reference to update the progress bar.
/// `f` returns Some with a cleanup function if done.
/// The cleanup function is called after teardown of progress bar,
/// and its return value is returned from here.
async fn wait_with_spinner<Fut, C, O>(
    millis: u64,
    f: impl Fn(usize, ProgressBar) -> Fut,
) -> Result<O, anyhow::Error>
where
    Fut: std::future::Future<Output = Result<Option<C>>>,
    C: FnOnce() -> O,
{
    let progress_bar = create_spinner();
    let mut count = 0usize;
    let cleanup = loop {
        if let Some(cleanup) = f(count, progress_bar.clone()).await? {
            break cleanup;
        }
        count += 1;
        sleep(Duration::from_millis(millis)).await;
    };
    progress_bar.finish_and_clear();

    Ok(cleanup())
}

fn create_spinner() -> ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.enable_steady_tick(std::time::Duration::from_millis(250));
    pb.set_style(
        indicatif::ProgressStyle::with_template("{spinner:.orange} {msg}")
            .unwrap()
            .tick_strings(&[
                "( ●    )",
                "(  ●   )",
                "(   ●  )",
                "(    ● )",
                "(     ●)",
                "(    ● )",
                "(   ●  )",
                "(  ●   )",
                "( ●    )",
                "(●     )",
                "(●●●●●●)",
            ]),
    );

    pb
}

#[cfg(test)]
mod tests {
    use flate2::read::GzDecoder;
    use tar::Archive;
    use zip::ZipArchive;

    use crate::args::{DeployArgs, ProjectArgs, SecretsArgs};
    use crate::Shuttle;
    use std::fs::{self, canonicalize};
    use std::io::Cursor;
    use std::path::PathBuf;

    pub fn path_from_workspace_root(path: &str) -> PathBuf {
        let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("..")
            .join(path);

        dunce::canonicalize(path).unwrap()
    }

    async fn get_archive_entries(
        project_args: ProjectArgs,
        deploy_args: DeployArgs,
        zip: bool,
    ) -> Vec<String> {
        let mut shuttle = Shuttle::new(crate::Binary::CargoShuttle).unwrap();
        shuttle
            .load_project(&project_args, false, false)
            .await
            .unwrap();

        let archive = shuttle
            .make_archive(deploy_args.secret_args.secrets, zip)
            .unwrap();

        if zip {
            let mut zip = ZipArchive::new(Cursor::new(archive)).unwrap();
            (0..zip.len())
                .map(|i| zip.by_index(i).unwrap().name().to_owned())
                .collect()
        } else {
            let tar = GzDecoder::new(&archive[..]);
            let mut archive = Archive::new(tar);

            archive
                .entries()
                .unwrap()
                .map(|entry| {
                    entry
                        .unwrap()
                        .path()
                        .unwrap()
                        .components()
                        .skip(1)
                        .collect::<PathBuf>()
                        .display()
                        .to_string()
                })
                .collect()
        }
    }

    #[tokio::test]
    async fn make_archive_respect_rules() {
        let working_directory = canonicalize(path_from_workspace_root(
            "cargo-shuttle/tests/resources/archiving",
        ))
        .unwrap();

        fs::write(working_directory.join("Secrets.toml"), "KEY = 'value'").unwrap();
        fs::write(working_directory.join("Secrets.dev.toml"), "KEY = 'dev'").unwrap();
        fs::write(working_directory.join("asset2"), "").unwrap();
        fs::write(working_directory.join("asset4"), "").unwrap();
        fs::create_dir_all(working_directory.join("dist")).unwrap();
        fs::write(working_directory.join("dist").join("dist1"), "").unwrap();

        fs::create_dir_all(working_directory.join("target")).unwrap();
        fs::write(working_directory.join("target").join("binary"), b"12345").unwrap();

        let project_args = ProjectArgs {
            working_directory: working_directory.clone(),
            name_or_id: Some("archiving-test".to_owned()),
        };
        let mut entries =
            get_archive_entries(project_args.clone(), Default::default(), false).await;
        entries.sort();

        let expected = vec![
            ".gitignore",
            ".ignore",
            "Cargo.toml",
            "Secrets.toml", // always included by default
            "Secrets.toml.example",
            "Shuttle.toml",
            "asset1", // normal file
            "asset2", // .gitignore'd, but included in Shuttle.toml
            // asset3 is .ignore'd
            "asset4",                // .gitignore'd, but un-ignored in .ignore
            "asset5",                // .ignore'd, but included in Shuttle.toml
            "dist/dist1",            // .gitignore'd, but included in Shuttle.toml
            "nested/static/nested1", // normal file
            // nested/static/nestedignore is .gitignore'd
            "src/main.rs",
        ];
        assert_eq!(entries, expected);

        // check that zip behaves the same way
        let mut entries = get_archive_entries(project_args.clone(), Default::default(), true).await;
        entries.sort();
        assert_eq!(entries, expected);

        fs::remove_file(working_directory.join("Secrets.toml")).unwrap();
        let mut entries = get_archive_entries(
            project_args,
            DeployArgs {
                secret_args: SecretsArgs {
                    secrets: Some(working_directory.join("Secrets.toml.example")),
                },
                ..Default::default()
            },
            false,
        )
        .await;
        entries.sort();

        assert_eq!(
            entries,
            vec![
                ".gitignore",
                ".ignore",
                "Cargo.toml",
                "Secrets.toml", // got moved here
                // Secrets.toml.example was the given secrets file, so it got moved
                "Shuttle.toml",
                "asset1", // normal file
                "asset2", // .gitignore'd, but included in Shuttle.toml
                // asset3 is .ignore'd
                "asset4",                // .gitignore'd, but un-ignored in .ignore
                "asset5",                // .ignore'd, but included in Shuttle.toml
                "dist/dist1",            // .gitignore'd, but included in Shuttle.toml
                "nested/static/nested1", // normal file
                // nested/static/nestedignore is .gitignore'd
                "src/main.rs",
            ]
        );
    }

    #[tokio::test]
    async fn finds_workspace_root() {
        let project_args = ProjectArgs {
            working_directory: path_from_workspace_root("examples/axum/hello-world/src"),
            name_or_id: None,
        };

        let mut shuttle = Shuttle::new(crate::Binary::CargoShuttle).unwrap();
        shuttle
            .load_project(&project_args, false, false)
            .await
            .unwrap();

        assert_eq!(
            project_args.working_directory,
            path_from_workspace_root("examples/axum/hello-world/src")
        );
        assert_eq!(
            project_args.workspace_path().unwrap(),
            path_from_workspace_root("examples/axum/hello-world")
        );
    }
}
