use std::error::Error as StdError;

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use serde::{ser::SerializeMap, Serialize};
use shuttle_common::models::error::ApiError;
use tracing::error;
use utoipa::ToSchema;

#[derive(thiserror::Error, Debug, ToSchema)]
pub enum Error {
    #[error("Streaming error: {0}")]
    Streaming(#[from] axum::Error),
    #[error("Custom error: {0}")]
    Custom(#[from] anyhow::Error),
    #[error("Builder error: {0}")]
    Builder(#[from] crate::builder::error::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &format!("{:?}", self))?;
        // use the error source if available, if not use display implementation
        map.serialize_entry("msg", &self.source().unwrap_or(self).to_string())?;
        map.end()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!(error = &self as &dyn std::error::Error, "request error");

        let code = StatusCode::INTERNAL_SERVER_ERROR;

        (
            code,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            )],
            Json(ApiError {
                message: self.to_string(),
                status_code: code.as_u16(),
            }),
        )
            .into_response()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
