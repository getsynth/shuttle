use anyhow::Result;
use shuttle_api_client::ShuttleApiClient;
use shuttle_common::models::{admin::ProjectResponse, stats};

pub struct Client {
    pub inner: ShuttleApiClient,
}

impl Client {
    pub fn new(api_url: String, api_key: String) -> Self {
        Self {
            inner: ShuttleApiClient::new(api_url, Some(api_key), None),
        }
    }

    pub async fn revive(&self) -> Result<String> {
        self.inner
            .post_json("/admin/revive", Option::<()>::None)
            .await
    }

    pub async fn destroy(&self) -> Result<String> {
        self.inner
            .post_json("/admin/destroy", Option::<()>::None)
            .await
    }

    pub async fn idle_cch(&self) -> Result<()> {
        self.inner
            .post("/admin/idle-cch", Option::<()>::None)
            .await?;

        Ok(())
    }

    pub async fn acme_account_create(
        &self,
        email: &str,
        acme_server: Option<String>,
    ) -> Result<serde_json::Value> {
        let path = format!("/admin/acme/{email}");
        self.inner.post_json(&path, Some(acme_server)).await
    }

    pub async fn acme_request_certificate(
        &self,
        fqdn: &str,
        project_name: &str,
        credentials: &serde_json::Value,
    ) -> Result<String> {
        let path = format!("/admin/acme/request/{project_name}/{fqdn}");
        self.inner.post_json(&path, Some(credentials)).await
    }

    pub async fn acme_renew_custom_domain_certificate(
        &self,
        fqdn: &str,
        project_name: &str,
        credentials: &serde_json::Value,
    ) -> Result<String> {
        let path = format!("/admin/acme/renew/{project_name}/{fqdn}");
        self.inner.post_json(&path, Some(credentials)).await
    }

    pub async fn acme_renew_gateway_certificate(
        &self,
        credentials: &serde_json::Value,
    ) -> Result<String> {
        let path = "/admin/acme/gateway/renew".to_string();
        self.inner.post_json(&path, Some(credentials)).await
    }

    pub async fn get_projects(&self) -> Result<Vec<ProjectResponse>> {
        self.inner.get_json("/admin/projects").await
    }

    pub async fn change_project_owner(&self, project_name: &str, new_user_id: &str) -> Result<()> {
        self.inner
            .get(format!(
                "/admin/projects/change-owner/{project_name}/{new_user_id}"
            ))
            .await?;

        Ok(())
    }

    pub async fn get_load(&self) -> Result<stats::LoadResponse> {
        self.inner.get_json("/admin/stats/load").await
    }

    pub async fn clear_load(&self) -> Result<stats::LoadResponse> {
        self.inner.delete_json("/admin/stats/load").await
    }

    pub async fn set_beta_access(&self, user_id: &str, access: bool) -> Result<()> {
        if access {
            self.inner
                .put(format!("/users/{user_id}/beta"), Option::<()>::None)
                .await?;
        } else {
            self.inner.delete(format!("/users/{user_id}/beta")).await?;
        }

        Ok(())
    }
}
