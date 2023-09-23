use std::fs::read_to_string;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Output;

use anyhow::{anyhow, bail, Context};
use cargo_metadata::Message;
use cargo_metadata::{Package, Target};
use crossbeam_channel::Sender;
use shuttle_common::{
    constants::{NEXT_NAME, RUNTIME_NAME},
    project::ProjectName,
};
use tracing::{debug, error, trace};

#[derive(Clone, Debug, Eq, PartialEq)]
/// This represents a compiled alpha or shuttle-next service.
pub struct BuiltService {
    pub workspace_path: PathBuf,
    pub manifest_path: PathBuf,
    pub package_name: String,
    pub executable_path: PathBuf,
    pub is_wasm: bool,
}

impl BuiltService {
    /// The directory that contains the crate (that Cargo.toml is in)
    pub fn crate_directory(&self) -> &Path {
        self.manifest_path
            .parent()
            .expect("manifest to be in a directory")
    }

    /// Try to get the service name of a crate from Shuttle.toml in the crate root, if it doesn't
    /// exist get it from the Cargo.toml package name of the crate.
    pub fn service_name(&self) -> anyhow::Result<ProjectName> {
        let shuttle_toml_path = self.crate_directory().join("Shuttle.toml");

        match extract_shuttle_toml_name(shuttle_toml_path) {
            Ok(service_name) => Ok(service_name.parse()?),
            Err(error) => {
                debug!(?error, "failed to get service name from Shuttle.toml");

                // Couldn't get name from Shuttle.toml, use package name instead.
                Ok(self.package_name.parse()?)
            }
        }
    }
}

fn extract_shuttle_toml_name(path: PathBuf) -> anyhow::Result<String> {
    let shuttle_toml =
        read_to_string(path.as_path()).map_err(|_| anyhow!("{} not found", path.display()))?;

    let toml: toml::Value =
        toml::from_str(&shuttle_toml).context("failed to parse Shuttle.toml")?;

    let name = toml
        .get("name")
        .context("couldn't find `name` key in Shuttle.toml")?
        .as_str()
        .context("`name` key in Shuttle.toml must be a string")?
        .to_string();

    Ok(name)
}

/// Given a project directory path, builds the crate
pub async fn build_workspace(
    project_path: &Path,
    release_mode: bool,
    tx: Sender<Message>,
    deployment: bool,
) -> anyhow::Result<Vec<BuiltService>> {
    let project_path = project_path.to_owned();

    let manifest_path = project_path.join("Cargo.toml");

    if !manifest_path.exists() {
        bail!(
            "failed to read the Shuttle project manifest: {}",
            manifest_path.display()
        );
    }
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()?;
    trace!("Cargo metadata parsed");

    let mut alpha_packages = Vec::new();
    let mut next_packages = Vec::new();

    for member in metadata.workspace_packages() {
        if is_next(member) {
            ensure_cdylib(member)?;
            next_packages.push(member);
        } else if is_alpha(member) {
            ensure_binary(member)?;
            alpha_packages.push(member);
        }
    }

    let mut runtimes = Vec::new();

    if !alpha_packages.is_empty() {
        let mut services = compile(
            alpha_packages,
            release_mode,
            false,
            project_path.clone(),
            metadata.target_directory.clone(),
            deployment,
            tx.clone(),
        )
        .await?;
        trace!("alpha packages compiled");

        runtimes.append(&mut services);
    }

    if !next_packages.is_empty() {
        let mut services = compile(
            next_packages,
            release_mode,
            true,
            project_path,
            metadata.target_directory.clone(),
            deployment,
            tx,
        )
        .await?;
        trace!("next packages compiled");

        runtimes.append(&mut services);
    }

    Ok(runtimes)
}

pub async fn clean_crate(project_path: &Path) -> anyhow::Result<Vec<String>> {
    let manifest_path = project_path.join("Cargo.toml");
    if !manifest_path.exists() {
        bail!("failed to read the Shuttle project manifest");
    }
    let Output {
        status,
        stdout,
        stderr,
    } = tokio::process::Command::new("cargo")
        .arg("clean")
        .arg("--manifest-path")
        .arg(manifest_path.to_str().unwrap())
        .output()
        .await
        .unwrap();

    if status.success() {
        let lines = vec![String::from_utf8(stderr)?, String::from_utf8(stdout)?];
        Ok(lines)
    } else {
        Err(anyhow!(
            "cargo clean failed with exit code {} and error {}",
            status.to_string(),
            String::from_utf8(stderr)?
        ))
    }
}

fn is_next(package: &Package) -> bool {
    package
        .dependencies
        .iter()
        .any(|dependency| dependency.name == NEXT_NAME)
}

fn is_alpha(package: &Package) -> bool {
    package
        .dependencies
        .iter()
        .any(|dependency| dependency.name == RUNTIME_NAME)
}

/// Make sure the project is a binary for alpha projects.
fn ensure_binary(package: &Package) -> anyhow::Result<()> {
    if package.targets.iter().any(|target| target.is_bin()) {
        Ok(())
    } else {
        bail!("Your Shuttle project must be a binary.")
    }
}

/// Make sure "cdylib" is set for shuttle-next projects, else set it if possible.
fn ensure_cdylib(package: &Package) -> anyhow::Result<()> {
    if package.targets.iter().any(is_cdylib) {
        Ok(())
    } else {
        bail!("Your Shuttle next project must be a library. Please add `[lib]` to your Cargo.toml file.")
    }
}

fn is_cdylib(target: &Target) -> bool {
    target.kind.iter().any(|kind| kind == "cdylib")
}

async fn compile(
    packages: Vec<&Package>,
    release_mode: bool,
    wasm: bool,
    project_path: PathBuf,
    target_path: impl Into<PathBuf>,
    deployment: bool,
    tx: Sender<Message>,
) -> anyhow::Result<Vec<BuiltService>> {
    let manifest_path = project_path.join("Cargo.toml");
    if !manifest_path.exists() {
        bail!("failed to read the Shuttle project manifest");
    }
    let target_path = target_path.into();

    let mut cargo = tokio::process::Command::new("cargo");
    cargo
        .arg("build")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--color=always") // piping disables auto color, but we want it
        .current_dir(project_path.as_path());

    if deployment {
        cargo.arg("--jobs=4");
    }

    for package in &packages {
        cargo.arg("--package").arg(package.name.as_str());
    }

    let profile = if release_mode {
        cargo.arg("--profile").arg("release");
        "release"
    } else {
        cargo.arg("--profile").arg("dev");
        "debug"
    };

    if wasm {
        cargo.arg("--target").arg("wasm32-wasi");
    }

    let (reader, writer) = os_pipe::pipe()?;
    let writer_clone = writer.try_clone()?;
    cargo.stdout(writer);
    cargo.stderr(writer_clone);

    let mut handle = cargo.spawn()?;

    tokio::task::spawn_blocking(move || {
        let reader = std::io::BufReader::new(reader);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Err(error) = tx.send(Message::TextLine(line)) {
                    error!("failed to send cargo message on channel: {error}");
                };
            } else {
                error!("Failed to read Cargo log messages");
            };
        }
    });

    let command = handle.wait().await?;

    if !command.success() {
        bail!("Build failed. Is the Shuttle runtime missing?");
    }

    let services = packages
        .iter()
        .map(|package| {
            let path = if wasm {
                let mut path: PathBuf = [
                    project_path.clone(),
                    target_path.clone(),
                    "wasm32-wasi".into(),
                    profile.into(),
                    package.name.replace('-', "_").into(),
                ]
                .iter()
                .collect();
                path.set_extension("wasm");
                path
            } else {
                let mut path: PathBuf = [
                    project_path.clone(),
                    target_path.clone(),
                    profile.into(),
                    package.name.clone().into(),
                ]
                .iter()
                .collect();
                path.set_extension(std::env::consts::EXE_EXTENSION);
                path
            };

            BuiltService {
                workspace_path: project_path.clone(),
                manifest_path: package.manifest_path.clone().into_std_path_buf(),
                package_name: package.name.clone(),
                executable_path: path,
                is_wasm: wasm,
            }
        })
        .collect();

    Ok(services)
}
