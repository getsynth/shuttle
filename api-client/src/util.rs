use anyhow::{Context, Result};
use async_trait::async_trait;
use http::StatusCode;
use serde::de::DeserializeOwned;
use shuttle_common::models::error::ApiError;

/// A to_json wrapper for handling our error states
#[async_trait]
pub trait ToJson {
    async fn to_json<T: DeserializeOwned>(self) -> Result<T>;
}

#[async_trait]
impl ToJson for reqwest::Response {
    async fn to_json<T: DeserializeOwned>(self) -> Result<T> {
        let status_code = self.status();
        let bytes = self.bytes().await?;
        let string = String::from_utf8(bytes.to_vec())
            .unwrap_or_else(|_| format!("[{} bytes]", bytes.len()));

        #[cfg(feature = "tracing")]
        tracing::trace!(response = %string, "Parsing response as JSON");

        if matches!(
            status_code,
            StatusCode::OK | StatusCode::SWITCHING_PROTOCOLS
        ) {
            serde_json::from_str(&string).context("failed to parse a successful response")
        } else {
            #[cfg(feature = "tracing")]
            tracing::trace!("Parsing response as API error");

            let res: ApiError = match serde_json::from_str(&string) {
                Ok(res) => res,
                _ => ApiError {
                    message: format!("Failed to parse response from the server:\n{}", string),
                    status_code: status_code.as_u16(),
                },
            };

            Err(res.into())
        }
    }
}
