use std::{
    ffi::OsString,
    fs::create_dir_all,
    io::{self, ErrorKind},
    path::PathBuf,
};

use anyhow::{bail, Context};
use cargo_metadata::MetadataCommand;
use clap::{
    builder::{OsStringValueParser, PossibleValue, TypedValueParser},
    Parser, ValueEnum,
};
use clap_complete::Shell;
use shuttle_common::constants::{DEFAULT_IDLE_MINUTES, EXAMPLES_REPO};
use shuttle_common::resource;

#[derive(Parser)]
#[command(
    version,
    // Cargo passes in the subcommand name to the invoked executable. Use a
    // hidden, optional positional argument to deal with it.
    arg(clap::Arg::new("dummy")
        .value_parser([PossibleValue::new("shuttle")])
        .required(false)
        .hide(true))
)]
pub struct ShuttleArgs {
    #[command(flatten)]
    pub project_args: ProjectArgs,
    /// Run this command against the API at the supplied URL
    /// (allows targeting a custom deployed instance for this command only, mainly for development)
    #[arg(long, env = "SHUTTLE_API")]
    pub api_url: Option<String>,
    /// Disable network requests that are not strictly necessary. Limits some features.
    #[arg(long, env = "SHUTTLE_OFFLINE")]
    pub offline: bool,
    /// Turn on tracing output for Shuttle libraries. (WARNING: can print sensitive data)
    #[arg(long, env = "SHUTTLE_DEBUG")]
    pub debug: bool,
    /// Target Shuttle's development environment
    #[arg(long, env = "SHUTTLE_BETA", hide = true)]
    pub beta: bool,

    #[command(subcommand)]
    pub cmd: Command,
}

// Common args for subcommands that deal with projects.
#[derive(Parser, Clone, Debug)]
pub struct ProjectArgs {
    /// Specify the working directory
    #[arg(global = true, long, visible_alias = "wd", default_value = ".", value_parser = OsStringValueParser::new().try_map(parse_path))]
    pub working_directory: PathBuf,
    /// Specify the name of the project (overrides crate name)
    #[arg(global = true, long)]
    pub name: Option<String>,
}

impl ProjectArgs {
    pub fn workspace_path(&self) -> anyhow::Result<PathBuf> {
        // NOTE: If crates cache is missing this blocks for several seconds during download
        let path = MetadataCommand::new()
            .current_dir(&self.working_directory)
            .exec()
            .context("failed to get cargo metadata")?
            .workspace_root
            .into();

        Ok(path)
    }

    pub fn project_name(&self) -> anyhow::Result<String> {
        let workspace_path = self.workspace_path()?;

        // NOTE: If crates cache is missing this blocks for several seconds during download
        let meta = MetadataCommand::new()
            .current_dir(&workspace_path)
            .exec()
            .context("failed to get cargo metadata")?;
        let package_name = if let Some(root_package) = meta.root_package() {
            root_package.name.clone()
        } else {
            workspace_path
                .file_name()
                .context("failed to get project name from workspace path")?
                .to_os_string()
                .into_string()
                .expect("workspace directory name should be valid unicode")
        };

        Ok(package_name)
    }
}

/// A cargo command for the Shuttle platform (https://www.shuttle.rs/)
///
/// See the CLI docs (https://docs.shuttle.rs/getting-started/shuttle-commands)
/// for more information.
#[derive(Parser)]
pub enum Command {
    /// Create a new Shuttle project
    Init(InitArgs),
    /// Run a Shuttle service locally
    Run(RunArgs),
    /// Deploy a Shuttle service
    Deploy(DeployArgs),
    /// Manage deployments of a Shuttle service
    #[command(subcommand)]
    Deployment(DeploymentCommand),
    /// View the status of a Shuttle service
    Status,
    /// Stop this Shuttle service
    Stop,
    /// View the logs of a deployment in this Shuttle service
    Logs(LogsArgs),
    /// List or manage projects on Shuttle
    #[command(subcommand)]
    Project(ProjectCommand),
    /// Manage resources of a Shuttle project
    #[command(subcommand)]
    Resource(ResourceCommand),
    /// Remove cargo build artifacts in the Shuttle environment
    Clean,
    /// Login to the Shuttle platform
    Login(LoginArgs),
    /// Log out of the Shuttle platform
    Logout(LogoutArgs),
    /// Generate shell completions and man page
    #[command(subcommand)]
    Generate(GenerateCommand),
    /// Open an issue on GitHub and provide feedback
    Feedback,
}

#[derive(Parser)]
pub enum GenerateCommand {
    /// Generate shell completions
    Shell {
        /// The shell to generate shell completion for
        shell: Shell,
        /// Output to a file (stdout by default)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate man page to the standard output
    Manpage,
}

#[derive(Parser)]
pub struct TableArgs {
    #[arg(long, default_value_t = false)]
    /// Output tables without borders
    pub raw: bool,
}

#[derive(Parser)]
pub enum DeploymentCommand {
    /// List all the deployments for a service
    List {
        #[arg(long, default_value = "1")]
        /// Which page to display
        page: u32,

        #[arg(long, default_value = "10", visible_alias = "per-page")]
        /// How many deployments per page to display
        limit: u32,

        #[command(flatten)]
        table: TableArgs,
    },
    /// View status of a deployment
    Status {
        /// ID of deployment to get status for
        id: Option<String>,
    },
    /// BETA: Stop running deployment(s)
    Stop,
}

#[derive(Parser)]
pub enum ResourceCommand {
    /// List all the resources for a project
    List {
        #[command(flatten)]
        table: TableArgs,

        #[arg(long, default_value_t = false)]
        /// Show secrets from resources (e.g. a password in a connection string)
        show_secrets: bool,
    },
    /// Delete a resource
    Delete {
        /// Type of the resource to delete.
        /// Use the string in the 'Type' column as displayed in the `resource list` command.
        /// For example, 'database::shared::postgres'.
        resource_type: resource::Type,
        #[command(flatten)]
        confirmation: ConfirmationArgs,
    },
}

#[derive(Parser)]
pub enum ProjectCommand {
    /// Create an environment for this project on Shuttle
    #[command(visible_alias = "create")]
    Start(ProjectStartArgs),
    /// Check the status of this project's environment on Shuttle
    Status {
        #[arg(short, long)]
        /// Follow status of project command
        follow: bool,
    },
    /// Destroy this project's environment (container) on Shuttle
    Stop,
    /// Destroy and create an environment for this project on Shuttle
    Restart(ProjectStartArgs),
    /// List all projects you have access to
    List {
        // deprecated args, kept around to not break
        #[arg(long, hide = true)]
        page: Option<u32>,
        #[arg(long, hide = true)]
        limit: Option<u32>,

        #[command(flatten)]
        table: TableArgs,
    },
    /// Delete a project and all linked data
    Delete(ConfirmationArgs),
}

#[derive(Parser, Debug)]
pub struct ConfirmationArgs {
    #[arg(long, short, default_value_t = false)]
    /// Skip confirmations and proceed
    pub yes: bool,
}

#[derive(Parser, Debug)]
pub struct ProjectStartArgs {
    #[arg(long, default_value_t = DEFAULT_IDLE_MINUTES)]
    /// How long to wait before putting the project in an idle state due to inactivity.
    /// 0 means the project will never idle
    pub idle_minutes: u64,
}

#[derive(Parser, Clone, Debug, Default)]
pub struct LoginArgs {
    /// API key for the Shuttle platform
    #[arg(long)]
    pub api_key: Option<String>,
}

#[derive(Parser, Clone, Debug)]
pub struct LogoutArgs {
    /// Reset the API key before logging out
    #[arg(long)]
    pub reset_api_key: bool,
}
#[derive(Parser, Default)]
pub struct DeployArgs {
    /// BETA: Deploy this Docker image instead of building one
    #[arg(long, short = 'i', hide = true)]
    pub image: Option<String>, // TODO?: Make this a subcommand instead? `cargo shuttle deploy image ...`
    /// BETA: Don't follow the deployment status, exit after the deployment begins
    #[arg(long, visible_alias = "nf", hide = true)]
    pub no_follow: bool,

    /// Allow deployment with uncommitted files
    #[arg(long, visible_alias = "ad")]
    pub allow_dirty: bool,
    /// Don't run pre-deploy tests
    #[arg(long, visible_alias = "nt")]
    pub no_test: bool,
    /// Don't display timestamps and log origin tags
    #[arg(long)]
    pub raw: bool,
    /// Output the deployment archive to a file instead of sending a deployment request
    #[arg(long)]
    pub output_archive: Option<PathBuf>,

    #[command(flatten)]
    pub secret_args: SecretsArgs,
}

#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Port to start service on
    #[arg(long, short = 'p', env, default_value = "8000")]
    pub port: u16,
    /// Use 0.0.0.0 instead of localhost (for usage with local external devices)
    #[arg(long)]
    pub external: bool,
    /// Use release mode for building the project
    #[arg(long, short = 'r')]
    pub release: bool,
    /// Don't display timestamps and log origin tags
    #[arg(long)]
    pub raw: bool,

    #[command(flatten)]
    pub secret_args: SecretsArgs,

    #[arg(long)]
    pub quiet: bool,
}

#[derive(Parser, Debug, Default)]
pub struct SecretsArgs {
    /// Use this secrets file instead
    #[arg(long, value_parser = OsStringValueParser::new().try_map(parse_path))]
    pub secrets: Option<PathBuf>,
}

#[derive(Parser, Clone, Debug, Default)]
pub struct InitArgs {
    /// Clone a starter template from Shuttle's official examples
    #[arg(long, short, value_enum, conflicts_with_all = &["from", "subfolder"])]
    pub template: Option<InitTemplateArg>,
    /// Clone a template from a git repository or local path using cargo-generate
    #[arg(long)]
    pub from: Option<String>,
    /// Path to the template in the source (used with --from)
    #[arg(long, requires = "from")]
    pub subfolder: Option<String>,

    /// Path where to place the new Shuttle project
    #[arg(default_value = ".", value_parser = OsStringValueParser::new().try_map(create_and_parse_path))]
    pub path: PathBuf,

    /// Don't check the project name's validity or availability and use it anyways
    #[arg(long)]
    pub force_name: bool,
    /// Whether to start the container for this project on Shuttle, and claim the project name
    #[arg(long)]
    pub create_env: bool,
    /// Don't initialize a new git repository
    #[arg(long)]
    pub no_git: bool,

    #[command(flatten)]
    pub login_args: LoginArgs,
}

#[derive(ValueEnum, Clone, Debug, strum::EnumMessage, strum::VariantArray)]
pub enum InitTemplateArg {
    /// Axum - Modular web framework from the Tokio ecosystem
    Axum,
    /// Actix Web - Powerful and fast web framework
    ActixWeb,
    /// Rocket - Simple and easy-to-use web framework
    Rocket,
    /// Loco - Batteries included web framework based on Axum
    Loco,
    /// Salvo - Powerful and simple web framework
    Salvo,
    /// Poem - Full-featured and easy-to-use web framework
    Poem,
    /// Poise - Discord Bot framework with good slash command support
    Poise,
    /// Serenity - Discord Bot framework
    Serenity,
    /// Tower - Modular service library
    Tower,
    /// Thruster - Web framework
    Thruster,
    /// Tide - Web framework
    Tide,
    /// Warp - Web framework
    Warp,
    /// No template - Make a custom service
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TemplateLocation {
    pub auto_path: String,
    pub subfolder: Option<String>,
}

impl InitArgs {
    pub fn git_template(&self) -> anyhow::Result<Option<TemplateLocation>> {
        if self.from.is_some() && self.template.is_some() {
            bail!("Template and From args can not be set at the same time.");
        }
        Ok(if let Some(from) = self.from.clone() {
            Some(TemplateLocation {
                auto_path: from,
                subfolder: self.subfolder.clone(),
            })
        } else {
            self.template.as_ref().map(|t| t.template())
        })
    }
}

impl InitTemplateArg {
    pub fn template(&self) -> TemplateLocation {
        use InitTemplateArg::*;
        let path = match self {
            ActixWeb => "actix-web/hello-world",
            Axum => "axum/hello-world",
            Loco => "loco/hello-world",
            Poem => "poem/hello-world",
            Poise => "poise/hello-world",
            Rocket => "rocket/hello-world",
            Salvo => "salvo/hello-world",
            Serenity => "serenity/hello-world",
            Thruster => "thruster/hello-world",
            Tide => "tide/hello-world",
            Tower => "tower/hello-world",
            Warp => "warp/hello-world",
            None => "custom-service/none",
        };

        TemplateLocation {
            auto_path: EXAMPLES_REPO.into(),
            subfolder: Some(path.to_string()),
        }
    }
}

#[derive(Parser, Clone, Debug, Default)]
pub struct LogsArgs {
    /// Deployment ID to get logs for. Defaults to the current deployment
    pub id: Option<String>,
    #[arg(short, long)]
    /// View logs from the most recent deployment (which is not always the latest running one)
    pub latest: bool,
    #[arg(short, long)]
    /// Follow log output
    pub follow: bool,
    /// Don't display timestamps and log origin tags
    #[arg(long)]
    pub raw: bool,
    /// View the first N log lines
    #[arg(long, group = "output_mode")]
    pub head: Option<u32>,
    /// View the last N log lines
    #[arg(long, group = "output_mode")]
    pub tail: Option<u32>,
    /// View all log lines
    #[arg(long, group = "output_mode")]
    pub all: bool,
    /// Get logs from all deployments instead of one deployment
    #[arg(long)]
    pub all_deployments: bool,
}

/// Helper function to parse and return the absolute path
fn parse_path(path: OsString) -> Result<PathBuf, io::Error> {
    dunce::canonicalize(&path).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidInput,
            format!("could not turn {path:?} into a real path: {e}"),
        )
    })
}

/// Helper function to parse, create if not exists, and return the absolute path
pub(crate) fn create_and_parse_path(path: OsString) -> Result<PathBuf, io::Error> {
    // Create the directory if does not exist
    create_dir_all(&path).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidInput,
            format!("Could not create directory: {e}"),
        )
    })?;

    parse_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::path_from_workspace_root;
    use clap::CommandFactory;

    #[test]
    fn test_shuttle_args() {
        ShuttleArgs::command().debug_assert();
    }

    #[test]
    fn test_init_args_framework() {
        // pre-defined template (only hello world)
        let init_args = InitArgs {
            template: Some(InitTemplateArg::Tower),
            from: None,
            subfolder: None,
            ..Default::default()
        };
        assert_eq!(
            init_args.git_template().unwrap(),
            Some(TemplateLocation {
                auto_path: EXAMPLES_REPO.into(),
                subfolder: Some("tower/hello-world".into())
            })
        );

        // pre-defined template (multiple)
        let init_args = InitArgs {
            template: Some(InitTemplateArg::Axum),
            from: None,
            subfolder: None,
            ..Default::default()
        };
        assert_eq!(
            init_args.git_template().unwrap(),
            Some(TemplateLocation {
                auto_path: EXAMPLES_REPO.into(),
                subfolder: Some("axum/hello-world".into())
            })
        );

        // pre-defined "none" template
        let init_args = InitArgs {
            template: Some(InitTemplateArg::None),
            from: None,
            subfolder: None,
            ..Default::default()
        };
        assert_eq!(
            init_args.git_template().unwrap(),
            Some(TemplateLocation {
                auto_path: EXAMPLES_REPO.into(),
                subfolder: Some("custom-service/none".into())
            })
        );

        // git template with path
        let init_args = InitArgs {
            template: None,
            from: Some("https://github.com/some/repo".into()),
            subfolder: Some("some/path".into()),
            ..Default::default()
        };
        assert_eq!(
            init_args.git_template().unwrap(),
            Some(TemplateLocation {
                auto_path: "https://github.com/some/repo".into(),
                subfolder: Some("some/path".into())
            })
        );

        // No template or repo chosen
        let init_args = InitArgs {
            template: None,
            from: None,
            subfolder: None,
            ..Default::default()
        };
        assert_eq!(init_args.git_template().unwrap(), None);
    }

    #[test]
    fn workspace_path() {
        let project_args = ProjectArgs {
            working_directory: path_from_workspace_root("examples/axum/hello-world/src"),
            name: None,
        };

        assert_eq!(
            project_args.workspace_path().unwrap(),
            path_from_workspace_root("examples/axum/hello-world/")
        );
    }

    #[test]
    fn project_name() {
        let project_args = ProjectArgs {
            working_directory: path_from_workspace_root("examples/axum/hello-world/src"),
            name: None,
        };

        assert_eq!(
            project_args.project_name().unwrap().to_string(),
            "hello-world"
        );
    }

    #[test]
    fn project_name_in_workspace() {
        let project_args = ProjectArgs {
            working_directory: path_from_workspace_root(
                "examples/rocket/workspace/hello-world/src",
            ),
            name: None,
        };

        assert_eq!(
            project_args.project_name().unwrap().to_string(),
            "workspace"
        );
    }
}
