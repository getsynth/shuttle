use std::fmt::Formatter;
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Extension, FromRequest, Path, RequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};

use crate::service::GatewayService;
use crate::{AccountName, Error, ErrorKind, ProjectName};

#[derive(Clone, Debug, sqlx::Type, PartialEq, Hash, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct Key(String);

impl Key {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[async_trait]
impl<B> FromRequest<B> for Key
where
    B: Send,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        TypedHeader::<Authorization<Bearer>>::from_request(req)
            .await
            .map_err(|_| Error::from(ErrorKind::KeyMissing))
            .and_then(|TypedHeader(Authorization(bearer))| bearer.token().trim().parse())
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Key {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Key {
    pub fn new_random() -> Self {
        Self(Alphanumeric.sample_string(&mut rand::thread_rng(), 16))
    }
}

const FALSE: fn() -> bool = || false;

/// A wrapper for a guard that verifies an API key is associated with a
/// valid user.
///
/// The `FromRequest` impl consumes the API key and verifies it is valid for the
/// a user. Generally you want to use [`ScopedUser`] instead to ensure the request
/// is valid against the user's owned resources.
#[derive(Clone, Deserialize, PartialEq, Eq, Serialize, Debug)]
pub struct User {
    pub name: AccountName,
    pub key: Key,
    pub projects: Vec<ProjectName>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default = "FALSE")]
    pub super_user: bool,
}

#[async_trait]
impl<B> FromRequest<B> for User
where
    B: Send,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let key = Key::from_request(req).await?;
        let Extension(service) = Extension::<Arc<GatewayService>>::from_request(req)
            .await
            .unwrap();
        let user = service
            .user_from_key(key)
            .await
            // Absord any error into `Unauthorized`
            .map_err(|e| Error::source(ErrorKind::Unauthorized, e))?;
        Ok(user)
    }
}

impl From<User> for shuttle_common::user::Response {
    fn from(user: User) -> Self {
        Self {
            name: user.name.to_string(),
            key: user.key.to_string(),
            projects: user
                .projects
                .into_iter()
                .map(|name| name.to_string())
                .collect(),
        }
    }
}

/// A wrapper for a guard that validates a user's API key *and*
/// scopes the request to a project they own.
///
/// It is guaranteed that [`ScopedUser::scope`] exists and is owned
/// by [`ScopedUser::name`].
pub struct ScopedUser {
    pub user: User,
    pub scope: ProjectName,
}

#[async_trait]
impl<B> FromRequest<B> for ScopedUser
where
    B: Send,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let user = User::from_request(req).await?;
        let scope = match Path::<ProjectName>::from_request(req).await {
            Ok(Path(p)) => p,
            Err(_) => Path::<(ProjectName, String)>::from_request(req)
                .await
                .map(|Path((p, _))| p)
                .unwrap(),
        };
        if user.projects.is_empty() {
            Err(Error::from(ErrorKind::ProjectNotFound))
        } else if user.super_user || user.projects.contains(&scope) {
            Ok(Self { user, scope })
        } else {
            Err(Error::from(ErrorKind::Forbidden))
        }
    }
}

pub struct Admin {
    pub user: User,
}

#[async_trait]
impl<B> FromRequest<B> for Admin
where
    B: Send,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let user = User::from_request(req).await?;
        if user.super_user {
            Ok(Self { user })
        } else {
            Err(Error::from(ErrorKind::Forbidden))
        }
    }
}
