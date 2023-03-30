use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context};
use cargo::core::compiler::{CompileKind, CompileMode, CompileTarget, MessageFormat};
use cargo::core::{Package, Shell, Verbosity, Workspace};
use cargo::ops::{self, clean, compile, CleanOptions, CompileOptions};
use cargo::util::homedir;
use cargo::util::interning::InternedString;
use cargo::Config;
use cargo_metadata::Message;
use crossbeam_channel::Sender;
use pipe::PipeWriter;
use tracing::{error, trace};

use crate::{NEXT_NAME, RUNTIME_NAME};

#[derive(Clone, Debug, Eq, PartialEq)]
/// How to run/build the project
pub enum Runtime {
    Next(PathBuf),
    Alpha(PathBuf),
}

/// Given a project directory path, builds the crate
pub async fn build_workspace(
    project_path: &Path,
    release_mode: bool,
    tx: Sender<Message>,
) -> anyhow::Result<Vec<Runtime>> {
    let (read, write) = pipe::pipe();
    let project_path = project_path.to_owned();

    // This needs to be on a separate thread, else deployer will block (reason currently unknown :D)
    tokio::task::spawn_blocking(move || {
        trace!("started thread to to capture build output stream");
        for message in Message::parse_stream(read) {
            trace!(?message, "parsed cargo message");
            match message {
                Ok(message) => {
                    if let Err(error) = tx.send(message) {
                        error!("failed to send cargo message on channel: {error}");
                    }
                }
                Err(error) => {
                    error!("failed to parse cargo message: {error}");
                }
            }
        }
    });

    let config = get_config(write)?;
    let manifest_path = project_path.join("Cargo.toml");
    let ws = Workspace::new(&manifest_path, &config)?;
    check_no_panic(&ws)?;

    let mut alpha_packages = Vec::new();
    let mut next_packages = Vec::new();

    for member in ws.members() {
        if is_next(member) {
            ensure_cdylib(member)?;
            next_packages.push(member.name().to_string());
        } else if is_alpha(member) {
            ensure_binary(member)?;
            alpha_packages.push(member.name().to_string());
        }
    }

    let mut runtimes = Vec::new();

    if !alpha_packages.is_empty() {
        let opts = get_compile_options(&config, alpha_packages, release_mode, false)?;
        let compilation = compile(&ws, &opts)?;

        let mut alpha_binaries = compilation
            .binaries
            .iter()
            .map(|binary| Runtime::Alpha(binary.path.clone()))
            .collect();

        runtimes.append(&mut alpha_binaries);
    }

    if !next_packages.is_empty() {
        let opts = get_compile_options(&config, next_packages, release_mode, true)?;
        let compilation = compile(&ws, &opts)?;

        let mut next_libraries = compilation
            .cdylibs
            .iter()
            .map(|binary| Runtime::Next(binary.path.clone()))
            .collect();

        runtimes.append(&mut next_libraries);
    }

    Ok(runtimes)
}

pub fn clean_crate(project_path: &Path, release_mode: bool) -> anyhow::Result<Vec<String>> {
    let (read, write) = pipe::pipe();
    let project_path = project_path.to_owned();

    tokio::task::spawn_blocking(move || {
        let config = get_config(write).unwrap();
        let manifest_path = project_path.join("Cargo.toml");
        let ws = Workspace::new(&manifest_path, &config).unwrap();

        let requested_profile = if release_mode {
            InternedString::new("release")
        } else {
            InternedString::new("dev")
        };

        let opts = CleanOptions {
            config: &config,
            spec: Vec::new(),
            targets: Vec::new(),
            requested_profile,
            profile_specified: true,
            doc: false,
        };

        clean(&ws, &opts).unwrap();
    });

    let mut lines = Vec::new();

    for message in Message::parse_stream(read) {
        trace!(?message, "parsed cargo message");
        match message {
            Ok(Message::TextLine(line)) => {
                lines.push(line);
            }
            Ok(_) => {}
            Err(error) => {
                error!("failed to parse cargo message: {error}");
            }
        }
    }

    Ok(lines)
}

/// Get the default compile config with output redirected to writer
pub fn get_config(writer: PipeWriter) -> anyhow::Result<Config> {
    let mut shell = Shell::from_write(Box::new(writer));
    shell.set_verbosity(Verbosity::Normal);
    let cwd = std::env::current_dir()
        .with_context(|| "couldn't get the current directory of the process")?;
    let homedir = homedir(&cwd).ok_or_else(|| {
        anyhow!(
            "Cargo couldn't find your home directory. \
                 This probably means that $HOME was not set."
        )
    })?;

    Ok(Config::new(shell, cwd, homedir))
}

/// Get options to compile in build mode
fn get_compile_options(
    config: &Config,
    packages: Vec<String>,
    release_mode: bool,
    wasm: bool,
) -> anyhow::Result<CompileOptions> {
    let mut opts = CompileOptions::new(config, CompileMode::Build)?;
    opts.build_config.message_format = MessageFormat::Json {
        render_diagnostics: false,
        short: false,
        ansi: false,
    };

    opts.build_config.requested_profile = if release_mode {
        InternedString::new("release")
    } else {
        InternedString::new("dev")
    };

    // This sets the max workers for cargo build to 4 for release mode (aka deployment),
    // but leaves it as default (num cpus) for local runs
    if release_mode {
        opts.build_config.jobs = 4
    };

    opts.build_config.requested_kinds = vec![if wasm {
        CompileKind::Target(CompileTarget::new("wasm32-wasi")?)
    } else {
        CompileKind::Host
    }];

    opts.spec = ops::Packages::Packages(packages);

    Ok(opts)
}

fn is_next(package: &Package) -> bool {
    package
        .dependencies()
        .iter()
        .any(|dependency| dependency.package_name() == NEXT_NAME)
}

fn is_alpha(package: &Package) -> bool {
    package
        .dependencies()
        .iter()
        .any(|dependency| dependency.package_name() == RUNTIME_NAME)
}

/// Make sure the project is a binary for alpha projects.
fn ensure_binary(package: &Package) -> anyhow::Result<()> {
    if package.targets().iter().any(|target| target.is_bin()) {
        Ok(())
    } else {
        bail!("Your Shuttle project must be a binary.")
    }
}

/// Make sure "cdylib" is set for shuttle-next projects, else set it if possible.
fn ensure_cdylib(package: &Package) -> anyhow::Result<()> {
    if package.targets().iter().any(|target| target.is_lib()) {
        Ok(())
    } else {
        bail!("Your Shuttle next project must be a library. Please add `[lib]` to your Cargo.toml file.")
    }
}

/// Ensure `panic = "abort"` is not set:
fn check_no_panic(ws: &Workspace) -> anyhow::Result<()> {
    if let Some(profiles) = ws.profiles() {
        for profile in profiles.get_all().values() {
            if profile.panic.as_deref() == Some("abort") {
                return Err(anyhow!("Your Shuttle project cannot have panics that abort. Please ensure your Cargo.toml does not contain `panic = \"abort\"` for any profiles."));
            }
        }
    }

    Ok(())
}
