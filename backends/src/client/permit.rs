use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use async_trait::async_trait;
use http::StatusCode;
use permit_client_rs::{
    apis::{
        relationship_tuples_api::{
            create_relationship_tuple, delete_relationship_tuple, list_relationship_tuples,
        },
        resource_instances_api::{create_resource_instance, delete_resource_instance},
        role_assignments_api::{assign_role, list_role_assignments, unassign_role},
        users_api::{create_user, delete_user, get_user},
        Error as PermitClientError,
    },
    models::{
        RelationshipTupleCreate, RelationshipTupleDelete, ResourceInstanceCreate,
        RoleAssignmentCreate, RoleAssignmentRemove, UserCreate, UserRead,
    },
};
use permit_pdp_client_rs::{
    apis::{
        authorization_api_api::{
            get_user_permissions_user_permissions_post, is_allowed_allowed_post,
        },
        data_updater_api::trigger_policy_data_update_data_updater_trigger_post,
        policy_updater_api::trigger_policy_update_policy_updater_trigger_post,
        Error as PermitPDPClientError,
    },
    models::{AuthorizationQuery, Resource, User, UserPermissionsQuery},
};
use serde::{Deserialize, Serialize};
use shuttle_common::{
    claims::AccountTier,
    models::{error::ApiError, organization, project},
};
use tracing::error;

#[async_trait]
pub trait PermissionsDal {
    // User management

    /// Get a user with the given ID
    async fn get_user(&self, user_id: &str) -> Result<UserRead>;
    /// Delete a user with the given ID
    async fn delete_user(&self, user_id: &str) -> Result<()>;
    /// Create a new user and set their tier correctly
    async fn new_user(&self, user_id: &str) -> Result<UserRead>;
    /// Set a user to be a Pro user
    async fn make_pro(&self, user_id: &str) -> Result<()>;
    /// Set a user to be a Basic user
    async fn make_basic(&self, user_id: &str) -> Result<()>;

    // Project management

    /// Creates a Project resource and assigns the user as admin for that project
    async fn create_project(&self, user_id: &str, project_id: &str) -> Result<()>;
    /// Deletes a Project resource
    async fn delete_project(&self, project_id: &str) -> Result<()>;

    /// Get list of all projects the user has direct permissions for
    async fn get_personal_projects(&self, user_id: &str) -> Result<Vec<String>>;

    // Organization management

    /// Creates an Organization resource and assigns the user as admin for the organization
    async fn create_organization(&self, user_id: &str, org: &Organization) -> Result<()>;

    /// Deletes an Organization resource
    async fn delete_organization(&self, user_id: &str, org_id: &str) -> Result<()>;

    /// Get the details of an organization
    async fn get_organization(&self, user_id: &str, org_id: &str)
        -> Result<organization::Response>;

    /// Get a list of all the organizations a user has access to
    async fn get_organizations(&self, user_id: &str) -> Result<Vec<organization::Response>>;

    /// Get a list of all project IDs that belong to an organization
    async fn get_organization_projects(&self, user_id: &str, org_id: &str) -> Result<Vec<String>>;

    /// Transfers a project from a user to another user
    async fn transfer_project_to_user(
        &self,
        user_id: &str,
        project_id: &str,
        new_user_id: &str,
    ) -> Result<()>;

    /// Transfers a project from a user to an organization
    async fn transfer_project_to_org(
        &self,
        user_id: &str,
        project_id: &str,
        org_id: &str,
    ) -> Result<()>;

    /// Transfers a project from an organization to a user
    async fn transfer_project_from_org(
        &self,
        user_id: &str,
        project_id: &str,
        org_id: &str,
    ) -> Result<()>;

    /// Add a user as a normal member to an organization
    async fn add_organization_member(
        &self,
        admin_user: &str,
        org_id: &str,
        user_id: &str,
    ) -> Result<()>;

    /// Remove a user from an organization
    async fn remove_organization_member(
        &self,
        admin_user: &str,
        org_id: &str,
        user_id: &str,
    ) -> Result<()>;

    /// Get a list of all the members of an organization
    async fn get_organization_members(
        &self,
        user_id: &str,
        org_id: &str,
    ) -> Result<Vec<organization::MemberResponse>>;

    // Permissions queries

    /// Check if user can perform action on this project
    async fn allowed(&self, user_id: &str, project_id: &str, action: &str) -> Result<bool>;

    /// Get the owner of a project
    async fn get_project_owner(&self, user_id: &str, project_id: &str) -> Result<Owner>;
}

/// Simple details of an organization to create
#[derive(Debug, PartialEq)]
pub struct Organization {
    /// Unique identifier for the organization. Should be `org_{ulid}`
    pub id: String,

    /// The name used to display the organization in the UI
    pub display_name: String,
}

#[derive(Deserialize, Serialize)]
/// The attributes stored with each organization resource
struct OrganizationAttributes {
    display_name: String,
}

impl OrganizationAttributes {
    fn new(org: &Organization) -> Self {
        Self {
            display_name: org.display_name.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Owner {
    User(String),
    Organization(String),
}

impl From<Owner> for project::Owner {
    fn from(owner: Owner) -> Self {
        match owner {
            Owner::User(id) => project::Owner::User(id),
            Owner::Organization(id) => project::Owner::Organization(id),
        }
    }
}

/// Wrapper for the Permit.io API and PDP (Policy decision point) API
#[derive(Clone)]
pub struct Client {
    pub api: permit_client_rs::apis::configuration::Configuration,
    pub pdp: permit_pdp_client_rs::apis::configuration::Configuration,
    pub proj_id: String,
    pub env_id: String,
}

impl Client {
    pub fn new(
        api_uri: String,
        pdp_uri: String,
        proj_id: String,
        env_id: String,
        api_key: String,
    ) -> Self {
        Self {
            api: permit_client_rs::apis::configuration::Configuration {
                base_path: api_uri
                    .strip_suffix('/')
                    .map(ToOwned::to_owned)
                    .unwrap_or(api_uri),
                user_agent: None,
                bearer_access_token: Some(api_key.clone()),
                ..Default::default()
            },
            pdp: permit_pdp_client_rs::apis::configuration::Configuration {
                base_path: pdp_uri
                    .strip_suffix('/')
                    .map(ToOwned::to_owned)
                    .unwrap_or(pdp_uri),
                user_agent: None,
                bearer_access_token: Some(api_key),
                ..Default::default()
            },
            proj_id,
            env_id,
        }
    }
}

#[async_trait]
impl PermissionsDal for Client {
    async fn get_user(&self, user_id: &str) -> Result<UserRead> {
        Ok(get_user(&self.api, &self.proj_id, &self.env_id, user_id).await?)
    }

    async fn delete_user(&self, user_id: &str) -> Result<()> {
        Ok(delete_user(&self.api, &self.proj_id, &self.env_id, user_id).await?)
    }

    async fn new_user(&self, user_id: &str) -> Result<UserRead> {
        let user = self.create_user(user_id).await?;
        self.make_basic(&user.id.to_string()).await?;

        self.get_user(&user.id.to_string()).await
    }

    async fn make_pro(&self, user_id: &str) -> Result<()> {
        let user = self.get_user(user_id).await?;

        if user.roles.is_some_and(|roles| {
            roles
                .iter()
                .any(|r| r.role == AccountTier::Basic.to_string())
        }) {
            self.unassign_role(user_id, &AccountTier::Basic).await?;
        }

        self.assign_role(user_id, &AccountTier::Pro).await
    }

    async fn make_basic(&self, user_id: &str) -> Result<()> {
        let user = self.get_user(user_id).await?;

        if user
            .roles
            .is_some_and(|roles| roles.iter().any(|r| r.role == AccountTier::Pro.to_string()))
        {
            self.unassign_role(user_id, &AccountTier::Pro).await?;
        }

        self.assign_role(user_id, &AccountTier::Basic).await
    }

    async fn create_project(&self, user_id: &str, project_id: &str) -> Result<()> {
        if let Err(e) = create_resource_instance(
            &self.api,
            &self.proj_id,
            &self.env_id,
            ResourceInstanceCreate {
                key: project_id.to_owned(),
                tenant: "default".to_owned(),
                resource: "Project".to_owned(),
                attributes: None,
            },
        )
        .await
        {
            // Early return all errors except 409's (project already exists)
            let e: Error = e.into();
            if let Error::ResponseError(ref re) = e {
                if re.status != StatusCode::CONFLICT {
                    return Err(e);
                }
            } else {
                return Err(e);
            }
        }

        self.assign_resource_role(user_id, format!("Project:{project_id}"), "admin")
            .await?;

        Ok(())
    }

    async fn delete_project(&self, project_id: &str) -> Result<()> {
        Ok(delete_resource_instance(
            &self.api,
            &self.proj_id,
            &self.env_id,
            format!("Project:{project_id}").as_str(),
        )
        .await?)
    }

    async fn get_personal_projects(&self, user_id: &str) -> Result<Vec<String>> {
        let projects = list_role_assignments(
            &self.api,
            &self.proj_id,
            &self.env_id,
            Some(user_id),
            Some("admin"),
            Some("default"),
            Some("Project"),
            None,
            None,
            None,
            None,
        )
        .await?
        .into_iter()
        .map(|ra| ra.resource_instance.expect("to have resource instance"))
        .map(|ri| {
            ri.strip_prefix("Project:")
                .expect("ID to start with the 'Project:' prefix")
                .to_string()
        })
        .collect();

        Ok(projects)
    }

    async fn allowed(&self, user_id: &str, project_id: &str, action: &str) -> Result<bool> {
        // NOTE: This API function was modified in upstream to use AuthorizationQuery
        let res = is_allowed_allowed_post(
            &self.pdp,
            AuthorizationQuery {
                user: Box::new(User {
                    key: user_id.to_owned(),
                    ..Default::default()
                }),
                action: action.to_owned(),
                resource: Box::new(Resource {
                    r#type: "Project".to_string(),
                    key: Some(project_id.to_owned()),
                    tenant: Some("default".to_owned()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
        )
        .await?;

        Ok(res.allow.unwrap_or_default())
    }

    async fn create_organization(&self, user_id: &str, org: &Organization) -> Result<()> {
        if !self.allowed_org(user_id, &org.id, "create").await? {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::FORBIDDEN,
                content:
                    "User does not have permission to create organization. Are you a pro user?"
                        .to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        if !self.get_organizations(user_id).await?.is_empty() {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::BAD_REQUEST,
                content: "User already has an organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        if let Err(e) = create_resource_instance(
            &self.api,
            &self.proj_id,
            &self.env_id,
            ResourceInstanceCreate {
                key: org.id.to_owned(),
                tenant: "default".to_owned(),
                resource: "Organization".to_owned(),
                attributes: serde_json::to_value(OrganizationAttributes::new(org)).ok(),
            },
        )
        .await
        {
            // Early return all errors except 409's (project already exists)
            let e: Error = e.into();
            if let Error::ResponseError(ref re) = e {
                if re.status != StatusCode::CONFLICT {
                    return Err(e);
                }
            } else {
                return Err(e);
            }
        }

        self.assign_resource_role(user_id, format!("Organization:{}", org.id), "admin")
            .await?;

        Ok(())
    }

    async fn delete_organization(&self, user_id: &str, org_id: &str) -> Result<()> {
        if !self.allowed_org(user_id, org_id, "manage").await? {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::FORBIDDEN,
                content: "User does not have permission to delete the organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        let projects = self.get_organization_projects(user_id, org_id).await?;

        if !projects.is_empty() {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::BAD_REQUEST,
                content: "Organization still has projects".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        Ok(delete_resource_instance(
            &self.api,
            &self.proj_id,
            &self.env_id,
            format!("Organization:{org_id}").as_str(),
        )
        .await?)
    }

    async fn get_organization_projects(&self, user_id: &str, org_id: &str) -> Result<Vec<String>> {
        if !self.allowed_org(user_id, org_id, "view").await? {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::FORBIDDEN,
                content: "User does not have permission to view the organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        let relationships = list_relationship_tuples(
            &self.api,
            &self.proj_id,
            &self.env_id,
            Some(true),
            None,
            None,
            Some("default"),
            Some(&format!("Organization:{org_id}")),
            Some("parent"),
            None,
            Some("Project"),
            None,
        )
        .await?;

        let projects = relationships
            .into_iter()
            .map(|rel| rel.object_details.expect("to have object details").key)
            .collect();

        Ok(projects)
    }

    async fn get_organization(
        &self,
        user_id: &str,
        org_id: &str,
    ) -> Result<organization::Response> {
        let mut perms = get_user_permissions_user_permissions_post(
            &self.pdp,
            UserPermissionsQuery {
                user: Box::new(User {
                    key: user_id.to_owned(),
                    ..Default::default()
                }),
                resources: Some(vec![format!("Organization:{org_id}")]),
                tenants: Some(vec!["default".to_owned()]),
                ..Default::default()
            },
            None,
            None,
        )
        .await?;

        let Some(org) = perms.remove(&format!("Organization:{org_id}")) else {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::FORBIDDEN,
                content: "User does not have permission to view the organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        };

        let res = if let Some(resource) = org.resource {
            let attributes = resource.attributes.unwrap_or_default();
            let org_attrs = serde_json::from_value::<OrganizationAttributes>(attributes)
                .expect("to read organization attributes");

            organization::Response {
                id: resource.key,
                display_name: org_attrs.display_name,
                is_admin: org.roles.unwrap_or_default().contains(&"admin".to_string()),
            }
        } else {
            unreachable!("the permission will always have a resource")
        };

        Ok(res)
    }

    async fn get_organizations(&self, user_id: &str) -> Result<Vec<organization::Response>> {
        let perms = get_user_permissions_user_permissions_post(
            &self.pdp,
            UserPermissionsQuery {
                user: Box::new(User {
                    key: user_id.to_owned(),
                    ..Default::default()
                }),
                resource_types: Some(vec!["Organization".to_owned()]),
                tenants: Some(vec!["default".to_owned()]),
                ..Default::default()
            },
            None,
            None,
        )
        .await?;

        let mut res = Vec::with_capacity(perms.len());

        for perm in perms.into_values() {
            if let Some(resource) = perm.resource {
                let attributes = resource.attributes.unwrap_or_default();
                let org = serde_json::from_value::<OrganizationAttributes>(attributes)
                    .expect("to read organization attributes");

                res.push(organization::Response {
                    id: resource.key,
                    display_name: org.display_name,
                    is_admin: perm
                        .roles
                        .unwrap_or_default()
                        .contains(&"admin".to_string()),
                });
            }
        }

        Ok(res)
    }

    async fn transfer_project_to_user(
        &self,
        user_id: &str,
        project_id: &str,
        new_user_id: &str,
    ) -> Result<()> {
        self.unassign_resource_role(user_id, format!("Project:{project_id}"), "admin")
            .await?;

        self.assign_resource_role(new_user_id, format!("Project:{project_id}"), "admin")
            .await?;

        Ok(())
    }

    async fn transfer_project_to_org(
        &self,
        user_id: &str,
        project_id: &str,
        org_id: &str,
    ) -> Result<()> {
        if !self.allowed_org(user_id, org_id, "manage").await? {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::FORBIDDEN,
                content: "User does not have permission to modify the organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        self.unassign_resource_role(user_id, format!("Project:{project_id}"), "admin")
            .await?;

        self.assign_relationship(
            format!("Organization:{org_id}"),
            "parent",
            format!("Project:{project_id}"),
        )
        .await?;

        Ok(())
    }

    async fn transfer_project_from_org(
        &self,
        user_id: &str,
        project_id: &str,
        org_id: &str,
    ) -> Result<()> {
        if !self.allowed_org(user_id, org_id, "manage").await? {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::FORBIDDEN,
                content: "User does not have permission to modify the organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        self.assign_resource_role(user_id, format!("Project:{project_id}"), "admin")
            .await?;

        self.unassign_relationship(
            format!("Organization:{org_id}"),
            "parent",
            format!("Project:{project_id}"),
        )
        .await?;

        Ok(())
    }

    async fn add_organization_member(
        &self,
        admin_user: &str,
        org_id: &str,
        user_id: &str,
    ) -> Result<()> {
        if !self.allowed_org(admin_user, org_id, "manage").await? {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::FORBIDDEN,
                content: "User does not have permission to modify the organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        let user = self.get_user(user_id).await?;

        if !user
            .roles
            .is_some_and(|roles| roles.iter().any(|r| r.role == AccountTier::Pro.to_string()))
        {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::BAD_REQUEST,
                content: "Only Pro users can be added to an organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        self.assign_resource_role(user_id, format!("Organization:{org_id}"), "member")
            .await?;

        Ok(())
    }

    async fn remove_organization_member(
        &self,
        admin_user: &str,
        org_id: &str,
        user_id: &str,
    ) -> Result<()> {
        if admin_user == user_id {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::BAD_REQUEST,
                content: "Cannot remove yourself from an organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        if !self.allowed_org(admin_user, org_id, "manage").await? {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::FORBIDDEN,
                content: "User does not have permission to modify the organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        self.unassign_resource_role(user_id, format!("Organization:{org_id}"), "member")
            .await?;

        Ok(())
    }

    async fn get_organization_members(
        &self,
        user_id: &str,
        org_id: &str,
    ) -> Result<Vec<organization::MemberResponse>> {
        if !self.allowed_org(user_id, org_id, "view").await? {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::FORBIDDEN,
                content: "User does not have permission to view the organization".to_owned(),
                entity: "Organization".to_owned(),
            }));
        }

        let assignments = list_role_assignments(
            &self.api,
            &self.proj_id,
            &self.env_id,
            None,
            None,
            Some("default"),
            None,
            Some(&format!("Organization:{org_id}")),
            None,
            None,
            None,
        )
        .await?;

        let members = assignments
            .into_iter()
            .map(|assignment| organization::MemberResponse {
                id: assignment.user,
                role: organization::MemberRole::from_str(&assignment.role)
                    .unwrap_or(organization::MemberRole::Member),
            })
            .collect();

        Ok(members)
    }

    async fn get_project_owner(&self, user_id: &str, project_id: &str) -> Result<Owner> {
        if !self.allowed(user_id, project_id, "view").await? {
            return Err(Error::ResponseError(ResponseContent {
                status: StatusCode::FORBIDDEN,
                content: "User does not have permission to view the project".to_owned(),
                entity: "Project".to_owned(),
            }));
        }

        let relationships = list_relationship_tuples(
            &self.api,
            &self.proj_id,
            &self.env_id,
            Some(true),
            None,
            None,
            Some("default"),
            None,
            Some("parent"),
            Some(&format!("Project:{project_id}")),
            None,
            Some("Organization"),
        )
        .await?;

        if let Some(rel) = relationships.into_iter().next() {
            let org_id = rel.subject_details.expect("to have subject details").key;
            Ok(Owner::Organization(org_id))
        } else {
            // If a user is able to view a project while the project has no parent org, then this user must be the project owner
            Ok(Owner::User(user_id.to_owned()))
        }
    }
}

// Helpers for trait methods
impl Client {
    async fn create_user(&self, user_id: &str) -> Result<UserRead> {
        Ok(create_user(
            &self.api,
            &self.proj_id,
            &self.env_id,
            UserCreate {
                key: user_id.to_owned(),
                ..Default::default()
            },
        )
        .await?)
    }

    async fn assign_role(&self, user_id: &str, role: &AccountTier) -> Result<()> {
        assign_role(
            &self.api,
            &self.proj_id,
            &self.env_id,
            RoleAssignmentCreate {
                role: role.to_string(),
                tenant: Some("default".to_owned()),
                resource_instance: None,
                user: user_id.to_owned(),
            },
        )
        .await?;

        Ok(())
    }

    async fn unassign_role(&self, user_id: &str, role: &AccountTier) -> Result<()> {
        unassign_role(
            &self.api,
            &self.proj_id,
            &self.env_id,
            RoleAssignmentRemove {
                role: role.to_string(),
                tenant: Some("default".to_owned()),
                resource_instance: None,
                user: user_id.to_owned(),
            },
        )
        .await?;

        Ok(())
    }

    async fn assign_resource_role(
        &self,
        user_id: &str,
        resource_instance: String,
        role: &str,
    ) -> Result<()> {
        assign_role(
            &self.api,
            &self.proj_id,
            &self.env_id,
            RoleAssignmentCreate {
                role: role.to_owned(),
                tenant: Some("default".to_owned()),
                resource_instance: Some(resource_instance),
                user: user_id.to_owned(),
            },
        )
        .await?;

        Ok(())
    }

    async fn unassign_resource_role(
        &self,
        user_id: &str,
        resource_instance: String,
        role: &str,
    ) -> Result<()> {
        unassign_role(
            &self.api,
            &self.proj_id,
            &self.env_id,
            RoleAssignmentRemove {
                role: role.to_owned(),
                tenant: Some("default".to_owned()),
                resource_instance: Some(resource_instance),
                user: user_id.to_owned(),
            },
        )
        .await?;

        Ok(())
    }

    async fn allowed_org(&self, user_id: &str, org_id: &str, action: &str) -> Result<bool> {
        // NOTE: This API function was modified in upstream to use AuthorizationQuery
        let res = is_allowed_allowed_post(
            &self.pdp,
            AuthorizationQuery {
                user: Box::new(User {
                    key: user_id.to_owned(),
                    ..Default::default()
                }),
                action: action.to_owned(),
                resource: Box::new(Resource {
                    r#type: "Organization".to_string(),
                    key: Some(org_id.to_owned()),
                    tenant: Some("default".to_owned()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
        )
        .await?;

        Ok(res.allow.unwrap_or_default())
    }

    async fn assign_relationship(&self, subject: String, role: &str, object: String) -> Result<()> {
        create_relationship_tuple(
            &self.api,
            &self.proj_id,
            &self.env_id,
            RelationshipTupleCreate {
                relation: role.to_owned(),
                tenant: Some("default".to_owned()),
                subject,
                object,
            },
        )
        .await?;

        Ok(())
    }

    async fn unassign_relationship(
        &self,
        subject: String,
        role: &str,
        object: String,
    ) -> Result<()> {
        delete_relationship_tuple(
            &self.api,
            &self.proj_id,
            &self.env_id,
            RelationshipTupleDelete {
                relation: role.to_owned(),
                subject,
                object,
            },
        )
        .await?;

        Ok(())
    }

    pub async fn sync_pdp(&self) -> Result<()> {
        trigger_policy_update_policy_updater_trigger_post(&self.pdp).await?;
        trigger_policy_data_update_data_updater_trigger_post(&self.pdp).await?;

        Ok(())
    }
}

/// Higher level management methods. Use with care.
mod admin {
    use permit_client_rs::{
        apis::environments_api::copy_environment,
        models::{
            environment_copy::ConflictStrategy, EnvironmentCopy, EnvironmentCopyScope,
            EnvironmentCopyScopeFilters, EnvironmentCopyTarget,
        },
    };

    use super::*;

    impl Client {
        /// Copy and overwrite a permit env's policies to another env.
        /// Requires a project level API key.
        pub async fn copy_environment(&self, target_env: &str) -> Result<()> {
            copy_environment(
                &self.api,
                &self.proj_id,
                &self.env_id,
                EnvironmentCopy {
                    target_env: Box::new(EnvironmentCopyTarget {
                        existing: Some(target_env.to_owned()),
                        new: None,
                    }),
                    conflict_strategy: Some(ConflictStrategy::Overwrite),
                    scope: Some(Box::new(EnvironmentCopyScope {
                        resources: Some(Box::new(EnvironmentCopyScopeFilters {
                            include: Some(vec!["*".to_owned()]),
                            exclude: None,
                        })),
                        roles: Some(Box::new(EnvironmentCopyScopeFilters {
                            include: Some(vec!["*".to_owned()]),
                            exclude: None,
                        })),
                        user_sets: Some(Box::new(EnvironmentCopyScopeFilters {
                            include: Some(vec!["*".to_owned()]),
                            exclude: None,
                        })),
                        resource_sets: Some(Box::new(EnvironmentCopyScopeFilters {
                            include: Some(vec!["*".to_owned()]),
                            exclude: None,
                        })),
                    })),
                },
            )
            .await?;

            Ok(())
        }
    }
}

/// Dumbed down and unified version of the client's errors to get rid of the genereic <T>
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Reqwest(reqwest::Error),
    #[error("serde error: {0}")]
    Serde(serde_json::Error),
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("response error: {0}")]
    ResponseError(ResponseContent),
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub struct ResponseContent {
    pub status: reqwest::StatusCode,
    pub content: String,
    pub entity: String,
}
impl Display for ResponseContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "status: {}, content: {}, entity: {}",
            self.status, self.content, self.entity
        )
    }
}
impl<T: Debug> From<PermitClientError<T>> for Error {
    fn from(value: PermitClientError<T>) -> Self {
        match value {
            PermitClientError::Reqwest(e) => Self::Reqwest(e),
            PermitClientError::Serde(e) => Self::Serde(e),
            PermitClientError::Io(e) => Self::Io(e),
            PermitClientError::ResponseError(e) => Self::ResponseError(ResponseContent {
                status: e.status,
                content: e.content,
                entity: format!("{:?}", e.entity),
            }),
        }
    }
}
impl<T: Debug> From<PermitPDPClientError<T>> for Error {
    fn from(value: PermitPDPClientError<T>) -> Self {
        match value {
            PermitPDPClientError::Reqwest(e) => Self::Reqwest(e),
            PermitPDPClientError::Serde(e) => Self::Serde(e),
            PermitPDPClientError::Io(e) => Self::Io(e),
            PermitPDPClientError::ResponseError(e) => Self::ResponseError(ResponseContent {
                status: e.status,
                content: e.content,
                entity: format!("{:?}", e.entity),
            }),
        }
    }
}
impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        match error {
            Error::ResponseError(value) => ApiError {
                message: value.content.to_string(),
                status_code: value.status.into(),
            },
            err => {
                error!(
                    error = &err as &dyn std::error::Error,
                    "Internal error while talking to service"
                );
                ApiError {
                    message: "Our server was unable to handle your request. A ticket should be created for us to fix this.".to_string(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR.into(),
                }
            }
        }
    }
}
