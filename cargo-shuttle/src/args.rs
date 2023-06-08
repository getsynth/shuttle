use std::{
    ffi::OsString,
    fs::create_dir_all,
    io::{self, ErrorKind},
    path::PathBuf,
};

use anyhow::Context;
use cargo_metadata::MetadataCommand;
use clap::{
    builder::{OsStringValueParser, PossibleValue, TypedValueParser},
    Parser, ValueEnum,
};
use clap_complete::Shell;
use shuttle_common::{models::project::IDLE_MINUTES, project::ProjectName};
use uuid::Uuid;

#[derive(Parser)]
#[command(
    version,
    about,
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
    #[command(subcommand)]
    pub cmd: Command,
}

// Common args for subcommands that deal with projects.
#[derive(Parser, Debug)]
pub struct ProjectArgs {
    /// Specify the working directory
    #[arg(global = true, long, default_value = ".", value_parser = OsStringValueParser::new().try_map(parse_init_path))]
    pub working_directory: PathBuf,
    /// Specify the name of the project (overrides crate name)
    #[arg(global = true, long)]
    pub name: Option<ProjectName>,
}

impl ProjectArgs {
    pub fn workspace_path(&self) -> anyhow::Result<PathBuf> {
        let path = MetadataCommand::new()
            .current_dir(&self.working_directory)
            .exec()
            .context("failed to get cargo metadata")?
            .workspace_root
            .into();

        Ok(path)
    }

    pub fn project_name(&self) -> anyhow::Result<ProjectName> {
        let workspace_path = self.workspace_path()?;

        let meta = MetadataCommand::new()
            .current_dir(&workspace_path)
            .exec()
            .unwrap();
        let package_name = if let Some(root_package) = meta.root_package() {
            root_package.name.clone().parse()?
        } else {
            workspace_path
                .file_name()
                .context("failed to get project name from workspace path")?
                .to_os_string()
                .into_string()
                .expect("workspace file name should be valid unicode")
                .parse()?
        };

        Ok(package_name)
    }
}

#[derive(Parser)]
pub enum Command {
    /// Create a new shuttle project
    Init(InitArgs),
    /// Run a shuttle service locally
    Run(RunArgs),
    /// Deploy a shuttle service
    Deploy(DeployArgs),
    /// Manage deployments of a shuttle service
    #[command(subcommand)]
    Deployment(DeploymentCommand),
    /// View the status of a shuttle service
    Status,
    /// Stop this shuttle service
    Stop,
    /// View the logs of a deployment in this shuttle service
    Logs {
        /// Deployment ID to get logs for. Defaults to currently running deployment
        id: Option<Uuid>,
        #[arg(short, long)]
        /// View logs from the most recent deployment (which is not always the latest running one)
        latest: bool,
        #[arg(short, long)]
        /// Follow log output
        follow: bool,
    },
    /// List or manage projects on shuttle
    #[command(subcommand)]
    Project(ProjectCommand),
    /// Manage resources of a shuttle project
    #[command(subcommand)]
    Resource(ResourceCommand),
    /// Manage secrets for this shuttle service
    Secrets,
    /// Remove cargo build artifacts in the shuttle environment
    Clean,
    /// Login to the shuttle platform
    Login(LoginArgs),
    /// Log out of the shuttle platform
    Logout(LogoutArgs),
    /// Generate shell completions
    Generate {
        /// Which shell
        #[arg(short, long, env, default_value_t = Shell::Bash)]
        shell: Shell,
        /// Output to a file (stdout by default)
        #[arg(short, long, env)]
        output: Option<PathBuf>,
    },
    /// Open an issue on GitHub and provide feedback
    Feedback,
}

#[derive(Parser)]
pub enum DeploymentCommand {
    /// List all the deployments for a service
    List {
        #[arg(long, default_value = "1")]
        /// Which page to display
        page: u32,

        #[arg(long, default_value = "10")]
        /// How many projects per page to display
        limit: u32,
    },
    /// View status of a deployment
    Status {
        /// ID of deployment to get status for
        id: Uuid,
    },
}

#[derive(Parser)]
pub enum ResourceCommand {
    /// List all the resources for a project
    List,
}

#[derive(Parser)]
pub enum ProjectCommand {
    /// Create an environment for this project on shuttle
    Start(ProjectStartArgs),
    /// Check the status of this project's environment on shuttle
    Status {
        #[arg(short, long)]
        /// Follow status of project command
        follow: bool,
    },
    /// Destroy this project's environment (container) on shuttle
    Stop,
    /// Destroy and create an environment for this project on shuttle
    Restart(ProjectStartArgs),
    /// List all projects belonging to the calling account
    List {
        #[arg(long, default_value = "1")]
        /// Which page to display
        page: u32,

        #[arg(long, default_value = "10")]
        /// How many projects per page to display
        limit: u32,
    },
}

#[derive(Parser, Debug)]
pub struct ProjectStartArgs {
    #[arg(long, default_value_t = IDLE_MINUTES)]
    /// How long to wait before putting the project in an idle state due to inactivity.
    /// 0 means the project will never idle
    pub idle_minutes: u64,
}

#[derive(Parser, Clone, Debug)]
pub struct LoginArgs {
    /// API key for the shuttle platform
    #[arg(long)]
    pub api_key: Option<String>,
}

#[derive(Parser, Clone, Debug)]
pub struct LogoutArgs {
    /// Reset the API key before logging out
    #[arg(long)]
    pub reset_api_key: bool,
}
#[derive(Parser)]
pub struct DeployArgs {
    /// Allow deployment with uncommited files
    #[arg(long)]
    pub allow_dirty: bool,
    /// Don't run pre-deploy tests
    #[arg(long)]
    pub no_test: bool,
}

#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Port to start service on
    #[arg(long, env, default_value = "8000")]
    pub port: u16,
    /// Use 0.0.0.0 instead of localhost (for usage with local external devices)
    #[arg(long)]
    pub external: bool,
    /// Use release mode for building the project
    #[arg(long, short = 'r')]
    pub release: bool,
}

#[derive(Parser, Clone, Debug)]
pub struct InitArgs {
    /// Initialize the project with a starter template
    #[arg(long, short, value_enum, conflicts_with_all = &["git", "git_path"])]
    pub template: Option<InitTemplateArg>,
    /// Initialize the project from a git repository
    #[arg(long, short)]
    pub git: Option<String>,
    /// Path to the folder in the repository (used with --git)
    #[arg(long, requires = "git")]
    pub git_path: Option<String>,

    /// Path to initialize a new shuttle project
    #[arg(default_value = ".", value_parser = OsStringValueParser::new().try_map(parse_init_path))]
    pub path: PathBuf,

    /// Whether to create the environment for this project on shuttle
    #[arg(long)]
    pub create_env: bool,
    #[command(flatten)]
    pub login_args: LoginArgs,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, strum::Display, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum InitTemplateArg {
    /// Actix-web framework
    ActixWeb,
    /// Axum web framework
    Axum,
    /// Poem web framework
    Poem,
    /// Poise Discord framework
    Poise,
    /// Rocket web framework
    Rocket,
    /// Salvo web framework
    Salvo,
    /// Serenity Discord framework
    Serenity,
    /// Tide web framework
    Tide,
    /// Thruster web framework
    Thruster,
    /// Tower web framework
    Tower,
    /// Warp web framework
    Warp,
    /// No template - Custom empty service
    None,
}

/// git repo and path for chosen template or given URL+path
pub type GitTemplate = Option<(String, Option<String>)>;

const EXAMPLES_REPO: &str = "http://github.com/shuttle-hq/shuttle-examples.git";

impl From<&InitArgs> for GitTemplate {
    fn from(value: &InitArgs) -> Self {
        if let Some(git) = value.git.clone() {
            return Some((git, value.git_path.clone()));
        }

        value.template.and_then(|t| (&t).into())
    }
}

impl From<&InitTemplateArg> for GitTemplate {
    fn from(value: &InitTemplateArg) -> Self {
        use InitTemplateArg::*;
        Some(match value {
            ActixWeb => (EXAMPLES_REPO.into(), Some("actix-web/hello-world".into())),
            Axum => (EXAMPLES_REPO.into(), Some("axum/hello-world".into())),
            Poem => (EXAMPLES_REPO.into(), Some("poem/hello-world".into())),
            Poise => (EXAMPLES_REPO.into(), Some("poise/hello-world".into())),
            Rocket => (EXAMPLES_REPO.into(), Some("rocket/hello-world".into())),
            Salvo => (EXAMPLES_REPO.into(), Some("salvo/hello-world".into())),
            Serenity => (EXAMPLES_REPO.into(), Some("serenity/hello-world".into())),
            Tide => (EXAMPLES_REPO.into(), Some("tide/hello-world".into())),
            Thruster => (EXAMPLES_REPO.into(), Some("thruster/hello-world".into())),
            Tower => (EXAMPLES_REPO.into(), Some("tower/hello-world".into())),
            Warp => (EXAMPLES_REPO.into(), Some("warp/hello-world".into())),
            None => (EXAMPLES_REPO.into(), Some("custon/none".into())),
        })
    }
}

/// Helper function to parse and return the absolute path
fn parse_path(path: OsString) -> Result<PathBuf, String> {
    dunce::canonicalize(&path).map_err(|e| format!("could not turn {path:?} into a real path: {e}"))
}

/// Helper function to parse, create if not exists, and return the absolute path
pub(crate) fn parse_init_path(path: OsString) -> Result<PathBuf, io::Error> {
    // Create the directory if does not exist
    create_dir_all(&path)?;

    parse_path(path.clone()).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidInput,
            format!("could not turn {path:?} into a real path: {e}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::tests::path_from_workspace_root;

    use super::*;

    #[test]
    fn test_init_args_framework() {
        todo!();
        // let init_args = InitArgs {
        //     template: Some(InitTemplateArg::Axum),
        //     create_env: false,
        //     login_args: LoginArgs { api_key: None },
        //     path: PathBuf::new(),
        // };
        // assert_eq!(init_args.framework(), Some(Template::Axum));
        // let init_args = InitArgs {
        //     template: Some(InitTemplateArg::None),
        //     create_env: false,
        //     login_args: LoginArgs { api_key: None },
        //     path: PathBuf::new(),
        // };
        // assert_eq!(init_args.framework(), Some(Template::None));
        // let init_args = InitArgs {
        //     template: None,
        //     create_env: false,
        //     login_args: LoginArgs { api_key: None },
        //     path: PathBuf::new(),
        // };
        // assert_eq!(init_args.framework(), None);
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
