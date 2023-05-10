#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RecordRequest {
    #[prost(string, tag = "1")]
    pub project_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub service_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub resources: ::prost::alloc::vec::Vec<record_request::Resource>,
}
/// Nested message and enum types in `RecordRequest`.
pub mod record_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Resource {
        #[prost(string, tag = "1")]
        pub r#type: ::prost::alloc::string::String,
        #[prost(bytes = "vec", tag = "2")]
        pub config: ::prost::alloc::vec::Vec<u8>,
        #[prost(bytes = "vec", tag = "3")]
        pub data: ::prost::alloc::vec::Vec<u8>,
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResultResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProjectResourcesRequest {
    #[prost(string, tag = "1")]
    pub project_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourcesResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub resources: ::prost::alloc::vec::Vec<Resource>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServiceResourcesRequest {
    #[prost(string, tag = "1")]
    pub service_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Resource {
    #[prost(string, tag = "1")]
    pub project_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub service_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "4")]
    pub config: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag = "6")]
    pub is_active: bool,
    #[prost(message, optional, tag = "7")]
    pub created_at: ::core::option::Option<::prost_types::Timestamp>,
}
/// Generated client implementations.
pub mod resource_recorder_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ResourceRecorderClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ResourceRecorderClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ResourceRecorderClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ResourceRecorderClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            ResourceRecorderClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Record a new set of resources
        pub async fn record_resources(
            &mut self,
            request: impl tonic::IntoRequest<super::RecordRequest>,
        ) -> Result<tonic::Response<super::ResultResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/resource_recorder.ResourceRecorder/RecordResources",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Get the resource belonging to a project
        pub async fn get_project_resources(
            &mut self,
            request: impl tonic::IntoRequest<super::ProjectResourcesRequest>,
        ) -> Result<tonic::Response<super::ResourcesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/resource_recorder.ResourceRecorder/GetProjectResources",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Get the resource belonging to a service
        pub async fn get_service_resources(
            &mut self,
            request: impl tonic::IntoRequest<super::ServiceResourcesRequest>,
        ) -> Result<tonic::Response<super::ResourcesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/resource_recorder.ResourceRecorder/GetServiceResources",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Delete a resource
        pub async fn delete_resource(
            &mut self,
            request: impl tonic::IntoRequest<super::Resource>,
        ) -> Result<tonic::Response<super::ResultResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/resource_recorder.ResourceRecorder/DeleteResource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod resource_recorder_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ResourceRecorderServer.
    #[async_trait]
    pub trait ResourceRecorder: Send + Sync + 'static {
        /// Record a new set of resources
        async fn record_resources(
            &self,
            request: tonic::Request<super::RecordRequest>,
        ) -> Result<tonic::Response<super::ResultResponse>, tonic::Status>;
        /// Get the resource belonging to a project
        async fn get_project_resources(
            &self,
            request: tonic::Request<super::ProjectResourcesRequest>,
        ) -> Result<tonic::Response<super::ResourcesResponse>, tonic::Status>;
        /// Get the resource belonging to a service
        async fn get_service_resources(
            &self,
            request: tonic::Request<super::ServiceResourcesRequest>,
        ) -> Result<tonic::Response<super::ResourcesResponse>, tonic::Status>;
        /// Delete a resource
        async fn delete_resource(
            &self,
            request: tonic::Request<super::Resource>,
        ) -> Result<tonic::Response<super::ResultResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ResourceRecorderServer<T: ResourceRecorder> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ResourceRecorder> ResourceRecorderServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ResourceRecorderServer<T>
    where
        T: ResourceRecorder,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/resource_recorder.ResourceRecorder/RecordResources" => {
                    #[allow(non_camel_case_types)]
                    struct RecordResourcesSvc<T: ResourceRecorder>(pub Arc<T>);
                    impl<
                        T: ResourceRecorder,
                    > tonic::server::UnaryService<super::RecordRequest>
                    for RecordResourcesSvc<T> {
                        type Response = super::ResultResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RecordRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).record_resources(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RecordResourcesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/resource_recorder.ResourceRecorder/GetProjectResources" => {
                    #[allow(non_camel_case_types)]
                    struct GetProjectResourcesSvc<T: ResourceRecorder>(pub Arc<T>);
                    impl<
                        T: ResourceRecorder,
                    > tonic::server::UnaryService<super::ProjectResourcesRequest>
                    for GetProjectResourcesSvc<T> {
                        type Response = super::ResourcesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ProjectResourcesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_project_resources(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetProjectResourcesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/resource_recorder.ResourceRecorder/GetServiceResources" => {
                    #[allow(non_camel_case_types)]
                    struct GetServiceResourcesSvc<T: ResourceRecorder>(pub Arc<T>);
                    impl<
                        T: ResourceRecorder,
                    > tonic::server::UnaryService<super::ServiceResourcesRequest>
                    for GetServiceResourcesSvc<T> {
                        type Response = super::ResourcesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ServiceResourcesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_service_resources(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetServiceResourcesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/resource_recorder.ResourceRecorder/DeleteResource" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteResourceSvc<T: ResourceRecorder>(pub Arc<T>);
                    impl<
                        T: ResourceRecorder,
                    > tonic::server::UnaryService<super::Resource>
                    for DeleteResourceSvc<T> {
                        type Response = super::ResultResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Resource>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).delete_resource(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteResourceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: ResourceRecorder> Clone for ResourceRecorderServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ResourceRecorder> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ResourceRecorder> tonic::server::NamedService for ResourceRecorderServer<T> {
        const NAME: &'static str = "resource_recorder.ResourceRecorder";
    }
}
