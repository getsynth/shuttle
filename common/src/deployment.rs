use std::{path::PathBuf, str::FromStr};

use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Display, Serialize, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[strum(ascii_case_insensitive)]
pub enum State {
    Queued,
    Building,
    Built,
    Loading,
    Running,
    Completed,
    Stopped,
    Crashed,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Display, Serialize, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[strum(ascii_case_insensitive)]
#[typeshare::typeshare]
pub enum DeploymentStateBeta {
    Pending,
    Building,
    Running,
    #[strum(serialize = "in progress")]
    InProgress,
    Stopped,
    Stopping,
    Failed,
    /// Fallback
    Unknown,
}

impl DeploymentStateBeta {
    /// We return a &str rather than a Color here, since `comfy-table` re-exports
    /// crossterm::style::Color and we depend on both `comfy-table` and `crossterm`
    /// we may end up with two different versions of Color.
    pub fn get_color(&self) -> &str {
        match self {
            Self::Pending => "dark_yellow",
            Self::Building => "yellow",
            Self::InProgress => "cyan",
            Self::Running => "green",
            Self::Stopped => "dark_blue",
            Self::Stopping => "blue",
            Self::Failed => "red",
            Self::Unknown => "grey",
        }
    }
    pub fn to_string_colored(&self) -> String {
        // Unwrap is safe because Color::from_str returns the color white if the argument is not a Color.
        self.to_string()
            .with(crossterm::style::Color::from_str(self.get_color()).unwrap())
            .to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetadata {
    pub env: Environment,
    pub project_name: String,
    /// Path to a folder that persists between deployments
    pub storage_path: PathBuf,
}

/// The environment this project is running in
#[derive(
    Clone, Copy, Debug, Default, Display, EnumString, PartialEq, Eq, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Environment {
    #[default]
    Local,
    #[strum(serialize = "production")] // Keep this around for a while for backward compat
    Deployment,
}

pub const DEPLOYER_END_MSG_STARTUP_ERR: &str = "Service startup encountered an error";
pub const DEPLOYER_END_MSG_BUILD_ERR: &str = "Service build encountered an error";
pub const DEPLOYER_END_MSG_CRASHED: &str = "Service encountered an error and crashed";
pub const DEPLOYER_END_MSG_STOPPED: &str = "Service was stopped by the user"; // don't include this in end messages so that logs are not stopped too early
pub const DEPLOYER_END_MSG_COMPLETED: &str = "Service finished running all on its own";
// There is a typo in this message, but it is matched on in cargo-shuttle, so we leave it for now.
pub const DEPLOYER_RUNTIME_START_RESPONSE: &str = "Runtime started successully";
pub const DEPLOYER_RUNTIME_START_FAILED: &str = "Runtime did not start successfully";

pub const DEPLOYER_END_MESSAGES_BAD: &[&str] = &[
    DEPLOYER_END_MSG_STARTUP_ERR,
    DEPLOYER_END_MSG_BUILD_ERR,
    DEPLOYER_END_MSG_CRASHED,
];
pub const DEPLOYER_END_MESSAGES_GOOD: &[&str] =
    &[DEPLOYER_END_MSG_COMPLETED, DEPLOYER_RUNTIME_START_RESPONSE];

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_state_deser() {
        assert_eq!(State::Queued, State::from_str("Queued").unwrap());
        assert_eq!(State::Unknown, State::from_str("unKnown").unwrap());
        assert_eq!(State::Built, State::from_str("built").unwrap());
    }

    #[test]
    fn test_env_deser() {
        assert_eq!(Environment::Local, Environment::from_str("local").unwrap());
        assert_eq!(
            Environment::Deployment,
            Environment::from_str("production").unwrap()
        );
        assert!(Environment::from_str("somewhere_else").is_err());
        assert_eq!(format!("{:?}", Environment::Local), "Local".to_owned());
        assert_eq!(format!("{}", Environment::Local), "local".to_owned());
        assert_eq!(Environment::Local.to_string(), "local".to_owned());
    }
}
