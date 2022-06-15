use super::{Built, DeploymentState, Queued};

#[derive(sqlx::FromRow, Debug)]
pub struct DeploymentInfo {
    pub name: String,
    pub state: DeploymentState,
}

impl From<&Queued> for DeploymentInfo {
    fn from(q: &Queued) -> Self {
        DeploymentInfo {
            name: q.name.clone(),
            state: q.state,
        }
    }
}

impl From<&Built> for DeploymentInfo {
    fn from(b: &Built) -> Self {
        DeploymentInfo {
            name: b.name.clone(),
            state: b.state,
        }
    }
}
