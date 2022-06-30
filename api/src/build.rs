use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use rocket::tokio;
use rocket::tokio::io::AsyncWriteExt;
use shuttle_service::loader::build_crate;
use uuid::Uuid;

#[cfg(debug_assertions)]
pub const DEFAULT_FS_ROOT: &str = "/tmp/shuttle/crates/";

#[cfg(not(debug_assertions))]
// as per: https://stackoverflow.com/questions/1510104/where-to-store-application-data-non-user-specific-on-linux
pub const DEFAULT_FS_ROOT: &str = "/var/lib/shuttle/crates/";

pub(crate) struct Build {
    pub(crate) so_path: PathBuf,
}

// remove the trait at some point
#[async_trait]
pub(crate) trait BuildSystem: Send + Sync {
    async fn build(
        &self,
        crate_bytes: &[u8],
        project: &str,
        buf: Box<dyn std::io::Write + Send>,
    ) -> Result<Build>;

    fn fs_root(&self) -> PathBuf;
}

/// A basic build system that uses the file system for caching and storage
pub(crate) struct FsBuildSystem {
    fs_root: PathBuf,
}

impl FsBuildSystem {
    /// Intialises the FS Build System. Optionally you can define the root
    /// of its file system. If unspecified, will default to `FS_ROOT`.
    /// The FS Build System will fail to intialise if the directory does not.
    /// exist
    pub(crate) fn initialise(path: Option<PathBuf>) -> Result<Self> {
        let fs_root = path.unwrap_or_else(|| PathBuf::from(DEFAULT_FS_ROOT));
        if !(fs_root.exists()) {
            return Err(anyhow!(
                r#"
            Failed to initialise FS Build System.
            The path {:?} does not exist.
            Please create the directory to continue with deployment"#,
                &fs_root
            ));
        }
        Ok(FsBuildSystem { fs_root })
    }

    /// Given an api key and project name returns a `PathBuf` to the project
    /// If the directory does not exist, creates it.
    fn project_path(&self, project: &str) -> Result<PathBuf> {
        let mut project_path = self.fs_root.clone();
        project_path.push(project);
        // create directory
        std::fs::create_dir_all(&project_path)?;
        Ok(project_path)
    }
}

#[async_trait]
impl BuildSystem for FsBuildSystem {
    async fn build(
        &self,
        crate_bytes: &[u8],
        project_name: &str,
        buf: Box<dyn std::io::Write + Send>,
    ) -> Result<Build> {
        // project path
        let project_path = self.project_path(project_name)?;
        debug!("Project path: {}", project_path.display());

        // clear directory
        clear_project_dir(&project_path).context(
            "there was an issue cleaning the project directory. Please try again in a bit.",
        )?;

        // crate path
        let crate_path = crate_location(&project_path, project_name);
        debug!("Crate path: {}", crate_path.display());

        // create target file
        let mut target_file = tokio::fs::File::create(&crate_path).await?;

        // write bytes to file
        target_file.write_all(crate_bytes).await?;

        // extract tarball
        extract_tarball(&crate_path, &project_path)?;

        // run cargo build (--debug for now)
        let so_path = build_crate(&project_path, buf)?;

        // create uniquely named so file to satisfy `libloading`
        let so_path = create_unique_named_so_file(&project_path, &so_path)?;

        // create marker file
        create_so_marker(&project_path, &so_path)?;

        Ok(Build { so_path })
    }

    fn fs_root(&self) -> PathBuf {
        self.fs_root.clone()
    }
}

/// Creates a marker file with the location of the `so` file
/// so that we can use it when bootstrapping the deployment
/// system
fn create_so_marker(project_path: &Path, so_path: &Path) -> Result<()> {
    let marker_path = project_path.join(".shuttle_marker");
    // unwraps here are ok since we are writing a valid `Path`
    Ok(std::fs::write(&marker_path, so_path.to_str().unwrap())?)
}

/// Copies the original `so` file to the project directory with a random name
/// to appease `libloading`.
fn create_unique_named_so_file(project_path: &Path, so_path: &Path) -> Result<PathBuf> {
    let so_unique_path = project_path.join(&format!("{}.so", Uuid::new_v4()));
    std::fs::copy(so_path, &so_unique_path)?;
    Ok(so_unique_path)
}

/// Clear everything which is not the target folder from the project path
fn clear_project_dir(project_path: &Path) -> Result<()> {
    // remove everything except for the target folder
    std::fs::read_dir(project_path)?
        .into_iter()
        .filter_map(|dir| dir.ok())
        .filter(|dir| {
            if dir.file_name() == "target" {
                return false;
            }

            if let Some(Some("so")) = dir.path().extension().map(|f| f.to_str()) {
                return false;
            }

            true
        })
        .try_for_each::<_, Result<_, io::Error>>(|dir| {
            if let Ok(file) = dir.file_type() {
                if file.is_dir() {
                    std::fs::remove_dir_all(&dir.path())?;
                } else if file.is_file() {
                    std::fs::remove_file(&dir.path())?;
                } else if file.is_symlink() {
                    // there shouldn't be any symlinks here
                    unimplemented!()
                }
            }
            Ok(())
        })?;
    Ok(())
}

/// Given a project path and a project name, return the location of the .crate file
fn crate_location(project_path: &Path, project_name: &str) -> PathBuf {
    project_path.join(project_name).with_extension("crate")
}

/// Given a .crate file (which is a gzipped tarball), extracts the contents
/// into the project_path
fn extract_tarball(crate_path: &Path, project_path: &Path) -> Result<()> {
    let output = Command::new("tar")
        .arg("-xzvf") // extract
        .arg(crate_path)
        .arg("-C") // target
        .arg(project_path)
        .arg("--strip-components") // remove top-level directory
        .arg("1")
        .arg("--touch") // touch to update mtime for cargo
        .output()?;
    if !output.status.success() {
        let err = String::from_utf8(output.stderr).unwrap_or_default();
        Err(anyhow::Error::msg(err).context(anyhow!("failed to unpack cargo archive")))
    } else {
        Ok(())
    }
}
