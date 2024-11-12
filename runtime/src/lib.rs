#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/shuttle-hq/shuttle/main/assets/logo-square-transparent.png",
    html_favicon_url = "https://raw.githubusercontent.com/shuttle-hq/shuttle/main/assets/favicon.ico"
)]

/// shuttle.rs runtime
mod alpha;
/// Built-in plugins
mod plugins;
/// shuttle.dev runtime
mod rt;
mod start;

// Public API
pub use plugins::{Metadata, Secrets};
pub use shuttle_codegen::main;
pub use shuttle_service::{
    CustomError, DbInput, DeploymentMetadata, Environment, Error, IntoResource, ResourceFactory,
    ResourceInputBuilder, SecretStore, Service,
};

// Useful re-exports
pub use async_trait::async_trait;
pub use tokio;

const VERSION_STRING: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

// Not part of public API
#[doc(hidden)]
pub mod __internals {
    // Internals used by the codegen
    pub use crate::start::start;

    // Dependencies required by the codegen
    pub use anyhow::Context;
    pub use serde_json;
    pub use strfmt::strfmt;

    use super::*;
    use std::future::Future;
    #[async_trait]
    pub trait Loader {
        async fn load(self, factory: ResourceFactory) -> Result<Vec<Vec<u8>>, Error>;
    }

    #[async_trait]
    impl<F, O> Loader for F
    where
        F: FnOnce(ResourceFactory) -> O + Send,
        O: Future<Output = Result<Vec<Vec<u8>>, Error>> + Send,
    {
        async fn load(self, factory: ResourceFactory) -> Result<Vec<Vec<u8>>, Error> {
            (self)(factory).await
        }
    }

    #[async_trait]
    pub trait Runner {
        type Service: Service;

        async fn run(self, resources: Vec<Vec<u8>>) -> Result<Self::Service, Error>;
    }

    #[async_trait]
    impl<F, O, S> Runner for F
    where
        F: FnOnce(Vec<Vec<u8>>) -> O + Send,
        O: Future<Output = Result<S, Error>> + Send,
        S: Service,
    {
        type Service = S;

        async fn run(self, resources: Vec<Vec<u8>>) -> Result<Self::Service, Error> {
            (self)(resources).await
        }
    }
}
