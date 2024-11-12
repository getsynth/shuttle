use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use shuttle_common::constants::API_URL_DEFAULT;
use shuttle_common::constants::API_URL_DEFAULT_BETA;
use tracing::trace;

use crate::args::ProjectArgs;
use crate::init::create_or_update_ignore_file;

/// Helper trait for dispatching fs ops for different config files
pub trait ConfigManager: Sized {
    fn directory(&self) -> PathBuf;

    fn file(&self) -> PathBuf;

    fn path(&self) -> PathBuf {
        self.directory().join(self.file())
    }

    fn exists(&self) -> bool {
        self.path().exists()
    }

    fn create<C>(&self) -> Result<()>
    where
        C: Serialize + Default,
    {
        if self.exists() {
            return Ok(());
        }
        let config = C::default();
        self.save(&config)
    }

    fn open<C>(&self) -> Result<C>
    where
        C: for<'de> Deserialize<'de>,
    {
        let path = self.path();
        let config_string = File::open(&path)
            .and_then(|mut f| {
                let mut buf = String::new();
                f.read_to_string(&mut buf)?;
                Ok(buf)
            })
            .with_context(|| anyhow!("Unable to read configuration file: {}", path.display()))?;
        toml::from_str(config_string.as_str())
            .with_context(|| anyhow!("Invalid global configuration file: {}", path.display()))
    }

    fn save<C>(&self, config: &C) -> Result<()>
    where
        C: Serialize,
    {
        let path = self.path();
        std::fs::create_dir_all(path.parent().unwrap())?;

        let mut config_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        let config_str = toml::to_string_pretty(config).unwrap();
        config_file.write(config_str.as_bytes()).with_context(|| {
            anyhow!(
                "Could not write the global configuration file: {}",
                path.display()
            )
        })?;
        Ok(())
    }
}

pub struct GlobalConfigManager;

impl ConfigManager for GlobalConfigManager {
    fn directory(&self) -> PathBuf {
        let shuttle_config_dir = dirs::config_dir()
            .ok_or_else(|| {
                anyhow!(
                    "Could not find a configuration directory. Your operating system may not be supported."
                )
            })
            .unwrap();
        shuttle_config_dir.join("shuttle")
    }

    fn file(&self) -> PathBuf {
        PathBuf::from("config.toml")
    }
}

/// An impl of [`ConfigManager`] which is localised to a working directory
pub struct LocalConfigManager {
    working_directory: PathBuf,
    file_name: String,
}

impl LocalConfigManager {
    pub fn new<P: AsRef<Path>>(working_directory: P, file_name: String) -> Self {
        Self {
            working_directory: working_directory.as_ref().to_path_buf(),
            file_name,
        }
    }
}

impl ConfigManager for LocalConfigManager {
    fn directory(&self) -> PathBuf {
        self.working_directory.clone()
    }

    fn file(&self) -> PathBuf {
        PathBuf::from(&self.file_name)
    }
}

/// Global client config for things like API keys.
#[derive(Deserialize, Serialize, Default)]
pub struct GlobalConfig {
    api_key: Option<String>,
    pub api_url: Option<String>,
}

impl GlobalConfig {
    pub fn api_key(&self) -> Option<String> {
        self.api_key.clone()
    }

    pub fn set_api_key(&mut self, api_key: String) -> Option<String> {
        self.api_key.replace(api_key)
    }

    pub fn clear_api_key(&mut self) {
        self.api_key = None;
    }

    pub fn api_url(&self) -> Option<String> {
        self.api_url.clone()
    }
}

/// Shuttle.toml schema (User-facing project-local config)
#[derive(Deserialize, Serialize, Default)]
pub struct ProjectConfig {
    // unused on new platform, but still used for project names in local runs
    pub name: Option<String>,
    /// Deprecated, now [`ProjectDeployConfig::include`]
    pub assets: Option<Vec<String>>,
    pub deploy: Option<ProjectDeployConfig>,
    pub build: Option<ProjectBuildConfig>,
}
/// Deployment command config
#[derive(Deserialize, Serialize, Default)]
pub struct ProjectDeployConfig {
    /// Successor to `assets`.
    /// Patterns of ignored files that should be included in deployments.
    pub include: Option<Vec<String>>,
    /// Set to true to deny deployments with uncommited changes. (use `--allow-dirty` to override)
    pub deny_dirty: Option<bool>,
}
/// Builder config
#[derive(Deserialize, Serialize, Default)]
pub struct ProjectBuildConfig {
    /// Successor to `build_assets`.
    /// Patterns of files that should be copied from the build to the runtime container.
    pub assets: Option<Vec<String>>,
}

/// .shuttle/config.toml schema (internal project-local config)
#[derive(Deserialize, Serialize, Default)]
pub struct InternalProjectConfig {
    // should be in internal local config
    pub id: Option<String>,
}

/// A handler for configuration files. The type parameter `M` is the [`ConfigManager`] which handles
/// indirection around file location and serde. The type parameter `C` is the configuration content.
///
/// # Usage
/// ```rust,no_run
/// # use cargo_shuttle::config::{Config, GlobalConfig, GlobalConfigManager};
/// #
/// let mut config = Config::new(GlobalConfigManager);
/// config.open().unwrap();
/// let content: &GlobalConfig = config.as_ref().unwrap();
/// ```
pub struct Config<M, C> {
    pub manager: M,
    config: Option<C>,
}

impl<M, C> Config<M, C>
where
    M: ConfigManager,
    C: Serialize + for<'de> Deserialize<'de>,
{
    /// Creates a new [`Config`] instance, without opening the underlying file
    pub fn new(manager: M) -> Self {
        Self {
            manager,
            config: None,
        }
    }

    /// Opens the underlying config file, as handled by the [`ConfigManager`]
    pub fn open(&mut self) -> Result<()> {
        let config = self.manager.open()?;
        self.config = Some(config);
        Ok(())
    }

    /// Saves the current state of the config to the file managed by the [`ConfigManager`]
    pub fn save(&self) -> Result<()> {
        self.manager.save(self.config.as_ref().unwrap())
    }

    /// Check if the file managed by the [`ConfigManager`] exists
    pub fn exists(&self) -> bool {
        self.manager.exists()
    }

    /// Replace the current config state with a new value.
    ///
    /// Does not persist the change to disk. Use [`Config::save`] for that.
    pub fn replace(&mut self, config: C) -> Option<C> {
        self.config.replace(config)
    }

    /// Get a mut ref to the underlying config state. Returns `None` if the config has not been
    /// opened.
    pub fn as_mut(&mut self) -> Option<&mut C> {
        self.config.as_mut()
    }

    /// Get a ref to the underlying config state. Returns `None` if the config has not been
    /// opened.
    pub fn as_ref(&self) -> Option<&C> {
        self.config.as_ref()
    }

    /// Ask the [`ConfigManager`] to create a default config file at the location it manages.
    ///
    /// If the file already exists, is a no-op.
    pub fn create(&self) -> Result<()>
    where
        C: Default,
    {
        self.manager.create::<C>()
    }
}

/// A wrapper around our two sources of configuration and overrides:
/// - Global config
/// - Local config
pub struct RequestContext {
    global: Config<GlobalConfigManager, GlobalConfig>,
    project: Option<Config<LocalConfigManager, ProjectConfig>>,
    project_internal: Option<Config<LocalConfigManager, InternalProjectConfig>>,
    api_url: Option<String>,
}

impl RequestContext {
    /// Create a [`RequestContext`], only loading in the global configuration details.
    pub fn load_global() -> Result<Self> {
        let mut global = Config::new(GlobalConfigManager);
        if !global.exists() {
            global.create()?;
        }
        global
            .open()
            .context("Unable to load global configuration")?;
        Ok(Self {
            global,
            project: None,
            project_internal: None,
            api_url: None,
        })
    }

    pub fn load_local_internal(&mut self, project_args: &ProjectArgs) -> Result<()> {
        let workspace_path = project_args
            .workspace_path()
            .unwrap_or(project_args.working_directory.clone());

        trace!(
            "looking for .shuttle/config.toml in {}",
            workspace_path.display()
        );
        let local_manager =
            LocalConfigManager::new(workspace_path, ".shuttle/config.toml".to_string());
        let mut project_internal = Config::new(local_manager);
        if !project_internal.exists() {
            trace!("no local .shuttle/config.toml found");
            project_internal.replace(InternalProjectConfig::default());
        } else {
            trace!("found a local .shuttle/config.toml");
            project_internal.open()?;
        }

        let config = project_internal.as_mut().unwrap();

        // Project id is preferred in this order:
        // 1. Name given on command line
        // 2. Name from .shuttle/config.toml file
        match (&project_args.name_or_id, &config.id) {
            // Command-line name parameter trumps everything
            (Some(id_from_args), _) => {
                trace!("using command-line project id");
                config.id = Some(id_from_args.clone());
            }
            // If key exists in config then keep it as it is
            (None, Some(_)) => {
                trace!("using .shuttle/config.toml project id");
            }
            (None, None) => {
                trace!("no project id in args or config found");
            }
        };

        self.project_internal = Some(project_internal);

        Ok(())
    }

    pub fn set_project_id(&mut self, id: String) {
        *self.project_internal.as_mut().unwrap().as_mut().unwrap() =
            InternalProjectConfig { id: Some(id) };
    }

    pub fn save_local_internal(&mut self) -> Result<()> {
        // if self.project_internal.is_some() {
        self.project_internal.as_ref().unwrap().save()?;
        // }

        // write updated gitignore file to root of workspace
        // TODO: assumes git is used
        create_or_update_ignore_file(
            &self
                .project
                .as_ref()
                .unwrap()
                .manager
                .working_directory
                .join(".gitignore"),
        )
        .context("Failed to create .gitignore file")?;

        Ok(())
    }

    /// Load the project configuration at the given `working_directory`
    ///
    /// Ensures that if `--name` is not specified on the command-line, and either the project
    /// file does not exist, or it has not set the `name` key then the `ProjectConfig` instance
    /// has `ProjectConfig.name = Some("crate-name")`.
    pub fn load_local(&mut self, project_args: &ProjectArgs) -> Result<()> {
        // Shuttle.toml
        let project = Self::get_local_config(project_args)?;

        self.project = Some(project);

        Ok(())
    }

    pub fn get_local_config(
        project_args: &ProjectArgs,
    ) -> Result<Config<LocalConfigManager, ProjectConfig>> {
        let workspace_path = project_args
            .workspace_path()
            .unwrap_or(project_args.working_directory.clone());

        trace!("looking for Shuttle.toml in {}", workspace_path.display());
        let local_manager = LocalConfigManager::new(workspace_path, "Shuttle.toml".to_string());
        let mut project = Config::new(local_manager);
        if !project.exists() {
            trace!("no local Shuttle.toml found");
            project.replace(ProjectConfig::default());
        } else {
            trace!("found a local Shuttle.toml");
            project.open()?;
        }

        let config = project.as_mut().unwrap();

        // Project names are preferred in this order:
        // 1. Name given on command line
        // 2. Name from Shuttle.toml file
        // 3. Name from Cargo.toml package if it's a crate
        // 4. Name from the workspace directory if it's a workspace
        match (&project_args.name_or_id, &config.name) {
            // Command-line name parameter trumps everything
            (Some(name_from_args), _) => {
                trace!("using command-line project name");
                config.name = Some(name_from_args.clone());
            }
            // If key exists in config then keep it as it is
            (None, Some(_)) => {
                trace!("using Shuttle.toml project name");
            }
            // If name key is not in project config, then we infer from crate name
            (None, None) => {
                trace!("using crate name as project name");
                config.name = Some(project_args.project_name()?);
            }
        };

        Ok(project)
    }

    pub fn set_api_url(&mut self, api_url: Option<String>) {
        self.api_url = api_url;
    }

    pub fn api_url(&self, beta: bool) -> String {
        if let Some(api_url) = self.api_url.clone() {
            api_url
        } else if let Some(api_url) = self.global.as_ref().unwrap().api_url() {
            api_url
        } else if beta {
            API_URL_DEFAULT_BETA.to_string()
        } else {
            API_URL_DEFAULT.to_string()
        }
    }

    /// Get the API key from the `SHUTTLE_API_KEY` env variable, or
    /// otherwise from the global configuration. Returns an error if
    /// an API key is not set.
    pub fn api_key(&self) -> Result<String> {
        match std::env::var("SHUTTLE_API_KEY") {
            Ok(key) => Ok(key),
            Err(_) => match self.global.as_ref().unwrap().api_key() {
                Some(key) => Ok(key),
                None => Err(anyhow!(
                    "Configuration file: `{}`",
                    self.global.manager.path().display()
                )
                .context(anyhow!(
                    "No valid API key found, try logging in first with `shuttle login`"
                ))),
            },
        }
    }

    /// Get the current context working directory
    ///
    /// # Panics
    /// Panics if project configuration has not been loaded.
    pub fn working_directory(&self) -> &Path {
        self.project
            .as_ref()
            .unwrap()
            .manager
            .working_directory
            .as_path()
    }

    /// Set the API key to the global configuration. Will persist the file.
    pub fn set_api_key(&mut self, api_key: String) -> Result<()> {
        self.global.as_mut().unwrap().set_api_key(api_key);
        self.global.save()
    }

    pub fn clear_api_key(&mut self) -> Result<()> {
        self.global.as_mut().unwrap().clear_api_key();
        self.global.save()
    }

    /// Get the current project name.
    ///
    /// # Panics
    /// Panics if the project configuration has not been loaded.
    pub fn project_name(&self) -> &str {
        self.project
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .name
            .as_ref()
            .unwrap()
            .as_str()
    }

    /// # Panics
    /// Panics if the project configuration has not been loaded.
    pub fn include(&self) -> Option<&Vec<String>> {
        self.project
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .deploy
            .as_ref()
            .and_then(|d| d.include.as_ref())
            .or(self
                .project
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .assets
                .as_ref())
    }

    /// # Panics
    /// Panics if the project configuration has not been loaded.
    pub fn deny_dirty(&self) -> Option<bool> {
        self.project
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .deploy
            .as_ref()
            .and_then(|d| d.deny_dirty)
    }

    /// Check if the current project id has been loaded.
    pub fn project_id_found(&self) -> bool {
        self.project_internal
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .id
            .is_some()
    }

    /// Get the current project id.
    ///
    /// # Panics
    /// Panics if the internal project configuration has not been loaded.
    pub fn project_id(&self) -> &str {
        self.project_internal
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .id
            .as_ref()
            .unwrap()
            .as_str()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{args::ProjectArgs, config::RequestContext};

    use super::{Config, LocalConfigManager, ProjectConfig};

    fn path_from_workspace_root(path: &str) -> PathBuf {
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("..")
            .join(path)
    }

    fn unwrap_project_name(config: &Config<LocalConfigManager, ProjectConfig>) -> String {
        config.as_ref().unwrap().name.as_ref().unwrap().to_string()
    }

    #[test]
    fn get_local_config_finds_name_in_cargo_toml() {
        let project_args = ProjectArgs {
            working_directory: path_from_workspace_root("examples/axum/hello-world/"),
            name_or_id: None,
        };

        let local_config = RequestContext::get_local_config(&project_args).unwrap();

        assert_eq!(unwrap_project_name(&local_config), "hello-world");
    }

    #[test]
    fn get_local_config_finds_name_from_workspace_dir() {
        let project_args = ProjectArgs {
            working_directory: path_from_workspace_root("examples/rocket/workspace/hello-world/"),
            name_or_id: None,
        };

        let local_config = RequestContext::get_local_config(&project_args).unwrap();

        assert_eq!(unwrap_project_name(&local_config), "workspace");
    }

    #[test]
    fn setting_name_overrides_name_in_config() {
        let project_args = ProjectArgs {
            working_directory: path_from_workspace_root("examples/axum/hello-world/"),
            name_or_id: Some("my-fancy-project-name".to_owned()),
        };

        let local_config = RequestContext::get_local_config(&project_args).unwrap();

        assert_eq!(unwrap_project_name(&local_config), "my-fancy-project-name");
    }
}
