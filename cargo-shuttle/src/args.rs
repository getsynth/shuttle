use std::{
    ffi::OsStr,
    fs::{canonicalize, create_dir_all},
    io::{self, ErrorKind},
    path::PathBuf,
};

use clap::Parser;
use clap_complete::Shell;
use shuttle_common::project::ProjectName;
use uuid::Uuid;

use crate::init::Framework;

#[derive(Parser)]
#[clap(
    version,
    about,
    // Cargo passes in the subcommand name to the invoked executable. Use a
    // hidden, optional positional argument to deal with it.
    arg(clap::Arg::with_name("dummy")
        .possible_value("shuttle")
        .required(false)
        .hidden(true))
)]
pub struct Args {
    /// run this command against the api at the supplied url
    /// (allows targeting a custom deployed instance for this command only)
    #[clap(long, env = "SHUTTLE_API")]
    pub api_url: Option<String>,
    #[clap(flatten)]
    pub project_args: ProjectArgs,
    #[clap(subcommand)]
    pub cmd: Command,
}

// Common args for subcommands that deal with projects.
#[derive(Parser, Debug)]
pub struct ProjectArgs {
    /// Specify the working directory
    #[clap(
        global = true,
        long,
        parse(try_from_os_str = parse_path),
        default_value = ".",
    )]
    pub working_directory: PathBuf,
    /// Specify the name of the project (overrides crate name)
    #[clap(global = true, long)]
    pub name: Option<ProjectName>,
}

#[derive(Parser)]
pub enum Command {
    /// deploy a shuttle service
    Deploy(DeployArgs),
    /// manage deployments of a shuttle service
    #[clap(subcommand)]
    Deployment(DeploymentCommand),
    /// create a new shuttle service
    Init(InitArgs),
    /// generate shell completions
    Generate {
        /// which shell
        #[clap(short, long, env, default_value_t = Shell::Bash)]
        shell: Shell,
        /// output to file or stdout by default
        #[clap(short, long, env)]
        output: Option<PathBuf>,
    },
    /// view the status of a shuttle service
    Status,
    /// view the logs of a deployment in this shuttle service
    Logs {
        /// Deployment ID to get logs for. Defaults to currently running deployment
        id: Option<Uuid>,

        #[clap(short, long)]
        /// Follow log output
        follow: bool,
    },
    /// delete this shuttle service
    Delete,
    /// manage secrets for this shuttle service
    Secrets,
    /// create user credentials for the shuttle platform
    Auth(AuthArgs),
    /// login to the shuttle platform
    Login(LoginArgs),
    /// run a shuttle service locally
    Run(RunArgs),
    /// manage a project on shuttle
    #[clap(subcommand)]
    Project(ProjectCommand),
}

#[derive(Parser)]
pub enum DeploymentCommand {
    /// list all the deployments for a service
    List,
    /// view status of a deployment
    Status {
        /// ID of deployment to get status for
        id: Uuid,
    },
}

#[derive(Parser)]
pub enum ProjectCommand {
    /// create an environment for this project on shuttle
    New,
    /// remove this project environment from shuttle
    Rm,
    /// show the status of this project's environment on shuttle
    Status {
        #[clap(short, long)]
        /// Follow status of project command
        follow: bool,
    },
}

#[derive(Parser, Clone, Debug)]
pub struct LoginArgs {
    /// api key for the shuttle platform
    #[clap(long)]
    pub api_key: Option<String>,
}

#[derive(Parser)]
pub struct AuthArgs {
    /// the desired username for the shuttle platform
    #[clap()]
    pub username: String,
}

#[derive(Parser)]
pub struct DeployArgs {
    /// allow dirty working directories to be packaged
    #[clap(long)]
    pub allow_dirty: bool,
    /// allows pre-deploy tests to be skipped
    #[clap(long)]
    pub no_test: bool,
}

#[derive(Parser, Debug)]
pub struct RunArgs {
    /// port to start service on
    #[clap(long, default_value = "8000")]
    pub port: u16,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    /// Initialize with actix-web framework
    #[clap(long="actix-web", conflicts_with_all = &["axum", "rocket", "tide", "tower", "poem", "serenity", "warp", "salvo", "thruster", "no-framework"])]
    pub actix_web: bool,
    /// Initialize with axum framework
    #[clap(long, conflicts_with_all = &["actix-web","rocket", "tide", "tower", "poem", "serenity", "warp", "salvo", "thruster", "no-framework"])]
    pub axum: bool,
    /// Initialize with rocket framework
    #[clap(long, conflicts_with_all = &["actix-web","axum", "tide", "tower", "poem", "serenity", "warp", "salvo", "thruster", "no-framework"])]
    pub rocket: bool,
    /// Initialize with tide framework
    #[clap(long, conflicts_with_all = &["actix-web","axum", "rocket", "tower", "poem", "serenity", "warp", "salvo", "thruster", "no-framework"])]
    pub tide: bool,
    /// Initialize with tower framework
    #[clap(long, conflicts_with_all = &["actix-web","axum", "rocket", "tide", "poem", "serenity", "warp", "salvo", "thruster", "no-framework"])]
    pub tower: bool,
    /// Initialize with poem framework
    #[clap(long, conflicts_with_all = &["actix-web","axum", "rocket", "tide", "tower", "serenity", "warp", "salvo", "thruster", "no-framework"])]
    pub poem: bool,
    /// Initialize with salvo framework
    #[clap(long, conflicts_with_all = &["actix-web","axum", "rocket", "tide", "tower", "poem", "warp", "serenity", "thruster", "no-framework"])]
    pub salvo: bool,
    /// Initialize with serenity framework
    #[clap(long, conflicts_with_all = &["actix-web","axum", "rocket", "tide", "tower", "poem", "warp", "salvo", "thruster", "no-framework"])]
    pub serenity: bool,
    /// Initialize with warp framework
    #[clap(long, conflicts_with_all = &["actix-web","axum", "rocket", "tide", "tower", "poem", "serenity", "salvo", "thruster", "no-framework"])]
    pub warp: bool,
    /// Initialize with thruster framework
    #[clap(long, conflicts_with_all = &["actix-web","axum", "rocket", "tide", "tower", "poem", "warp", "salvo", "serenity", "no-framework"])]
    pub thruster: bool,
    /// Initialize without a framework
    #[clap(long, conflicts_with_all = &["actix-web","axum", "rocket", "tide", "tower", "poem", "warp", "salvo", "serenity", "thruster"])]
    pub no_framework: bool,
    /// Whether to create the environment for this project on Shuttle
    #[clap(long)]
    pub new: bool,
    #[clap(flatten)]
    pub login_args: LoginArgs,
    /// Path to initialize a new shuttle project
    #[clap(
        parse(try_from_os_str = parse_init_path),
        default_value = ".",
    )]
    pub path: PathBuf,
}

impl InitArgs {
    pub fn framework(&self) -> Option<Framework> {
        if self.actix_web {
            Some(Framework::ActixWeb)
        } else if self.axum {
            Some(Framework::Axum)
        } else if self.rocket {
            Some(Framework::Rocket)
        } else if self.tide {
            Some(Framework::Tide)
        } else if self.tower {
            Some(Framework::Tower)
        } else if self.poem {
            Some(Framework::Poem)
        } else if self.salvo {
            Some(Framework::Salvo)
        } else if self.serenity {
            Some(Framework::Serenity)
        } else if self.warp {
            Some(Framework::Warp)
        } else if self.thruster {
            Some(Framework::Thruster)
        } else if self.no_framework {
            Some(Framework::None)
        } else {
            None
        }
    }
}

// Helper function to parse and return the absolute path
fn parse_path(path: &OsStr) -> Result<PathBuf, io::Error> {
    canonicalize(path).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidInput,
            format!("could not turn {path:?} into a real path: {e}"),
        )
    })
}

// Helper function to parse, create if not exists, and return the absolute path
pub(crate) fn parse_init_path(path: &OsStr) -> Result<PathBuf, io::Error> {
    // Create the directory if does not exist
    create_dir_all(path)?;

    parse_path(path)
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    fn init_args_factory(framework: &str) -> InitArgs {
        let mut init_args = InitArgs {
            actix_web: false,
            axum: false,
            rocket: false,
            tide: false,
            tower: false,
            poem: false,
            salvo: false,
            serenity: false,
            warp: false,
            thruster: false,
            no_framework: false,
            new: false,
            login_args: LoginArgs { api_key: None },
            path: PathBuf::new(),
        };

        match framework {
            "actix-web" => init_args.actix_web = true,
            "axum" => init_args.axum = true,
            "rocket" => init_args.rocket = true,
            "tide" => init_args.tide = true,
            "tower" => init_args.tower = true,
            "poem" => init_args.poem = true,
            "salvo" => init_args.salvo = true,
            "serenity" => init_args.serenity = true,
            "warp" => init_args.warp = true,
            "thruster" => init_args.thruster = true,
            "none" => init_args.no_framework = true,
            _ => unreachable!(),
        }

        init_args
    }

    #[test]
    fn test_init_args_framework() {
        for framework in Framework::iter() {
            let args = init_args_factory(&framework.to_string());
            assert_eq!(args.framework(), Some(framework));
        }
    }
}
