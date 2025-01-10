#![doc = include_str!("../README.md")]
use shuttle_runtime::{CustomError, Error};
use std::net::SocketAddr;

#[cfg(feature = "axum")]
use axum::Router;
#[cfg(feature = "axum-0-7")]
use axum_0_7::Router;

/// A wrapper type for [axum::Router] so we can implement [shuttle_runtime::Service] for it.
pub struct AxumService(pub Router);

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for AxumService {
    /// Takes the router that is returned by the user in their [shuttle_runtime::main] function
    /// and binds to an address passed in by shuttle.
    async fn bind(mut self, addr: SocketAddr) -> Result<(), Error> {
        #[cfg(feature = "axum")]
        axum::serve(
            shuttle_runtime::tokio::net::TcpListener::bind(addr)
                .await
                .map_err(CustomError::new)?,
            self.0,
        )
        .await
        .map_err(CustomError::new)?;
        #[cfg(feature = "axum-0-7")]
        axum_0_7::serve(
            shuttle_runtime::tokio::net::TcpListener::bind(addr)
                .await
                .map_err(CustomError::new)?,
            self.0,
        )
        .await
        .map_err(CustomError::new)?;

        Ok(())
    }
}

impl From<Router> for AxumService {
    fn from(router: Router) -> Self {
        Self(router)
    }
}

#[doc = include_str!("../README.md")]
pub type ShuttleAxum = Result<AxumService, Error>;
