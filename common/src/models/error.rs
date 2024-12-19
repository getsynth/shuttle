use std::fmt::{Display, Formatter};

use crossterm::style::Stylize;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, warn};

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        warn!("{}", self.message);

        (self.status(), axum::Json(self)).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[typeshare::typeshare]
pub struct ApiError {
    pub message: String,
    pub status_code: u16,
}

impl ApiError {
    pub fn internal(message: &str) -> Self {
        Self {
            message: message.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        }
    }

    /// Creates an internal error without exposing sensitive information to the user.
    #[inline(always)]
    pub fn internal_safe<E>(message: &str, error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        error!(error = &error as &dyn std::error::Error, "{message}");

        // Return the raw error during debug builds
        #[cfg(debug_assertions)]
        {
            ApiError::internal(&error.to_string())
        }
        // Return the safe message during release builds
        #[cfg(not(debug_assertions))]
        {
            ApiError::internal(message)
        }
    }

    pub fn unavailable(error: impl std::error::Error) -> Self {
        Self {
            message: error.to_string(),
            status_code: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
        }
    }

    pub fn bad_request(error: impl std::error::Error) -> Self {
        Self {
            message: error.to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            message: "Unauthorized".to_string(),
            status_code: StatusCode::UNAUTHORIZED.as_u16(),
        }
    }

    pub fn forbidden() -> Self {
        Self {
            message: "Forbidden".to_string(),
            status_code: StatusCode::FORBIDDEN.as_u16(),
        }
    }

    pub fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub trait ErrorContext<T> {
    /// Make a new internal server error with the given message.
    #[inline(always)]
    fn context_internal_error(self, message: &str) -> Result<T, ApiError>
    where
        Self: Sized,
    {
        self.with_context_internal_error(move || message.to_string())
    }

    /// Make a new internal server error using the given function to create the message.
    fn with_context_internal_error(self, message: impl FnOnce() -> String) -> Result<T, ApiError>;

    /// Make a new bad request error with the given message.
    #[inline(always)]
    fn context_bad_request(self, message: &str) -> Result<T, ApiError>
    where
        Self: Sized,
    {
        self.with_context_bad_request(move || message.to_string())
    }

    /// Make a new bad request error using the given function to create the message.
    fn with_context_bad_request(self, message: impl FnOnce() -> String) -> Result<T, ApiError>;

    /// Make a new not found error with the given message.
    #[inline(always)]
    fn context_not_found(self, message: &str) -> Result<T, ApiError>
    where
        Self: Sized,
    {
        self.with_context_not_found(move || message.to_string())
    }

    /// Make a new not found error using the given function to create the message.
    fn with_context_not_found(self, message: impl FnOnce() -> String) -> Result<T, ApiError>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: std::error::Error + 'static,
{
    #[inline(always)]
    fn with_context_internal_error(self, message: impl FnOnce() -> String) -> Result<T, ApiError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(ApiError::internal_safe(message().as_ref(), error)),
        }
    }

    #[inline(always)]
    fn with_context_bad_request(self, message: impl FnOnce() -> String) -> Result<T, ApiError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err({
                let message = message();
                warn!(
                    error = &error as &dyn std::error::Error,
                    "bad request: {message}"
                );

                ApiError {
                    message,
                    status_code: StatusCode::BAD_REQUEST.as_u16(),
                }
            }),
        }
    }

    #[inline(always)]
    fn with_context_not_found(self, message: impl FnOnce() -> String) -> Result<T, ApiError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err({
                let message = message();
                warn!(
                    error = &error as &dyn std::error::Error,
                    "not found: {message}"
                );

                ApiError {
                    message,
                    status_code: StatusCode::NOT_FOUND.as_u16(),
                }
            }),
        }
    }
}

impl<T> ErrorContext<T> for Option<T> {
    #[inline]
    fn with_context_internal_error(self, message: impl FnOnce() -> String) -> Result<T, ApiError> {
        match self {
            Some(value) => Ok(value),
            None => Err(ApiError::internal(message().as_ref())),
        }
    }

    #[inline]
    fn with_context_bad_request(self, message: impl FnOnce() -> String) -> Result<T, ApiError> {
        match self {
            Some(value) => Ok(value),
            None => Err({
                ApiError {
                    message: message(),
                    status_code: StatusCode::BAD_REQUEST.as_u16(),
                }
            }),
        }
    }

    #[inline]
    fn with_context_not_found(self, message: impl FnOnce() -> String) -> Result<T, ApiError> {
        match self {
            Some(value) => Ok(value),
            None => Err({
                ApiError {
                    message: message(),
                    status_code: StatusCode::NOT_FOUND.as_u16(),
                }
            }),
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nMessage: {}",
            self.status().to_string().bold(),
            self.message.to_string().red()
        )
    }
}

impl std::error::Error for ApiError {}

// Note: The string "Invalid project name" is used by cargo-shuttle to determine what type of error was returned.
// Changing it is breaking.
#[derive(Debug, Clone, PartialEq, Error)]
#[error(
    "Invalid project name. Project names must:
    1. only contain lowercase alphanumeric characters or dashes `-`.
    2. not start or end with a dash.
    3. not be empty.
    4. be shorter than 64 characters.
    5. not contain any profanities.
    6. not be a reserved word."
)]
pub struct InvalidProjectName;

impl From<InvalidProjectName> for ApiError {
    fn from(err: InvalidProjectName) -> Self {
        Self::bad_request(err)
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
#[error("Invalid team name. Must not be more than 30 characters long.")]
pub struct InvalidTeamName;

impl From<InvalidTeamName> for ApiError {
    fn from(err: InvalidTeamName) -> Self {
        Self::bad_request(err)
    }
}

#[derive(Debug, Error)]
#[error("Project is not ready. Try to restart it")]
pub struct ProjectNotReady;

#[derive(Debug, Error)]
#[error("Project is running but is not responding correctly. Try to restart it")]
pub struct ProjectUnavailable;

#[derive(Debug, Error)]
#[error("Project '{0}' not found. Make sure you are the owner of this project. Run the `project start` command to create a new project.")]
pub struct ProjectNotFound(pub String);

impl From<ProjectNotFound> for ApiError {
    fn from(err: ProjectNotFound) -> Self {
        Self {
            message: err.to_string(),
            status_code: StatusCode::NOT_FOUND.as_u16(),
        }
    }
}

#[derive(Debug, Error)]
#[error("{0} See https://docs.shuttle.dev/platform-update/platform-update for more information and steps to upgrade.")]
pub struct Deprecated(pub String);

impl From<Deprecated> for ApiError {
    fn from(err: Deprecated) -> Self {
        Self {
            message: err.to_string(),
            status_code: StatusCode::FORBIDDEN.as_u16(),
        }
    }
}

#[derive(Debug, Error)]
#[error("Could not automatically delete the following resources: {0:?}. Please reach out to Shuttle support for help.")]
pub struct ProjectHasResources(pub Vec<String>);

impl From<ProjectHasResources> for ApiError {
    fn from(err: ProjectHasResources) -> Self {
        Self::bad_request(err)
    }
}

#[derive(Debug, Error)]
#[error("Could not automatically stop the running deployment for the project. Please reach out to Shuttle support for help.")]
pub struct ProjectHasRunningDeployment;

impl From<ProjectHasRunningDeployment> for ApiError {
    fn from(err: ProjectHasRunningDeployment) -> Self {
        Self::bad_request(err)
    }
}

#[derive(Debug, Error)]
#[error("Project currently has a deployment that is busy building. Use `cargo shuttle deployment list` to see it and wait for it to finish")]
pub struct ProjectHasBuildingDeployment;

impl From<ProjectHasBuildingDeployment> for ApiError {
    fn from(err: ProjectHasBuildingDeployment) -> Self {
        Self::bad_request(err)
    }
}

#[derive(Debug, Error)]
#[error("Tried to get project into a ready state for deletion but failed. Please reach out to Shuttle support for help.")]
pub struct ProjectCorrupted;

impl From<ProjectCorrupted> for ApiError {
    fn from(err: ProjectCorrupted) -> Self {
        Self::bad_request(err)
    }
}

#[derive(Debug, Error)]
#[error("Invalid custom domain")]
pub struct InvalidCustomDomain;

impl From<InvalidCustomDomain> for ApiError {
    fn from(err: InvalidCustomDomain) -> Self {
        Self::bad_request(err)
    }
}
