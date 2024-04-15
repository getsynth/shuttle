#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/shuttle-hq/shuttle/main/assets/logo-square-transparent.png",
    html_favicon_url = "https://raw.githubusercontent.com/shuttle-hq/shuttle/main/assets/favicon.ico"
)]

// Public API
pub use shuttle_codegen::main;
pub use shuttle_service::{
    CustomError, DbInput, DeploymentMetadata, Environment, Error, IntoResource, ResourceFactory,
    ResourceInputBuilder, SecretStore, Service,
};

// Useful re-exports
pub use async_trait::async_trait;
pub use tokio;

mod alpha;
mod args;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
fn print_version() {
    println!("{} {}", crate::NAME, crate::VERSION);
}

// Not part of public API
#[doc(hidden)]
pub mod __internals {
    // Internals used by the codegen
    pub use crate::alpha::{start, Alpha};

    // Dependencies required by the codegen
    pub use anyhow::Context;
    #[cfg(feature = "setup-tracing")]
    pub use colored;
    pub use serde_json;
    pub use strfmt::strfmt;
    #[cfg(feature = "setup-tracing")]
    pub use tracing_subscriber;
}

pub use plugins::*;
/// Built-in plugins
mod plugins {
    use crate::async_trait;
    use shuttle_service::{
        resource::{ProvisionResourceRequest, ShuttleResourceOutput, Type},
        DeploymentMetadata, Error, ResourceFactory, ResourceInputBuilder, SecretStore,
    };

    /// ## Shuttle Metadata
    ///
    /// Plugin for getting various metadata at runtime.
    ///
    /// ### Usage
    ///
    /// ```rust,ignore
    /// #[shuttle_runtime::main]
    /// async fn main(
    ///     #[shuttle_runtime::Metadata] metadata: DeploymentMetadata,
    /// ) -> __ { ... }
    #[derive(Default)]
    pub struct Metadata;

    #[async_trait]
    impl ResourceInputBuilder for Metadata {
        type Input = DeploymentMetadata;
        type Output = DeploymentMetadata;

        async fn build(self, factory: &ResourceFactory) -> Result<Self::Input, Error> {
            Ok(factory.get_metadata())
        }
    }

    /// ## Shuttle Secrets
    ///
    /// Plugin for getting secrets in your [Shuttle](https://www.shuttle.rs) service.
    ///
    /// ### Usage
    ///
    /// Add a `Secrets.toml` file to the root of your crate with the secrets you'd like to store.
    /// Make sure to add `Secrets*.toml` to `.gitignore` to omit your secrets from version control.
    ///
    /// Next, add `#[shuttle_runtime::Secrets] secrets: SecretStore` as a parameter to your `shuttle_service::main` function.
    /// `SecretStore::get` can now be called to retrieve your API keys and other secrets at runtime.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// #[shuttle_runtime::main]
    /// async fn main(
    ///     #[shuttle_runtime::Secrets] secrets: SecretStore
    /// ) -> ShuttleAxum {
    ///     // get secret defined in `Secrets.toml` file.
    ///     let secret = secrets.get("MY_API_KEY").unwrap();
    ///
    ///     let router = Router::new()
    ///         .route("/", || async move { format!("My secret is: {}", secret) });
    ///
    ///     Ok(router.into())
    /// }
    /// ```
    #[derive(Default)]
    pub struct Secrets;

    #[async_trait]
    impl ResourceInputBuilder for Secrets {
        type Input = ProvisionResourceRequest;
        type Output = ShuttleResourceOutput<SecretStore>;

        async fn build(self, _factory: &ResourceFactory) -> Result<Self::Input, Error> {
            Ok(ProvisionResourceRequest::new(
                Type::Secrets,
                serde_json::Value::Null,
                serde_json::Value::Null,
            ))
        }
    }
}
