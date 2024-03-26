use strum::{Display, EnumString};
use uuid::Uuid;

/// States a deployment can be in
#[derive(sqlx::Type, Debug, Default, Display, Clone, Copy, EnumString, PartialEq, Eq)]
#[strum(ascii_case_insensitive)]
pub enum State {
    /// Deployment is queued to be build
    Queued,

    /// Deployment is building, but is not done yet
    Building,

    /// Deployment is built, but has not been started yet
    Built,

    /// Deployment is being loaded and resources are provisioned
    Loading,

    /// Deployment is running - ie. its thread is active
    Running,

    /// Deployment was running, but stopped running all by itself. This is expected for things like background workers
    Completed,

    /// Deployment was running, but has been stopped by the user.
    Stopped,

    /// Something in the deployment process failed
    Crashed,

    /// We never expect this state and entering this state should be considered a bug
    #[default]
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DeploymentState {
    pub id: Uuid,
    pub state: State,
}

impl From<State> for shuttle_common::deployment::State {
    fn from(state: State) -> Self {
        match state {
            State::Queued => Self::Queued,
            State::Building => Self::Building,
            State::Built => Self::Built,
            State::Loading => Self::Loading,
            State::Running => Self::Running,
            State::Completed => Self::Completed,
            State::Stopped => Self::Stopped,
            State::Crashed => Self::Crashed,
            State::Unknown => Self::Unknown,
        }
    }
}

impl From<shuttle_common::deployment::State> for State {
    fn from(state: shuttle_common::deployment::State) -> Self {
        match state {
            shuttle_common::deployment::State::Queued => Self::Queued,
            shuttle_common::deployment::State::Building => Self::Building,
            shuttle_common::deployment::State::Built => Self::Built,
            shuttle_common::deployment::State::Loading => Self::Loading,
            shuttle_common::deployment::State::Running => Self::Running,
            shuttle_common::deployment::State::Completed => Self::Completed,
            shuttle_common::deployment::State::Stopped => Self::Stopped,
            shuttle_common::deployment::State::Crashed => Self::Crashed,
            shuttle_common::deployment::State::Unknown => Self::Unknown,
        }
    }
}

/// Records state logs for the deployment progress
pub trait StateRecorder: Clone + Send + Sync + 'static {
    type Err: std::error::Error + Send;

    /// Takes a state and send it on to the async thread that records it.
    fn record_state(&self, log: DeploymentState) -> Result<(), Self::Err>;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::persistence::State;

    #[test]
    fn test_state_deser() {
        assert_eq!(State::Building, State::from_str("builDing").unwrap());
        assert_eq!(State::Queued, State::from_str("queued").unwrap());
        assert_eq!(State::Stopped, State::from_str("Stopped").unwrap());
    }
}
