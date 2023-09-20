use async_trait::async_trait;
use dal::{Dal, DalError, Resource};
use prost_types::TimestampError;
use shuttle_common::{backends::auth::VerifyClaim, claims::Scope};
use shuttle_proto::resource_recorder::{
    self, resource_recorder_server::ResourceRecorder, ProjectResourcesRequest, RecordRequest,
    ResourceResponse, ResourcesResponse, ResultResponse, ServiceResourcesRequest,
};
use thiserror::Error;
use tonic::{Request, Response, Status};

pub mod args;
mod dal;
mod r#type;

pub use dal::Sqlite;
use tracing::error;
use ulid::DecodeError;

/// A wrapper to capture any error possible with this service
#[derive(Error, Debug)]
pub enum Error {
    #[error("could not decode id: {0}")]
    UlidDecode(#[from] DecodeError),

    #[error("failed to interact with database: {0}")]
    Dal(#[from] DalError),

    #[error("could not parse resource type: {0}")]
    String(String),

    #[error("could not parse timestamp: {0}")]
    Timestamp(#[from] TimestampError),
}

// thiserror is not happy to handle a `#[from] String`
impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

pub struct Service<D> {
    dal: D,
}

impl<D> Service<D>
where
    D: Dal + Send + Sync + 'static,
{
    pub fn new(dal: D) -> Self {
        Self { dal }
    }

    /// Record the addition of a new resource
    async fn add(&self, request: RecordRequest) -> Result<(), Error> {
        self.dal
            .add_resources(
                request.project_id.parse()?,
                request.service_id.parse()?,
                request
                    .resources
                    .into_iter()
                    .map(TryInto::<Resource>::try_into)
                    .collect::<Result<_, _>>()?,
            )
            .await?;

        Ok(())
    }

    /// Get the resources that belong to a project
    async fn project_resources(
        &self,
        project_id: String,
    ) -> Result<Vec<resource_recorder::Resource>, Error> {
        let resources = self.dal.get_project_resources(project_id.parse()?).await?;

        Ok(resources.into_iter().map(Into::into).collect())
    }

    /// Get the resources that belong to a service
    async fn service_resources(
        &self,
        service_id: String,
    ) -> Result<Vec<resource_recorder::Resource>, Error> {
        let resources = self.dal.get_service_resources(service_id.parse()?).await?;

        Ok(resources.into_iter().map(Into::into).collect())
    }

    /// Get a resource
    async fn get_resource(
        &self,
        resource: resource_recorder::Resource,
    ) -> Result<resource_recorder::Resource, Error> {
        let resource = self.dal.get_resource(&resource.try_into()?).await?;

        Ok(resource.into())
    }

    /// Delete a resource
    async fn delete_resource(&self, resource: resource_recorder::Resource) -> Result<(), Error> {
        self.dal.delete_resource(&resource.try_into()?).await?;

        Ok(())
    }
}

#[async_trait]
impl<D> ResourceRecorder for Service<D>
where
    D: Dal + Send + Sync + 'static,
{
    async fn record_resources(
        &self,
        request: Request<RecordRequest>,
    ) -> Result<Response<ResultResponse>, Status> {
        request.verify(Scope::ResourcesWrite)?;

        let request = request.into_inner();
        let result = match self.add(request).await {
            Ok(()) => ResultResponse {
                success: true,
                message: Default::default(),
            },
            Err(e) => ResultResponse {
                success: false,
                message: e.to_string(),
            },
        };

        Ok(Response::new(result))
    }

    async fn get_project_resources(
        &self,
        request: Request<ProjectResourcesRequest>,
    ) -> Result<Response<ResourcesResponse>, Status> {
        request.verify(Scope::Resources)?;

        let request = request.into_inner();
        let result = match self.project_resources(request.project_id).await {
            Ok(resources) => ResourcesResponse {
                success: true,
                message: Default::default(),
                resources,
            },
            Err(e) => ResourcesResponse {
                success: false,
                message: e.to_string(),
                resources: Vec::new(),
            },
        };

        Ok(Response::new(result))
    }

    async fn get_service_resources(
        &self,
        request: Request<ServiceResourcesRequest>,
    ) -> Result<Response<ResourcesResponse>, Status> {
        request.verify(Scope::Resources)?;

        let request = request.into_inner();
        let result = match self.service_resources(request.service_id).await {
            Ok(resources) => ResourcesResponse {
                success: true,
                message: Default::default(),
                resources,
            },
            Err(e) => ResourcesResponse {
                success: false,
                message: e.to_string(),
                resources: Vec::new(),
            },
        };

        Ok(Response::new(result))
    }

    async fn get_resource(
        &self,
        request: Request<resource_recorder::Resource>,
    ) -> Result<Response<ResourceResponse>, Status> {
        request.verify(Scope::Resources)?;

        let request = request.into_inner();
        let result = match self.get_resource(request).await {
            Ok(resource) => ResourceResponse {
                success: true,
                message: Default::default(),
                resource: Some(resource),
            },
            Err(e) => ResourceResponse {
                success: false,
                message: e.to_string(),
                resource: Default::default(),
            },
        };

        Ok(Response::new(result))
    }

    async fn delete_resource(
        &self,
        request: Request<resource_recorder::Resource>,
    ) -> Result<Response<ResultResponse>, Status> {
        request.verify(Scope::ResourcesWrite)?;

        let request = request.into_inner();
        let result = match self.delete_resource(request).await {
            Ok(()) => ResultResponse {
                success: true,
                message: Default::default(),
            },
            Err(e) => ResultResponse {
                success: false,
                message: e.to_string(),
            },
        };

        Ok(Response::new(result))
    }
}
