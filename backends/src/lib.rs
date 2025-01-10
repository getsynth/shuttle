use tracing::instrument;

use shuttle_common::{
    claims::{Claim, Scope},
    models::user::AccountTier,
};

use client::{ProjectsDal, ResourceDal};

pub mod auth;
pub mod axum;
pub mod cache;
pub mod client;
mod future;
pub mod headers;
pub mod key;
pub mod metrics;
mod otlp_tracing_bridge;
pub mod project_name;
pub mod trace;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

#[allow(async_fn_in_trait)]
pub trait ClaimExt {
    /// Verify that the [Claim] has the [Scope::Admin] scope.
    fn is_admin(&self) -> bool;
    fn is_deployer(&self) -> bool;
    /// Verify that the user's current project count is lower than the account limit in [Claim::limits].
    fn can_create_project(&self, current_count: u32) -> bool;
    /// Verify that the user has permission to provision RDS instances.
    async fn can_provision_rds<G: ProjectsDal, R: ResourceDal>(
        &self,
        projects_dal: &G,
        resource_dal: &mut R,
    ) -> Result<bool, client::Error>;
    /// Verify if the claim subject has ownership of a project.
    async fn owns_project<G: ProjectsDal>(
        &self,
        projects_dal: &G,
        project_ident: &str,
    ) -> Result<bool, client::Error>;
}

impl ClaimExt for Claim {
    fn is_admin(&self) -> bool {
        self.scopes.contains(&Scope::Admin)
    }
    fn is_deployer(&self) -> bool {
        self.tier == AccountTier::Deployer
    }

    fn can_create_project(&self, current_count: u32) -> bool {
        self.is_admin() || self.limits.project_limit() > current_count
    }

    #[instrument(skip_all)]
    async fn can_provision_rds<G: ProjectsDal, R: ResourceDal>(
        &self,
        projects_dal: &G,
        resource_dal: &mut R,
    ) -> Result<bool, client::Error> {
        let token = self.token.as_ref().expect("token to be set");

        let projects = projects_dal.get_user_project_ids(token).await?;

        let mut rds_count = 0;

        for project_id in projects {
            rds_count += resource_dal
                .get_project_rds_resources(&project_id, token)
                .await?
                .len();
        }

        Ok(self.limits.rds_quota > (rds_count as u32) || self.is_admin())
    }

    #[instrument(skip_all)]
    async fn owns_project<G: ProjectsDal>(
        &self,
        projects_dal: &G,
        project_ident: &str,
    ) -> Result<bool, client::Error> {
        let token = self.token.as_ref().expect("token to be set");
        projects_dal.head_user_project(token, project_ident).await
    }
}
