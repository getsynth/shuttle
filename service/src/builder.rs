use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{anyhow, bail, Context};
use cargo_metadata::{Metadata, Package};
use shuttle_common::constants::RUNTIME_NAME;
use tokio::io::AsyncBufReadExt;
use tracing::{debug, error, info, trace};

#[derive(Clone, Debug, Eq, PartialEq)]
/// This represents a compiled alpha service
pub struct BuiltService {
    pub workspace_path: PathBuf,
    pub manifest_path: PathBuf,
    pub package_name: String,
    pub executable_path: PathBuf,
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
    pub fn service_name(&self) -> anyhow::Result<String> {
        let shuttle_toml_path = self.crate_directory().join("Shuttle.toml");

        match extract_shuttle_toml_name(shuttle_toml_path) {
            Ok(service_name) => Ok(service_name),
            Err(error) => {
                debug!(?error, "failed to get service name from Shuttle.toml");

                // Couldn't get name from Shuttle.toml, use package name instead.
                Ok(self.package_name.clone())
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
    tx: tokio::sync::mpsc::Sender<String>,
    deployment: bool,
) -> anyhow::Result<Vec<BuiltService>> {
    let project_path = project_path.to_owned();
    let manifest_path = project_path.join("Cargo.toml");
    if !manifest_path.exists() {
        bail!("Cargo manifest file not found: {}", manifest_path.display());
    }

    // Cargo's "Downloading ..." lines are quite verbose.
    // Instead, a custom message is printed if the download takes significant time.
    // Cargo seems to have similar logic, where it prints nothing if this step takes little time.
    let mut command = tokio::process::Command::new("cargo");
    command
        .arg("fetch")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--color=always")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let notification = tokio::spawn({
        let tx = tx.clone();
        async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            tx.send("      Downloading crates...".into())
                .await
                .expect("log receiver to exist");
        }
    });
    if !command.status().await?.success() {
        tx.send("      Failed to fetch crates".into())
            .await
            .expect("log receiver to exist");
    }
    notification.abort();

    let metadata = async_cargo_metadata(manifest_path.as_path()).await?;
    let packages = find_shuttle_packages(&metadata)?;
    if packages.is_empty() {
        bail!(
            "Did not find any packages that Shuttle can run. \
            Make sure your crate has a binary target that uses `#[shuttle_runtime::main]`."
        );
    }

    let services = compile(
        packages,
        release_mode,
        project_path.clone(),
        metadata.target_directory.clone(),
        deployment,
        tx.clone(),
    )
    .await?;
    trace!("packages compiled");

    Ok(services)
}

pub async fn async_cargo_metadata(manifest_path: &Path) -> anyhow::Result<Metadata> {
    let metadata = {
        // Modified implementaion of `cargo_metadata::MetadataCommand::exec` (from v0.15.3).
        // Uses tokio Command instead of std, to make this operation non-blocking.
        let mut cmd = tokio::process::Command::from(
            cargo_metadata::MetadataCommand::new()
                .manifest_path(manifest_path)
                .cargo_command(),
        );

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(cargo_metadata::Error::CargoMetadata {
                stderr: String::from_utf8(output.stderr)?,
            })?;
        }
        let json = std::str::from_utf8(&output.stdout)?
            .lines()
            .find(|line| line.starts_with('{'))
            .ok_or(cargo_metadata::Error::NoJson)?;
        cargo_metadata::MetadataCommand::parse(json)?
    };
    trace!("Cargo metadata parsed");

    Ok(metadata)
}

pub fn find_shuttle_packages(metadata: &Metadata) -> anyhow::Result<Vec<Package>> {
    let mut packages = Vec::new();
    for member in metadata.workspace_packages() {
        // skip non-Shuttle-related crates
        // (must have runtime dependency and not be just a library)
        let has_runtime_dep = member
            .dependencies
            .iter()
            .any(|dependency| dependency.name == RUNTIME_NAME);
        let has_binary_target = member.targets.iter().any(|t| t.is_bin());
        if !(has_runtime_dep && has_binary_target) {
            continue;
        }

        let mut shuttle_deps = member
            .dependencies
            .iter()
            .filter(|&d| d.name.starts_with("shuttle-"))
            .map(|d| format!("{} '{}'", d.name, d.req))
            .collect::<Vec<_>>();
        shuttle_deps.sort();
        info!(name = member.name, deps = ?shuttle_deps, "Found workspace member with shuttle dependencies");
        packages.push(member.to_owned());
    }

    Ok(packages)
}

// Only used in deployer
pub async fn clean_crate(project_path: &Path) -> anyhow::Result<()> {
    let manifest_path = project_path.join("Cargo.toml");
    if !manifest_path.exists() {
        bail!("Cargo manifest file not found: {}", manifest_path.display());
    }
    if !tokio::process::Command::new("cargo")
        .arg("clean")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--offline")
        .status()
        .await?
        .success()
    {
        bail!("`cargo clean` failed. Did you build anything yet?");
    }

    Ok(())
}

async fn compile(
    packages: Vec<Package>,
    release_mode: bool,
    project_path: PathBuf,
    target_path: impl Into<PathBuf>,
    deployment: bool,
    tx: tokio::sync::mpsc::Sender<String>,
) -> anyhow::Result<Vec<BuiltService>> {
    let manifest_path = project_path.join("Cargo.toml");
    if !manifest_path.exists() {
        bail!("Cargo manifest file not found: {}", manifest_path.display());
    }
    let target_path = target_path.into();

    // TODO?: Use https://crates.io/crates/escargot instead

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--color=always") // piping disables auto color, but we want it
        .current_dir(project_path.as_path());

    if deployment {
        cmd.arg("--jobs=4");
    }

    // TODO: Compile only one binary target in the package.
    for package in &packages {
        if package.features.contains_key("shuttle") {
            cmd.arg("--no-default-features").arg("--features=shuttle");
        }
        cmd.arg("--package").arg(package.name.as_str());
    }

    let profile = if release_mode {
        cmd.arg("--release");
        "release"
    } else {
        "debug"
    };

    cmd.stderr(Stdio::piped());
    cmd.stdout(Stdio::null());
    let mut handle = cmd.spawn()?;
    let reader = tokio::io::BufReader::new(handle.stderr.take().unwrap());
    tokio::spawn(async move {
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await.unwrap() {
            let _ = tx
                .send(line)
                .await
                .map_err(|error| error!(error = &error as &dyn std::error::Error));
        }
    });
    let status = handle.wait().await?;
    if !status.success() {
        bail!("Build failed. Is the Shuttle runtime missing?");
    }

    let services = packages
        .iter()
        .map(|package| {
            let mut path: PathBuf = [
                project_path.clone(),
                target_path.clone(),
                profile.into(),
                package.name.clone().into(),
            ]
            .iter()
            .collect();
            path.set_extension(std::env::consts::EXE_EXTENSION);

            BuiltService {
                workspace_path: project_path.clone(),
                manifest_path: package.manifest_path.clone().into_std_path_buf(),
                package_name: package.name.clone(),
                executable_path: path,
            }
        })
        .collect();

    Ok(services)
}
