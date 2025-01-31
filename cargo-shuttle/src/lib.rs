mod args;
pub mod builder;
pub mod config;
mod init;
mod provisioner_server;
mod util;

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs::{read_to_string, File};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use args::DeploymentTrackingArgs;
use chrono::Utc;
use clap::{parser::ValueSource, CommandFactory, FromArgMatches};
use crossterm::style::Stylize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use futures::{SinkExt, StreamExt};
use git2::Repository;
use globset::{Glob, GlobSetBuilder};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use indicatif::ProgressBar;
use indoc::formatdoc;
use reqwest::header::HeaderMap;
use shuttle_api_client::ShuttleApiClient;
use shuttle_common::{
    constants::{
        headers::X_CARGO_SHUTTLE_VERSION, API_URL_DEFAULT_BETA, EXAMPLES_REPO, RUNTIME_NAME,
        STORAGE_DIRNAME, TEMPLATES_SCHEMA_VERSION,
    },
    models::{
        auth::{KeyMessage, TokenMessage},
        deployment::{
            BuildArgs, BuildArgsRust, BuildMeta, DeploymentRequest, DeploymentRequestBuildArchive,
            DeploymentRequestImage, DeploymentResponse, DeploymentState, Environment,
            GIT_STRINGS_MAX_LENGTH,
        },
        error::ApiError,
        log::LogItem,
        project::ProjectUpdateRequest,
        resource::ResourceType,
    },
    tables::{deployments_table, get_certificates_table, get_projects_table, get_resource_tables},
};
use strum::{EnumMessage, VariantArray};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, trace};
use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};
use zip::write::FileOptions;

use crate::args::{
    CertificateCommand, ConfirmationArgs, DeployArgs, DeploymentCommand, GenerateCommand, InitArgs,
    LoginArgs, LogoutArgs, LogsArgs, ProjectCommand, ProjectUpdateCommand, ResourceCommand,
    SecretsArgs, TableArgs, TemplateLocation,
};
pub use crate::args::{Command, ProjectArgs, RunArgs, ShuttleArgs};
use crate::builder::{async_cargo_metadata, build_workspace, find_shuttle_packages, BuiltService};
use crate::config::RequestContext;
use crate::provisioner_server::{ProvApiState, ProvisionerServer};
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
    /// Alter behaviour based on which CLI is used
    bin: Binary,
}

impl Shuttle {
    pub fn new(bin: Binary) -> Result<Self> {
        let ctx = RequestContext::load_global()?;
        Ok(Self {
            ctx,
            client: None,
            bin,
        })
    }

    pub async fn run(mut self, args: ShuttleArgs, provided_path_to_init: bool) -> Result<()> {
        if matches!(args.cmd, Command::Resource(ResourceCommand::Dump { .. })) {
            bail!("This command is not yet supported on the NEW platform (shuttle.dev).");
        }

        if let Some(ref url) = args.api_url {
            if url != API_URL_DEFAULT_BETA {
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
                | Command::Logs { .. }
                | Command::Account
                | Command::Login(..)
                | Command::Logout(..)
                | Command::Deployment(..)
                | Command::Resource(..)
                | Command::Certificate(..)
                | Command::Project(..)
        ) || (
            // project linking on beta requires api client
            // TODO: refactor so that beta local run does not need to know project id / always uses crate name ???
            matches!(args.cmd, Command::Run(..))
        ) {
            let client = ShuttleApiClient::new(
                self.ctx.api_url(),
                self.ctx.api_key().ok(),
                Some(
                    HeaderMap::try_from(&HashMap::from([(
                        X_CARGO_SHUTTLE_VERSION.clone(),
                        crate::VERSION.to_owned(),
                    )]))
                    .unwrap(),
                ),
                None,
            );
            self.client = Some(client);
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
                    ProjectCommand::Create
                        | ProjectCommand::Update(..)
                        | ProjectCommand::Status { .. }
                        | ProjectCommand::Delete { .. }
                        | ProjectCommand::Link
                )
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

        match args.cmd {
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
                self.local_run(run_args, args.debug).await
            }
            Command::Deploy(deploy_args) => self.deploy(deploy_args).await,
            Command::Logs(logs_args) => self.logs(logs_args).await,
            Command::Deployment(cmd) => match cmd {
                DeploymentCommand::List { page, limit, table } => {
                    self.deployments_list(page, limit, table).await
                }
                DeploymentCommand::Status { id } => self.deployment_get(id).await,
                DeploymentCommand::Redeploy { id, tracking_args } => {
                    self.deployment_redeploy(id, tracking_args).await
                }
                DeploymentCommand::Stop { tracking_args } => self.stop(tracking_args).await,
            },
            Command::Resource(cmd) => match cmd {
                ResourceCommand::List {
                    table,
                    show_secrets,
                } => self.resources_list(table, show_secrets).await,
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
                ProjectCommand::Create => self.project_create().await,
                ProjectCommand::Update(cmd) => match cmd {
                    ProjectUpdateCommand::Name { name } => self.project_rename(name).await,
                },
                ProjectCommand::Status => self.project_status().await,
                ProjectCommand::List { table, .. } => self.projects_list(table).await,
                ProjectCommand::Delete(ConfirmationArgs { yes }) => self.project_delete(yes).await,
                ProjectCommand::Link => Ok(()), // logic is done in `load_local`
            },
            Command::Upgrade { preview } => update_cargo_shuttle(preview).await,
        }
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
                                "Template list with incompatible version found. Consider upgrading Shuttle CLI. Falling back to internal list."
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
                .with_prompt(format!(
                    r#"Create a project on Shuttle with the name "{name}"?"#
                ))
                .default(true)
                .interact()?;
            println!();
            should_create
        };

        if should_create_environment {
            // Set the project working directory path to the init path,
            // so `load_project` is ran with the correct project path
            project_args.working_directory.clone_from(&path);

            self.load_project(&project_args, true, true).await?;
        }

        if std::env::current_dir().is_ok_and(|d| d != path) {
            println!("You can `cd` to the directory, then:");
        }
        println!("Run `shuttle run` to run the app locally.");
        if !should_create_environment {
            println!("Run `shuttle deploy` to deploy it to Shuttle.");
        }

        Ok(())
    }

    /// Return value: true -> success or unknown. false -> try again.
    async fn check_project_name(&self, project_args: &mut ProjectArgs, name: String) -> bool {
        let client = self.client.as_ref().unwrap();
        match client.check_project_name(&name).await {
            Ok(true) => {
                project_args.name_or_id = Some(name);

                true
            }
            Ok(false) => {
                // should not be possible
                panic!("Unexpected API response");
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
        create_missing_project: bool,
    ) -> Result<()> {
        trace!("project arguments: {project_args:?}");

        self.ctx.load_local(project_args)?;

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
                    .get_projects_list()
                    .await?
                    .projects
                    .into_iter()
                    .find(|p| p.name == *name)
                {
                    trace!("found project by name");
                    self.ctx.set_project_id(proj.id);
                } else {
                    trace!("did not find project by name");
                    if create_missing_project {
                        trace!("creating project since it was not found");
                        let proj = client.create_project(name).await?;
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

        Ok(())
    }

    async fn project_link(&mut self, id_or_name: Option<String>) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let projs = client.get_projects_list().await?.projects;

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

                    let proj = client.create_project(&name).await?;
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
        let user = client.get_current_user().await?;
        print!("{}", user.to_string_colored());

        Ok(())
    }

    /// Log in with the given API key or after prompting the user for one.
    async fn login(&mut self, login_args: LoginArgs, offline: bool) -> Result<()> {
        let api_key = match login_args.api_key {
            Some(api_key) => api_key,
            None => {
                if login_args.prompt {
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

            if offline {
                eprintln!("INFO: Skipping API key verification");
            } else {
                let u = client
                    .get_current_user()
                    .await
                    .context("failed to check API key validity")?;
                println!("Logged in as {} ({})", u.name.bold(), u.id.bold());
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
                if let Err(e) = tx.send(Message::Ping(Default::default())).await {
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
            self.reset_api_key().await?;
            println!("Successfully reset the API key.");
        }
        self.ctx.clear_api_key()?;
        println!("Successfully logged out.");
        println!(" -> Use `shuttle login` to log in again.");

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

    async fn stop(&self, tracking_args: DeploymentTrackingArgs) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let pid = self.ctx.project_id();
        let res = client.stop_service(pid).await?;
        println!("{res}");

        if tracking_args.no_follow {
            return Ok(());
        }

        wait_with_spinner(2000, |_, pb| async move {
            let deployment = client.get_current_deployment(pid).await?;

            let get_cleanup = |d: Option<DeploymentResponse>| {
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
                    DeploymentState::Pending
                    | DeploymentState::Stopping
                    | DeploymentState::InProgress
                    | DeploymentState::Running => Ok(None),
                    DeploymentState::Building // a building deployment should take it back to InProgress then Running, so don't follow that sequence
                    | DeploymentState::Failed
                    | DeploymentState::Stopped
                    | DeploymentState::Unknown => Ok(Some(cleanup)),
                }
        })
        .await?;

        Ok(())
    }

    async fn logs(&self, args: LogsArgs) -> Result<()> {
        if args.follow {
            eprintln!("Streamed logs are not yet supported on the shuttle.dev platform.");
            return Ok(());
        }
        if args.tail.is_some() | args.head.is_some() {
            eprintln!("Fetching log ranges are not yet supported on the shuttle.dev platform.");
            return Ok(());
        }
        let client = self.client.as_ref().unwrap();
        let pid = self.ctx.project_id();
        let logs = if args.all_deployments {
            client.get_project_logs(pid).await?.logs
        } else {
            let id = if args.latest {
                // Find latest deployment (not always an active one)
                let deployments = client.get_deployments(pid, 1, 1).await?.deployments;
                let Some(most_recent) = deployments.into_iter().next() else {
                    println!("No deployments found");
                    return Ok(());
                };
                eprintln!("Getting logs from: {}", most_recent.id);
                most_recent.id
            } else if let Some(id) = args.id {
                id
            } else {
                let Some(current) = client.get_current_deployment(pid).await? else {
                    println!("No deployments found");
                    return Ok(());
                };
                eprintln!("Getting logs from: {}", current.id);
                current.id
            };
            client.get_deployment_logs(pid, &id).await?.logs
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

    async fn deployments_list(&self, page: u32, limit: u32, table_args: TableArgs) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        if limit == 0 {
            println!();
            return Ok(());
        }
        let limit = limit + 1;

        let proj_name = self.ctx.project_name();

        let mut deployments = client
            .get_deployments(self.ctx.project_id(), page as i32, limit as i32)
            .await?
            .deployments;
        let page_hint = if deployments.len() == limit as usize {
            deployments.pop();
            true
        } else {
            false
        };
        let table = deployments_table(&deployments, table_args.raw);

        println!(
            "{}",
            format!("Deployments in project '{}'", proj_name).bold()
        );
        println!("{table}");
        if page_hint {
            println!("View the next page using `--page {}`", page + 1);
        }

        Ok(())
    }

    async fn deployment_get(&self, deployment_id: Option<String>) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let pid = self.ctx.project_id();

        let deployment = match deployment_id {
            Some(id) => client.get_deployment(pid, &id).await,
            None => {
                let d = client.get_current_deployment(pid).await?;
                let Some(d) = d else {
                    println!("No deployment found");
                    return Ok(());
                };
                Ok(d)
            }
        }?;

        println!("{}", deployment.to_string_colored());

        Ok(())
    }

    async fn deployment_redeploy(
        &self,
        deployment_id: Option<String>,
        tracking_args: DeploymentTrackingArgs,
    ) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        let pid = self.ctx.project_id();
        let deployment_id = match deployment_id {
            Some(id) => id,
            None => {
                let d = client.get_current_deployment(pid).await?;
                let Some(d) = d else {
                    println!("No deployment found");
                    return Ok(());
                };
                d.id
            }
        };
        let deployment = client.redeploy(pid, &deployment_id).await?;

        if tracking_args.no_follow {
            println!("{}", deployment.to_string_colored());
            return Ok(());
        }
        self.track_deployment_status_and_print_logs_on_fail(pid, &deployment.id, tracking_args.raw)
            .await?;

        Ok(())
    }

    async fn resources_list(&self, table_args: TableArgs, show_secrets: bool) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let pid = self.ctx.project_id();
        let resources = client.get_service_resources(pid).await?.resources;
        let table = get_resource_tables(resources.as_slice(), pid, table_args.raw, show_secrets);

        println!("{table}");

        Ok(())
    }

    async fn resource_delete(&self, resource_type: &ResourceType, no_confirm: bool) -> Result<()> {
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

        let msg = client
            .delete_service_resource(self.ctx.project_id(), resource_type)
            .await?;
        println!("{msg}");

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

    async fn resource_dump(&self, _resource_type: &ResourceType) -> Result<()> {
        unimplemented!();
        // let client = self.client.as_ref().unwrap();
        // let bytes = client...;
        // std::io::stdout().write_all(&bytes).unwrap();
        // Ok(())
    }

    async fn list_certificates(&self, table_args: TableArgs) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let certs = client
            .list_certificates(self.ctx.project_id())
            .await?
            .certificates;

        let table = get_certificates_table(certs.as_ref(), table_args.raw);
        println!("{}", table);

        Ok(())
    }
    async fn add_certificate(&self, domain: String) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let cert = client
            .add_certificate(self.ctx.project_id(), domain.clone())
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
            .delete_certificate(self.ctx.project_id(), domain.clone())
            .await?;
        println!("{msg}");

        Ok(())
    }

    fn get_secrets(
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

    fn find_available_port(run_args: &mut RunArgs) {
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

    async fn local_run(&self, mut run_args: RunArgs, debug: bool) -> Result<()> {
        let project_name = self.ctx.project_name().to_owned();
        let working_directory = self.ctx.working_directory();
        let services = self.pre_local_run(&run_args).await?;
        let service = services
            .first()
            .expect("at least one shuttle service")
            .to_owned();

        trace!(path = ?service.executable_path, "runtime executable");

        let secrets =
            Shuttle::get_secrets(&run_args.secret_args, working_directory)?.unwrap_or_default();
        Shuttle::find_available_port(&mut run_args);
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
        tokio::spawn(async move { ProvisionerServer::run(state, &api_addr).await });

        println!(
            "\n    {} {} on http://{}:{}\n",
            "Starting".bold().green(),
            service.package_name,
            ip,
            run_args.port,
        );

        let mut envs = vec![
            ("SHUTTLE_BETA", "true".to_owned()),
            ("SHUTTLE_PROJECT_ID", "proj_LOCAL".to_owned()),
            ("SHUTTLE_PROJECT_NAME", project_name),
            ("SHUTTLE_ENV", Environment::Local.to_string()),
            ("SHUTTLE_RUNTIME_IP", ip.to_string()),
            ("SHUTTLE_RUNTIME_PORT", run_args.port.to_string()),
            ("SHUTTLE_API", format!("http://127.0.0.1:{}", api_port)),
        ];
        // Use a nice debugging tracing level if user does not provide their own
        if debug && std::env::var("RUST_LOG").is_err() {
            envs.push(("RUST_LOG", "info,shuttle=trace,reqwest=debug".to_owned()));
        }

        info!(
            path = %runtime_executable.display(),
            "Spawning runtime process",
        );
        let mut runtime = tokio::process::Command::new(
            dunce::canonicalize(runtime_executable).context("canonicalize path of executable")?,
        )
        .current_dir(&service.workspace_path)
        .envs(envs)
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
                    let log_item = LogItem::new(Utc::now(), "app".to_owned(), line);
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
                    let log_item = LogItem::new(Utc::now(), "app".to_owned(), line);
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
                    eprintln!("Received SIGTERM. Killing the runtime...");
                    None
                },
                _ = sigint_notif.recv() => {
                    eprintln!("Received SIGINT. Killing the runtime...");
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
                    eprintln!("Received ctrl-break.");
                    None
                },
                _ = ctrl_c_notif.recv() => {
                    eprintln!("Received ctrl-c.");
                    None
                },
                _ = ctrl_close_notif.recv() => {
                    eprintln!("Received ctrl-close.");
                    None
                },
                _ = ctrl_logoff_notif.recv() => {
                    eprintln!("Received ctrl-logoff.");
                    None
                },
                _ = ctrl_shutdown_notif.recv() => {
                    eprintln!("Received ctrl-shutdown.");
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

    async fn deploy(&mut self, args: DeployArgs) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let working_directory = self.ctx.working_directory();
        let manifest_path = working_directory.join("Cargo.toml");

        let secrets = Shuttle::get_secrets(&args.secret_args, working_directory)?;

        // Image deployment mode
        if let Some(image) = args.image {
            let pid = self.ctx.project_id();
            let deployment_req_image = DeploymentRequestImage { image, secrets };

            let deployment = client
                .deploy(pid, DeploymentRequest::Image(deployment_req_image))
                .await?;

            if args.tracking_args.no_follow {
                println!("{}", deployment.to_string_colored());
                return Ok(());
            }

            self.track_deployment_status_and_print_logs_on_fail(
                pid,
                &deployment.id,
                args.tracking_args.raw,
            )
            .await?;

            return Ok(());
        }

        // Build archive deployment mode
        let mut deployment_req = DeploymentRequestBuildArchive {
            secrets,
            ..Default::default()
        };
        let mut build_meta = BuildMeta::default();
        let mut rust_build_args = BuildArgsRust::default();

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

        deployment_req.build_args = Some(BuildArgs::Rust(rust_build_args));

        // TODO: have all of the above be configurable in CLI and Shuttle.toml

        if let Ok(repo) = Repository::discover(working_directory) {
            let repo_path = repo
                .workdir()
                .context("getting working directory of repository")?;
            let repo_path = dunce::canonicalize(repo_path)?;
            trace!(?repo_path, "found git repository");

            let dirty = is_dirty(&repo);
            build_meta.git_dirty = Some(dirty.is_err());

            let check_dirty = self.ctx.deny_dirty().is_some_and(|d| d);
            if check_dirty && !args.allow_dirty && dirty.is_err() {
                bail!(dirty.unwrap_err());
            }

            if let Ok(head) = repo.head() {
                // This is typically the name of the current branch
                // It is "HEAD" when head detached, for example when a tag is checked out
                build_meta.git_branch = head
                    .shorthand()
                    .map(|s| s.chars().take(GIT_STRINGS_MAX_LENGTH).collect());
                if let Ok(commit) = head.peel_to_commit() {
                    build_meta.git_commit_id = Some(commit.id().to_string());
                    // Summary is None if error or invalid utf-8
                    build_meta.git_commit_msg = commit
                        .summary()
                        .map(|s| s.chars().take(GIT_STRINGS_MAX_LENGTH).collect());
                }
            }
        }

        eprintln!("Packing files...");
        let archive = self.make_archive(args.secret_args.secrets.clone())?;

        if let Some(path) = args.output_archive {
            eprintln!("Writing archive to {}", path.display());
            std::fs::write(path, archive).context("writing archive")?;

            return Ok(());
        }

        // TODO: upload secrets separately

        let pid = self.ctx.project_id();

        eprintln!("Uploading code...");
        let arch = client.upload_archive(pid, archive).await?;
        deployment_req.archive_version_id = arch.archive_version_id;
        deployment_req.build_meta = Some(build_meta);

        eprintln!("Creating deployment...");
        let deployment = client
            .deploy(pid, DeploymentRequest::BuildArchive(deployment_req))
            .await?;

        if args.tracking_args.no_follow {
            println!("{}", deployment.to_string_colored());
            return Ok(());
        }

        self.track_deployment_status_and_print_logs_on_fail(
            pid,
            &deployment.id,
            args.tracking_args.raw,
        )
        .await?;

        Ok(())
    }

    /// Returns true if the deployment failed
    async fn track_deployment_status(&self, pid: &str, id: &str) -> Result<bool> {
        let client = self.client.as_ref().unwrap();
        let failed = wait_with_spinner(2000, |_, pb| async move {
            let deployment = client.get_deployment(pid, id).await?;

            let state = deployment.state.clone();
            pb.set_message(deployment.to_string_summary_colored());
            let failed = state == DeploymentState::Failed;
            let cleanup = move || {
                println!("{}", deployment.to_string_colored());
                failed
            };
            match state {
                DeploymentState::Pending
                | DeploymentState::Building
                | DeploymentState::InProgress => Ok(None),
                DeploymentState::Running
                | DeploymentState::Stopped
                | DeploymentState::Stopping
                | DeploymentState::Unknown
                | DeploymentState::Failed => Ok(Some(cleanup)),
            }
        })
        .await?;

        Ok(failed)
    }

    async fn track_deployment_status_and_print_logs_on_fail(
        &self,
        proj_id: &str,
        depl_id: &str,
        raw: bool,
    ) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        if self.track_deployment_status(proj_id, depl_id).await? {
            for log in client.get_deployment_logs(proj_id, depl_id).await?.logs {
                if raw {
                    println!("{}", log.line);
                } else {
                    println!("{log}");
                }
            }
        }

        Ok(())
    }

    async fn project_create(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let name = self.ctx.project_name();
        let project = client.create_project(name).await?;

        println!("Created project '{}' with id {}", project.name, project.id);

        Ok(())
    }
    async fn project_rename(&self, name: String) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        let project = client
            .update_project(
                self.ctx.project_id(),
                ProjectUpdateRequest {
                    name: Some(name),
                    ..Default::default()
                },
            )
            .await?;

        println!("Renamed project {} to {}", project.id, project.name);

        Ok(())
    }

    async fn projects_list(&self, table_args: TableArgs) -> Result<()> {
        let client = self.client.as_ref().unwrap();

        let projects_table =
            get_projects_table(&client.get_projects_list().await?.projects, table_args.raw);

        println!("{}", "Personal Projects".bold());
        println!("{projects_table}\n");

        Ok(())
    }

    async fn project_status(&self) -> Result<()> {
        let client = self.client.as_ref().unwrap();
        let project = client.get_project(self.ctx.project_id()).await?;
        print!("{}", project.to_string_colored());

        Ok(())
    }

    async fn project_delete(&self, no_confirm: bool) -> Result<()> {
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

        let res = client.delete_project(pid).await?;

        println!("{res}");

        Ok(())
    }

    fn make_archive(&self, secrets_file: Option<PathBuf>) -> Result<Vec<u8>> {
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
            if path.is_dir() {
                trace!("Skipping {:?}: is a directory", path);
                continue;
            }
            // symlinks == chaos
            if path.is_symlink() {
                trace!("Skipping {:?}: is a symlink", path);
                continue;
            }

            // zip file puts all files in root
            let mut name = path
                .strip_prefix(working_directory)
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

        let bytes = {
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
    ) -> Vec<String> {
        let mut shuttle = Shuttle::new(crate::Binary::Shuttle).unwrap();
        shuttle
            .load_project(&project_args, false, false)
            .await
            .unwrap();

        let archive = shuttle
            .make_archive(deploy_args.secret_args.secrets)
            .unwrap();

        let mut zip = ZipArchive::new(Cursor::new(archive)).unwrap();
        (0..zip.len())
            .map(|i| zip.by_index(i).unwrap().name().to_owned())
            .collect()
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
            name_or_id: Some("proj_archiving-test".to_owned()),
        };
        let mut entries = get_archive_entries(project_args.clone(), Default::default()).await;
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

        fs::remove_file(working_directory.join("Secrets.toml")).unwrap();
        let mut entries = get_archive_entries(
            project_args,
            DeployArgs {
                secret_args: SecretsArgs {
                    secrets: Some(working_directory.join("Secrets.toml.example")),
                },
                ..Default::default()
            },
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

        assert_eq!(
            project_args.workspace_path().unwrap(),
            path_from_workspace_root("examples/axum/hello-world")
        );
    }
}
