//! This is a layer for [tracing] to capture the state transition of deploys
//!
//! The idea is as follow: as a deployment moves through the [super::DeploymentManager] a set of functions will be invoked.
//! These functions are clear markers for the deployment entering a new state so we would want to change the state as soon as entering these functions.
//! But rather than passing a persistence layer around to be able record the state in these functions we can rather use [tracing].
//!
//! This is very similar to Aspect Oriented Programming where we use the annotations from the function to trigger the recording of a new state.
//! This annotation is a [#[instrument]](https://docs.rs/tracing-attributes/latest/tracing_attributes/attr.instrument.html) with an `id` and `state` field as follow:
//! ```no-test
//! #[instrument(fields(id = %built.id, state = %State::Built))]
//! pub async fn new_state_fn(built: Built) {
//!     // Get built ready for starting
//! }
//! ```
//!
//! Here the `id` is extracted from the `built` argument and the `state` is taken from the [State] enum (the special `%` is needed to use the `Display` trait to convert the values to a str).
//!
//! All `debug!()` etc in these functions will be captured by this layer and will be associated with the deployment and the state.
//!
//! **Warning** Don't log out sensitive info in functions with these annotations

use chrono::{DateTime, Utc};
use shuttle_common::{log::InternalLogOrigin, tracing::JsonVisitor};
use std::str::FromStr;
use tracing::{field::Visit, span, warn, Metadata, Subscriber};
use tracing_subscriber::Layer;
use uuid::Uuid;

/// Records logs for the deployment progress
pub trait LogRecorder: Clone + Send + 'static {
    fn record(&self, log: Log);
}

/// An event or state transition log
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Log {
    /// Deployment id
    pub deployment_id: Uuid,

    /// Internal service that produced this log
    pub internal_origin: InternalLogOrigin,

    /// Time log was produced
    pub tx_timestamp: DateTime<Utc>,

    /// The log line
    pub line: String,
}

impl From<Log> for shuttle_common::LogItem {
    fn from(log: Log) -> Self {
        Self {
            id: log.deployment_id,
            internal_origin: log.internal_origin,
            timestamp: log.tx_timestamp,
            line: log.line,
        }
    }
}

/// Tracing subscriber layer which keeps track of a deployment's state
pub struct DeployLayer<R>
where
    R: LogRecorder + Send + Sync,
{
    recorder: R,
    internal_service: InternalLogOrigin,
}

impl<R> DeployLayer<R>
where
    R: LogRecorder + Send + Sync,
{
    pub fn new(recorder: R, internal_service: InternalLogOrigin) -> Self {
        Self {
            recorder,
            internal_service,
        }
    }
}

impl<R, S> Layer<S> for DeployLayer<R>
where
    S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
    R: LogRecorder + Send + Sync + 'static,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        // We only care about events in some state scope
        let scope = if let Some(scope) = ctx.event_scope(event) {
            scope
        } else {
            return;
        };

        // Find the first scope with the scope details containing the current state
        for span in scope.from_root() {
            let extensions = span.extensions();

            if let Some(details) = extensions.get::<ScopeDetails>() {
                let mut visitor = JsonVisitor::default();

                event.record(&mut visitor);
                let metadata = event.metadata();

                self.recorder.record(Log {
                    deployment_id: details.id,
                    internal_origin: self.internal_service,
                    tx_timestamp: Utc::now(),
                    line: "Test".into(),
                });
                break;
            }
        }
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // We only care about spans that change the state
        if !NewServiceVisitor::is_valid(attrs.metadata()) {
            return;
        }

        let mut visitor = NewServiceVisitor::default();

        attrs.record(&mut visitor);

        let details = visitor.details;

        if details.id.is_nil() {
            warn!("scope details does not have a valid id");
            return;
        }

        // Safe to unwrap since this is the `on_new_span` method
        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();
        let metadata = span.metadata();

        self.recorder.record(Log {
            deployment_id: details.id,
            internal_origin: self.internal_service,
            tx_timestamp: Utc::now(),
            line: "Test".into(),
        });

        extensions.insert::<ScopeDetails>(details);
    }
}

use shuttle_proto::logger::logger_client::LoggerClient;
impl LogRecorder
    for LoggerClient<
        shuttle_common::claims::ClaimService<
            shuttle_common::claims::InjectPropagation<tonic::transport::Channel>,
        >,
    >
{
    fn record(&self, log: Log) {
        // TODO: Make async + error handling?
        self.send_logs(request)
            .await
            .expect("Failed to sens log line");
    }
}

/// Used to keep track of the current state a deployment scope is in
#[derive(Debug, Default)]
struct ScopeDetails {
    id: Uuid,
    internal_origin: InternalLogOrigin,
}

/// This visitor is meant to extract the `ScopeDetails` for any scope with `name` and `status` fields
#[derive(Default)]
struct NewServiceVisitor {
    details: ScopeDetails,
}

impl NewServiceVisitor {
    /// Field containing the deployment identifier
    const ID_IDENT: &'static str = "id";

    /// Field containing the service that started the span
    const SERVICE_IDENT: &'static str = "state";

    fn is_valid(metadata: &Metadata) -> bool {
        metadata.is_span()
            && metadata.fields().field(Self::ID_IDENT).is_some()
            && metadata.fields().field(Self::SERVICE_IDENT).is_some()
    }
}

impl Visit for NewServiceVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == Self::SERVICE_IDENT {
            self.details.internal_origin =
                InternalLogOrigin::from_str(&format!("{value:?}")).unwrap_or_default();
        } else if field.name() == Self::ID_IDENT {
            self.details.id = Uuid::try_parse(&format!("{value:?}")).unwrap_or_default();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::read_dir,
        net::{Ipv4Addr, SocketAddr},
        path::PathBuf,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{
        persistence::{DeploymentUpdater, ResourceManager},
        RuntimeManager,
    };
    use async_trait::async_trait;
    use axum::body::Bytes;
    use ctor::ctor;
    use flate2::{write::GzEncoder, Compression};
    use portpicker::pick_unused_port;
    use shuttle_common::claims::{Claim, ClaimLayer, InjectPropagationLayer};
    use shuttle_proto::{
        logger::{logger_client::LoggerClient, Batcher},
        provisioner::{
            provisioner_server::{Provisioner, ProvisionerServer},
            DatabaseDeletionResponse, DatabaseRequest, DatabaseResponse, Ping, Pong,
        },
        resource_recorder::{ResourcesResponse, ResultResponse},
    };
    use tempfile::Builder;
    use tokio::{select, time::sleep};
    use tonic::transport::{Endpoint, Server};
    use tower::ServiceBuilder;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    use ulid::Ulid;
    use uuid::Uuid;

    use crate::{
        deployment::{
            deploy_layer::LogType, gateway_client::BuildQueueClient, ActiveDeploymentsGetter,
            Built, DeploymentManager, Queued,
        },
        persistence::{Secret, SecretGetter, SecretRecorder},
    };

    use super::{DeployLayer, InternalLogOrigin, Log, LogRecorder};

    #[ctor]
    static RECORDER: Arc<Mutex<RecorderMock>> = {
        let recorder = RecorderMock::new();

        // Copied from the test-log crate
        let event_filter = {
            use ::tracing_subscriber::fmt::format::FmtSpan;

            match ::std::env::var("RUST_LOG_SPAN_EVENTS") {
                Ok(value) => {
                    value
                        .to_ascii_lowercase()
                        .split(',')
                        .map(|filter| match filter.trim() {
                            "new" => FmtSpan::NEW,
                            "enter" => FmtSpan::ENTER,
                            "exit" => FmtSpan::EXIT,
                            "close" => FmtSpan::CLOSE,
                            "active" => FmtSpan::ACTIVE,
                            "full" => FmtSpan::FULL,
                            _ => panic!("test-log: RUST_LOG_SPAN_EVENTS must contain filters separated by `,`.\n\t\
                                         For example: `active` or `new,close`\n\t\
                                         Supported filters: new, enter, exit, close, active, full\n\t\
                                         Got: {}", value),
                        })
                        .fold(FmtSpan::NONE, |acc, filter| filter | acc)
                },
                Err(::std::env::VarError::NotUnicode(_)) =>
                    panic!("test-log: RUST_LOG_SPAN_EVENTS must contain a valid UTF-8 string"),
                Err(::std::env::VarError::NotPresent) => FmtSpan::NONE,
            }
        };
        let fmt_layer = fmt::layer()
            .with_test_writer()
            .with_span_events(event_filter);
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("shuttle_deployer"))
            .unwrap();

        tracing_subscriber::registry()
            .with(DeployLayer::new(
                Arc::clone(&recorder),
                InternalLogOrigin::Deployer,
            ))
            .with(filter_layer)
            .with(fmt_layer)
            .init();

        recorder
    };

    #[derive(Clone)]
    struct RecorderMock {
        log_origins: Arc<Mutex<Vec<TestInternalLog>>>,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct TestInternalLog {
        id: Uuid,
        origin: InternalLogOrigin,
    }

    impl From<Log> for TestInternalLog {
        fn from(log: Log) -> Self {
            Self {
                id: log.deployment_id,
                origin: log.internal_origin,
            }
        }
    }

    impl RecorderMock {
        fn new() -> Arc<Mutex<Self>> {
            Arc::new(Mutex::new(Self {
                log_origins: Arc::new(Mutex::new(Vec::new())),
            }))
        }

        fn get_deployment_log_origins(&self, id: &Uuid) -> Vec<TestInternalLog> {
            self.log_origins
                .lock()
                .unwrap()
                .iter()
                .filter(|log| log.id == *id)
                .cloned()
                .collect()
        }
    }

    impl LogRecorder for RecorderMock {
        fn record(&self, event: Log) {
            // We are only testing the state transitions
            self.log_origins.lock().unwrap().push(event.into());
        }
    }

    struct ProvisionerMock;

    #[async_trait]
    impl Provisioner for ProvisionerMock {
        async fn provision_database(
            &self,
            _request: tonic::Request<DatabaseRequest>,
        ) -> Result<tonic::Response<DatabaseResponse>, tonic::Status> {
            panic!("no deploy layer tests should request a db");
        }

        async fn delete_database(
            &self,
            _request: tonic::Request<DatabaseRequest>,
        ) -> Result<tonic::Response<DatabaseDeletionResponse>, tonic::Status> {
            panic!("no deploy layer tests should request delete a db");
        }

        async fn health_check(
            &self,
            _request: tonic::Request<Ping>,
        ) -> Result<tonic::Response<Pong>, tonic::Status> {
            panic!("no run tests should do a health check");
        }
    }

    async fn get_runtime_manager() -> Arc<tokio::sync::Mutex<RuntimeManager>> {
        let provisioner_addr =
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_unused_port().unwrap());
        let logger_uri = format!(
            "http://{}",
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_unused_port().unwrap())
        );
        let mock = ProvisionerMock;

        tokio::spawn(async move {
            Server::builder()
                .add_service(ProvisionerServer::new(mock))
                .serve(provisioner_addr)
                .await
                .unwrap();
        });

        let tmp_dir = Builder::new().prefix("shuttle_run_test").tempdir().unwrap();
        let path = tmp_dir.into_path();

        let channel = Endpoint::try_from(logger_uri.to_string())
            .unwrap()
            .connect()
            .await
            .expect("failed to connect to logger");

        let channel = ServiceBuilder::new()
            .layer(ClaimLayer)
            .layer(InjectPropagationLayer)
            .service(channel);

        let logger_client = Batcher::wrap(LoggerClient::new(channel));

        RuntimeManager::new(
            path,
            format!("http://{}", provisioner_addr),
            logger_uri,
            logger_client,
            None,
        )
    }

    #[async_trait::async_trait]
    impl SecretRecorder for Arc<Mutex<RecorderMock>> {
        type Err = std::io::Error;

        async fn insert_secret(
            &self,
            _service_id: &Ulid,
            _key: &str,
            _value: &str,
        ) -> Result<(), Self::Err> {
            panic!("no tests should set secrets")
        }
    }

    impl<R: LogRecorder> LogRecorder for Arc<Mutex<R>> {
        fn record(&self, event: Log) {
            self.lock().unwrap().record(event);
        }
    }

    #[derive(Clone)]
    struct StubDeploymentUpdater;

    #[async_trait::async_trait]
    impl DeploymentUpdater for StubDeploymentUpdater {
        type Err = std::io::Error;

        async fn set_address(&self, _id: &Uuid, _address: &SocketAddr) -> Result<(), Self::Err> {
            Ok(())
        }

        async fn set_is_next(&self, _id: &Uuid, _is_next: bool) -> Result<(), Self::Err> {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct StubActiveDeploymentGetter;

    #[async_trait::async_trait]
    impl ActiveDeploymentsGetter for StubActiveDeploymentGetter {
        type Err = std::io::Error;

        async fn get_active_deployments(
            &self,
            _service_id: &Ulid,
        ) -> std::result::Result<Vec<Uuid>, Self::Err> {
            Ok(vec![])
        }
    }

    #[derive(Clone)]
    struct StubBuildQueueClient;

    #[async_trait::async_trait]
    impl BuildQueueClient for StubBuildQueueClient {
        async fn get_slot(
            &self,
            _id: Uuid,
        ) -> Result<bool, crate::deployment::gateway_client::Error> {
            Ok(true)
        }

        async fn release_slot(
            &self,
            _id: Uuid,
        ) -> Result<(), crate::deployment::gateway_client::Error> {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct StubSecretGetter;

    #[async_trait::async_trait]
    impl SecretGetter for StubSecretGetter {
        type Err = std::io::Error;

        async fn get_secrets(&self, _service_id: &Ulid) -> Result<Vec<Secret>, Self::Err> {
            Ok(Default::default())
        }
    }

    #[derive(Clone)]
    struct StubResourceManager;

    #[async_trait]
    impl ResourceManager for StubResourceManager {
        type Err = std::io::Error;

        async fn insert_resources(
            &mut self,
            _resource: Vec<shuttle_proto::resource_recorder::record_request::Resource>,
            _service_id: &ulid::Ulid,
            _claim: Claim,
        ) -> Result<ResultResponse, Self::Err> {
            Ok(ResultResponse {
                success: true,
                message: "dummy impl".to_string(),
            })
        }
        async fn get_resources(
            &mut self,
            _service_id: &ulid::Ulid,
            _claim: Claim,
        ) -> Result<ResourcesResponse, Self::Err> {
            Ok(ResourcesResponse {
                success: true,
                message: "dummy impl".to_string(),
                resources: Vec::new(),
            })
        }
    }

    async fn test_origins(id: &Uuid, expected_origins: Vec<TestInternalLog>) {
        loop {
            let origins = RECORDER.lock().unwrap().get_deployment_log_origins(id);
            if origins == expected_origins {
                return;
            }

            for (actual, expected) in origins.iter().zip(&expected_origins) {
                if actual != expected {
                    return;
                }
            }

            sleep(Duration::from_millis(250)).await;
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn deployment_to_be_queued() {
        let deployment_manager = get_deployment_manager().await;

        let queued = get_queued_test_project("sleep-async");
        let id = queued.id;
        deployment_manager.queue_push(queued).await;

        let test = test_origins(
            &id,
            vec![TestInternalLog {
                id,
                origin: InternalLogOrigin::Deployer,
            }],
        );

        select! {
            _ = sleep(Duration::from_secs(460)) => {
                let states = RECORDER.lock().unwrap().get_deployment_log_origins(&id);
                panic!("states should go into 'Running' for a valid service: {:#?}", states);
            },
            _ = test => {}
        };

        // Send kill signal
        deployment_manager.kill(id).await;

        let test = test_origins(
            &id,
            vec![TestInternalLog {
                id,
                origin: InternalLogOrigin::Deployer,
            }],
        );

        select! {
            _ = sleep(Duration::from_secs(60)) => {
                let states = RECORDER.lock().unwrap().get_deployment_log_origins(&id);
                panic!("states should go into 'Stopped' for a valid service: {:#?}", states);
            },
            _ = test => {}
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn deployment_self_stop() {
        let deployment_manager = get_deployment_manager().await;

        let queued = get_queued_test_project("self-stop");
        let id = queued.id;
        deployment_manager.queue_push(queued).await;

        let test = test_origins(
            &id,
            vec![TestInternalLog {
                id,
                origin: InternalLogOrigin::Deployer,
            }],
        );

        select! {
            _ = sleep(Duration::from_secs(460)) => {
                let states = RECORDER.lock().unwrap().get_deployment_log_origins(&id);
                panic!("states should go into 'Completed' when a service stops by itself: {:#?}", states);
            }
            _ = test => {}
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn deployment_bind_panic() {
        let deployment_manager = get_deployment_manager().await;

        let queued = get_queued_test_project("bind-panic");
        let id = queued.id;
        deployment_manager.queue_push(queued).await;

        let test = test_origins(
            &id,
            vec![TestInternalLog {
                id,
                origin: InternalLogOrigin::Deployer,
            }],
        );

        select! {
            _ = sleep(Duration::from_secs(460)) => {
                let states = RECORDER.lock().unwrap().get_deployment_log_origins(&id);
                panic!("states should go into 'Crashed' panicking in bind: {:#?}", states);
            }
            _ = test => {}
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn deployment_main_panic() {
        let deployment_manager = get_deployment_manager().await;

        let queued = get_queued_test_project("main-panic");
        let id = queued.id;
        deployment_manager.queue_push(queued).await;

        let test = test_origins(
            &id,
            vec![TestInternalLog {
                id,
                origin: InternalLogOrigin::Deployer,
            }],
        );

        select! {
            _ = sleep(Duration::from_secs(460)) => {
                let states = RECORDER.lock().unwrap().get_deployment_log_origins(&id);
                panic!("states should go into 'Crashed' when panicking in main: {:#?}", states);
            }
            _ = test => {}
        }
    }

    #[tokio::test]
    async fn deployment_from_run() {
        let deployment_manager = get_deployment_manager().await;

        let id = Uuid::new_v4();
        deployment_manager
            .run_push(Built {
                id,
                service_name: "run-test".to_string(),
                service_id: Ulid::new(),
                project_id: Ulid::new(),
                tracing_context: Default::default(),
                is_next: false,
                claim: Default::default(),
            })
            .await;

        let test = test_origins(
            &id,
            vec![TestInternalLog {
                id,
                origin: InternalLogOrigin::Deployer,
            }],
        );

        select! {
            _ = sleep(Duration::from_secs(50)) => {
                let states = RECORDER.lock().unwrap().get_deployment_log_origins(&id);
                panic!("from running should start in built and end in crash for invalid: {:#?}", states)
            },
            _ = test => {}
        };
    }

    #[tokio::test]
    async fn scope_with_nil_id() {
        let deployment_manager = get_deployment_manager().await;

        let id = Uuid::nil();
        deployment_manager
            .queue_push(Queued {
                id,
                service_name: "nil_id".to_string(),
                service_id: Ulid::new(),
                project_id: Ulid::new(),
                data: Bytes::from("violets are red").to_vec(),
                will_run_tests: false,
                tracing_context: Default::default(),
                claim: Default::default(),
            })
            .await;

        // Give it a small time to start up
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let recorder = RECORDER.lock().unwrap();
        let states = recorder.get_deployment_log_origins(&id);

        assert!(
            states.is_empty(),
            "no logs should be recorded when the scope id is invalid:\n\t{states:#?}"
        );
    }

    async fn get_deployment_manager() -> DeploymentManager {
        DeploymentManager::builder()
            .build_log_recorder(RECORDER.clone())
            .secret_recorder(RECORDER.clone())
            .active_deployment_getter(StubActiveDeploymentGetter)
            .artifacts_path(PathBuf::from("/tmp"))
            .secret_getter(StubSecretGetter)
            .resource_manager(StubResourceManager)
            .runtime(get_runtime_manager().await)
            .deployment_updater(StubDeploymentUpdater)
            .queue_client(StubBuildQueueClient)
            .build()
    }

    fn get_queued_test_project(name: &str) -> Queued {
        let enc = GzEncoder::new(Vec::new(), Compression::fast());
        let mut tar = tar::Builder::new(enc);

        for dir_entry in read_dir(format!("tests/deploy_layer/{name}")).unwrap() {
            let dir_entry = dir_entry.unwrap();
            if dir_entry.file_name() != "target" {
                let path = format!("{name}/{}", dir_entry.file_name().to_str().unwrap());

                if dir_entry.file_type().unwrap().is_dir() {
                    tar.append_dir_all(path, dir_entry.path()).unwrap();
                } else {
                    tar.append_path_with_name(dir_entry.path(), path).unwrap();
                }
            }
        }

        let enc = tar.into_inner().unwrap();
        let bytes = enc.finish().unwrap();

        println!("{name}: finished getting archive for test");

        Queued {
            id: Uuid::new_v4(),
            service_name: format!("deploy-layer-{name}"),
            service_id: Ulid::new(),
            project_id: Ulid::new(),
            data: bytes,
            will_run_tests: false,
            tracing_context: Default::default(),
            claim: Default::default(),
        }
    }
}
