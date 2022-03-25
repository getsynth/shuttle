//! Types representing various errors that can occur in the process of building and deploying a service.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Rocket error: {0}")]
    Rocket(#[from] rocket::Error),
    #[error("Custom error: {0}")]
    Custom(#[from] CustomError)
}

pub type CustomError = anyhow::Error;
