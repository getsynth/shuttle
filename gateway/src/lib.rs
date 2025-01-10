#[macro_use]
extern crate async_trait;

use std::convert::Infallible;
use std::error::Error as StdError;
use std::pin::Pin;

use bollard::Docker;
use futures::prelude::*;
use hyper::client::HttpConnector;
use hyper::Client;
use once_cell::sync::Lazy;
use service::ContainerSettings;
use shuttle_backends::project_name::ProjectName;
use shuttle_common::models::user::UserId;
use strum::Display;

pub mod acme;
pub mod api;
pub mod args;
pub mod auth;
pub mod project;
pub mod proxy;
pub mod service;
pub mod task;
pub mod tls;
pub mod worker;

pub const DOCKER_STATS_PATH_CGROUP_V1: &str = "/sys/fs/cgroup/cpuacct/docker";
pub const DOCKER_STATS_PATH_CGROUP_V2: &str = "/sys/fs/cgroup/system.slice";

#[derive(Clone, Display, PartialEq, Eq)]
pub enum DockerStatsSource {
    CgroupV1,
    CgroupV2,
    Bollard,
}
static AUTH_CLIENT: Lazy<Client<HttpConnector>> = Lazy::new(Client::new);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDetails {
    pub project_name: ProjectName,
    pub account_name: Option<String>,
    pub user_id: UserId,
}

impl From<ProjectDetails> for shuttle_common::models::admin::ProjectResponse {
    fn from(project: ProjectDetails) -> Self {
        Self {
            project_name: project.project_name.to_string(),
            account_name: project.account_name.unwrap_or_default(),
            user_id: project.user_id,
        }
    }
}

#[async_trait]
pub trait DockerContext: Send + Sync {
    fn docker(&self) -> &Docker;

    fn container_settings(&self) -> &ContainerSettings;

    async fn get_stats(&self, container_id: &str) -> Result<u64, String>;
}

/// A generic state which can, when provided with a [`Context`], do
/// some work and advance itself
#[async_trait]
pub trait State<Ctx>: Send {
    type Next;

    type Error;

    async fn next(self, ctx: &Ctx) -> Result<Self::Next, Self::Error>;
}

pub type StateTryStream<'c, St, Err> = Pin<Box<dyn Stream<Item = Result<St, Err>> + Send + 'c>>;

pub trait StateExt<Ctx>: TryState + State<Ctx, Error = Infallible, Next = Self>
where
    Ctx: Sync,
    Self: Clone,
{
    /// Convert the state into a [`TryStream`] that yields
    /// the generated states.
    ///
    /// This stream will not end.
    fn into_stream<'c>(self, ctx: &'c Ctx) -> StateTryStream<'c, Self, Self::ErrorVariant>
    where
        Self: 'c,
    {
        Box::pin(stream::try_unfold((self, ctx), |(state, ctx)| async move {
            state
                .next(ctx)
                .await
                .unwrap() // EndState's `next` is Infallible
                .into_result()
                .map(|state| Some((state.clone(), (state, ctx))))
        }))
    }
}

impl<Ctx, S> StateExt<Ctx> for S
where
    S: Clone + TryState + State<Ctx, Error = Infallible, Next = Self>,
    Ctx: Send + Sync,
{
}

/// A [`State`] which contains all its transitions, including
/// failures
pub trait TryState: Sized {
    type ErrorVariant;

    fn into_result(self) -> Result<Self, Self::ErrorVariant>;
}

pub trait IntoTryState<S>
where
    S: TryState,
{
    fn into_try_state(self) -> Result<S, Infallible>;
}

impl<S, F, Err> IntoTryState<S> for Result<F, Err>
where
    S: TryState + From<F> + From<Err>,
{
    fn into_try_state(self) -> Result<S, Infallible> {
        self.map(|s| S::from(s)).or_else(|err| Ok(S::from(err)))
    }
}

#[async_trait]
pub trait Refresh<Ctx>: Sized {
    type Error: StdError;

    async fn refresh(self, ctx: &Ctx) -> Result<Self, Self::Error>;
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::fs::{canonicalize, read_dir};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use anyhow::{anyhow, Context as AnyhowContext};
    use axum::headers::authorization::Bearer;
    use axum::headers::Authorization;
    use axum::routing::get;
    use axum::{extract, Router, TypedHeader};
    use bollard::Docker;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use fqdn::FQDN;
    use futures::prelude::*;
    use http::Method;
    use hyper::client::HttpConnector;
    use hyper::http::uri::Scheme;
    use hyper::http::Uri;
    use hyper::{Body, Client as HyperClient, Request, Response, StatusCode};
    use jsonwebtoken::EncodingKey;
    use rand::distributions::{Alphanumeric, DistString, Distribution, Uniform};
    use ring::signature::{self, Ed25519KeyPair, KeyPair};
    use shuttle_backends::auth::ConvertResponse;
    use shuttle_backends::test_utils::gateway::PermissionsMock;
    use shuttle_backends::test_utils::provisioner::get_mocked_provisioner;
    use shuttle_backends::test_utils::resource_recorder::get_mocked_resource_recorder;
    use shuttle_common::claims::Claim;
    use shuttle_common::models::deployment::DeploymentRequest;
    use shuttle_common::models::user::AccountTier;
    use shuttle_common::models::{project, service};
    use sqlx::sqlite::SqliteConnectOptions;
    use sqlx::{query, SqlitePool};
    use test_context::AsyncTestContext;
    use tokio::sync::mpsc::{channel, Sender};
    use tokio::time::sleep;
    use tower::Service;

    use crate::acme::AcmeClient;
    use crate::api::latest::ApiBuilder;
    use crate::args::{PermitArgs, ServiceArgs, StartArgs, UseTls};
    use crate::project::Project;
    use crate::proxy::UserServiceBuilder;
    use crate::service::{ContainerSettings, GatewayService, MIGRATIONS};
    use crate::task::BoxedTask;
    use crate::worker::Worker;
    use crate::DockerContext;

    macro_rules! value_block_helper {
        ($next:ident, $block:block) => {
            $block
        };
        ($next:ident,) => {
            $next
        };
    }

    macro_rules! assert_stream_matches {
        (
            $stream:ident,
            $(#[assertion = $assert:literal])?
                $($pattern:pat_param)|+ $(if $guard:expr)? $(=> $more:block)?,
        ) => {{
            let next = ::futures::stream::StreamExt::next(&mut $stream)
                .await
                .expect("Stream ended before the last of assertions");

            match &next {
                $($pattern)|+ $(if $guard)? => {
                    print!("{}", ::colored::Colorize::green(::colored::Colorize::bold("[ok]")));
                    $(print!(" {}", $assert);)?
                        print!("\n");
                    crate::tests::value_block_helper!(next, $($more)?)
                },
                _ => {
                    eprintln!("{} {:#?}", ::colored::Colorize::red(::colored::Colorize::bold("[err]")), next);
                    eprint!("{}", ::colored::Colorize::red(::colored::Colorize::bold("Assertion failed")));
                    $(eprint!(": {}", $assert);)?
                        eprint!("\n");
                    panic!("State mismatch")
                }
            }
        }};
        (
            $stream:ident,
            $(#[$($meta:tt)*])*
                $($pattern:pat_param)|+ $(if $guard:expr)? $(=> $more:block)?,
            $($(#[$($metas:tt)*])* $($patterns:pat_param)|+ $(if $guards:expr)? $(=> $mores:block)?,)+
        ) => {{
            assert_stream_matches!(
                $stream,
                $(#[$($meta)*])* $($pattern)|+ $(if $guard)? => {
                    $($more)?
                        assert_stream_matches!(
                            $stream,
                            $($(#[$($metas)*])* $($patterns)|+ $(if $guards)? $(=> $mores)?,)+
                        )
                },
            )
        }};
    }

    macro_rules! assert_matches {
        {
            $ctx:ident,
            $state:expr,
            $($(#[$($meta:tt)*])* $($patterns:pat_param)|+ $(if $guards:expr)? $(=> $mores:block)?,)+
        } => {{
            let state = $state;
            let mut stream = crate::StateExt::into_stream(state, &$ctx);
            assert_stream_matches!(
                stream,
                $($(#[$($meta)*])* $($patterns)|+ $(if $guards)? $(=> $mores)?,)+
            )
        }}
    }

    macro_rules! timed_loop {
        (wait: $wait:literal$(, max: $max:literal)?, $block:block) => {{
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            let mut tries = 0;
            loop {
                $block
                    tries += 1;
                $(if tries > $max {
                    panic!("timed out in the loop");
                })?
                    ::tokio::time::sleep(::std::time::Duration::from_secs($wait)).await;
            }
        }};
    }

    pub(crate) use {assert_matches, assert_stream_matches, value_block_helper};

    mod request_builder_ext {
        pub trait Sealed {}

        impl Sealed for axum::http::request::Builder {}

        impl Sealed for &mut axum::headers::HeaderMap {}

        impl<B> Sealed for axum::http::Request<B> {}
    }

    pub trait RequestBuilderExt: Sized + request_builder_ext::Sealed {
        fn with_header<H: axum::headers::Header>(self, header: &H) -> Self;
    }

    impl RequestBuilderExt for axum::http::request::Builder {
        fn with_header<H: axum::headers::Header>(mut self, header: &H) -> Self {
            self.headers_mut().unwrap().with_header(header);
            self
        }
    }

    impl RequestBuilderExt for &mut axum::headers::HeaderMap {
        fn with_header<H: axum::headers::Header>(self, header: &H) -> Self {
            let mut buf = vec![];
            header.encode(&mut buf);
            self.append(H::name(), buf.pop().unwrap());
            self
        }
    }

    impl<B> RequestBuilderExt for Request<B> {
        fn with_header<H: axum::headers::Header>(mut self, header: &H) -> Self {
            self.headers_mut().with_header(header);
            self
        }
    }

    pub struct Client<C = HttpConnector, B = Body> {
        target: SocketAddr,
        hyper: Option<HyperClient<C, B>>,
    }

    impl<C, B> Client<C, B> {
        pub fn new<A: Into<SocketAddr>>(target: A) -> Self {
            Self {
                target: target.into(),
                hyper: None,
            }
        }

        pub fn with_hyper_client(mut self, client: HyperClient<C, B>) -> Self {
            self.hyper = Some(client);
            self
        }
    }

    impl Client<HttpConnector, Body> {
        pub async fn request(
            &self,
            mut req: Request<Body>,
        ) -> Result<Response<Vec<u8>>, hyper::Error> {
            if req.uri().authority().is_none() {
                let mut uri = req.uri().clone().into_parts();
                uri.scheme = Some(Scheme::HTTP);
                uri.authority = Some(self.target.to_string().parse().unwrap());
                *req.uri_mut() = Uri::from_parts(uri).unwrap();
            }
            self.hyper
                .as_ref()
                .unwrap()
                .request(req)
                .and_then(|mut resp| async move {
                    let body = resp
                        .body_mut()
                        .try_fold(Vec::new(), |mut acc, x| async move {
                            acc.extend(x);
                            Ok(acc)
                        })
                        .await?;
                    let (parts, _) = resp.into_parts();
                    Ok(Response::from_parts(parts, body))
                })
                .await
        }
    }

    pub struct World {
        docker: Docker,
        settings: ContainerSettings,
        args: StartArgs,
        pool: SqlitePool,
        acme_client: AcmeClient,
        auth_service: Arc<Mutex<AuthService>>,
        auth_uri: Uri,
    }

    #[derive(Clone)]
    pub struct WorldContext {
        pub docker: Docker,
        pub container_settings: ContainerSettings,
        pub auth_uri: Uri,
    }

    impl World {
        pub async fn new() -> Self {
            let docker = Docker::connect_with_local_defaults().unwrap();

            docker
                .list_images::<&str>(None)
                .await
                .context(anyhow!("A docker daemon does not seem accessible",))
                .unwrap();

            let control: i16 = Uniform::from(9000..10000).sample(&mut rand::thread_rng());
            let user = control + 1;
            let bouncer = user + 1;
            let auth_port = bouncer + 1;
            let control = format!("127.0.0.1:{control}").parse().unwrap();
            let user = format!("127.0.0.1:{user}").parse().unwrap();
            let bouncer = format!("127.0.0.1:{bouncer}").parse().unwrap();
            let auth: SocketAddr = format!("0.0.0.0:{auth_port}").parse().unwrap();
            let auth_uri: Uri = format!("http://{auth}").parse().unwrap();
            let resource_recorder_port = get_mocked_resource_recorder().await;
            let provisioner_port = get_mocked_provisioner().await;

            let auth_service = AuthService::new(auth);
            auth_service
                .lock()
                .unwrap()
                .users
                .insert("gateway".to_string(), AccountTier::Deployer);

            let prefix = format!(
                "shuttle_test_{}_",
                Alphanumeric.sample_string(&mut rand::thread_rng(), 4)
            );

            let image = env::var("SHUTTLE_TESTS_RUNTIME_IMAGE")
                .unwrap_or_else(|_| "public.ecr.aws/shuttle-dev/deployer:latest".to_string());

            let network_name =
                env::var("SHUTTLE_TESTS_NETWORK").unwrap_or_else(|_| "shuttle_default".to_string());

            let docker_host = "/var/run/docker.sock".to_string();

            let args = StartArgs {
                control,
                user,
                bouncer,
                use_tls: UseTls::Disable,
                cors_origin: "http://localhost:3001".to_string(),
                context: ServiceArgs {
                    docker_host,
                    image,
                    prefix,
                    provisioner_uri: format!("http://host.docker.internal:{provisioner_port}"),
                    // The started containers need to reach auth on the host.
                    // For this to work, the firewall should not be blocking traffic on the `SHUTTLE_TESTS_NETWORK` interface.
                    // The following command can be used on NixOs to allow traffic on the interface.
                    // ```
                    // sudo iptables -I nixos-fw -i <interface> -j nixos-fw-accept
                    // ```
                    //
                    // Something like this should work on other systems.
                    // ```
                    // sudo iptables -I INPUT -i <interface> -j ACCEPT
                    // ```
                    auth_uri: format!("http://host.docker.internal:{auth_port}")
                        .parse()
                        .unwrap(),
                    resource_recorder_uri: format!(
                        "http://host.docker.internal:{resource_recorder_port}"
                    )
                    .parse()
                    .unwrap(),
                    network_name,
                    proxy_fqdn: FQDN::from_str("test.shuttleapp.rs").unwrap(),
                    admin_key: "dummykey".to_string(),
                    deploys_api_key: "gateway".to_string(),
                    cch_container_limit: 1,
                    soft_container_limit: 2,
                    hard_container_limit: 3,

                    // Allow access to the auth on the host
                    extra_hosts: vec!["host.docker.internal:host-gateway".to_string()],
                },
                permit: PermitArgs {
                    permit_api_uri: Default::default(), // TODO: will need mock?
                    permit_pdp_uri: Default::default(), // TODO: will need mock?
                    permit_env: Default::default(),     // TODO: will need mock?
                    permit_api_key: Default::default(), // TODO: will need mock?
                },
            };

            let settings = ContainerSettings::builder().from_args(&args.context).await;

            let pool = SqlitePool::connect_with(
                SqliteConnectOptions::from_str("sqlite::memory:")
                    .unwrap()
                    // Set the ulid0 extension for generating ULID's in migrations.
                    // This uses the ulid0.so file in the crate root, with the
                    // LD_LIBRARY_PATH env set in build.rs.
                    .extension("ulid0"),
            )
            .await
            .unwrap();
            MIGRATIONS.run(&pool).await.unwrap();

            let acme_client = AcmeClient::new();

            Self {
                docker,
                settings,
                args,
                pool,
                acme_client,
                auth_service,
                auth_uri,
            }
        }

        pub fn args(&self) -> ServiceArgs {
            self.args.context.clone()
        }

        pub fn pool(&self) -> SqlitePool {
            self.pool.clone()
        }

        pub fn fqdn(&self) -> FQDN {
            self.args().proxy_fqdn
        }

        pub fn acme_client(&self) -> AcmeClient {
            self.acme_client.clone()
        }

        /// Create user with a specific tier
        pub fn create_user(&self, user: &str, tier: AccountTier) -> String {
            self.auth_service
                .lock()
                .unwrap()
                .users
                .insert(user.to_string(), tier);

            user.to_string()
        }

        /// Create a user with the given name and tier and return the authorization bearer for the user
        pub fn create_authorization_bearer(
            &self,
            user: &str,
            tier: AccountTier,
        ) -> Authorization<Bearer> {
            let user_key = self.create_user(user, tier);
            Authorization::bearer(&user_key).unwrap()
        }

        pub fn set_super_user(&self, user: &str) {
            if let Some(tier) = self.auth_service.lock().unwrap().users.get_mut(user) {
                *tier = AccountTier::Admin;
            }
        }

        /// Create a service and sender to handle tasks. Also starts up a worker to create actual Docker containers for all requests
        pub async fn service(&self) -> (Arc<GatewayService>, Sender<BoxedTask>) {
            let service = Arc::new(
                GatewayService::init(
                    self.args(),
                    self.pool(),
                    "".into(),
                    Box::<PermissionsMock>::default(),
                )
                .await
                .unwrap(),
            );
            let worker = Worker::new();

            let (sender, mut receiver) = channel(256);
            tokio::spawn({
                let worker_sender = worker.sender();
                async move {
                    while let Some(work) = receiver.recv().await {
                        // Forward tasks to an actual worker
                        worker_sender
                            .send(work)
                            .await
                            .map_err(|_| "could not send work")
                            .unwrap();
                    }
                }
            });

            let _worker = tokio::spawn(async move {
                worker.start().await;
            });

            // Allow the spawns to start
            tokio::time::sleep(Duration::from_secs(1)).await;

            (service, sender)
        }

        /// Create a router to make API calls against
        pub fn router(&self, service: Arc<GatewayService>, sender: Sender<BoxedTask>) -> Router {
            ApiBuilder::new()
                .with_service(Arc::clone(&service))
                .with_sender(sender)
                .with_default_routes()
                .with_auth_service(self.context().auth_uri, "dummykey".to_string())
                .into_router()
        }

        pub fn client<A: Into<SocketAddr>>(addr: A) -> Client {
            let hyper = HyperClient::builder().build(HttpConnector::new());
            Client::new(addr).with_hyper_client(hyper)
        }

        pub fn context(&self) -> WorldContext {
            WorldContext {
                docker: self.docker.clone(),
                container_settings: self.settings.clone(),
                auth_uri: self.auth_uri.clone(),
            }
        }
    }

    #[async_trait]
    impl DockerContext for WorldContext {
        fn docker(&self) -> &Docker {
            &self.docker
        }

        fn container_settings(&self) -> &ContainerSettings {
            &self.container_settings
        }

        async fn get_stats(&self, _container_id: &str) -> Result<u64, String> {
            Ok(0)
        }
    }

    struct AuthService {
        users: HashMap<String, AccountTier>,
        encoding_key: EncodingKey,
        public_key: Vec<u8>,
    }

    impl AuthService {
        fn new(address: SocketAddr) -> Arc<Mutex<Self>> {
            let doc = signature::Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())
                .unwrap();
            let encoding_key = EncodingKey::from_ed_der(doc.as_ref());
            let pair = Ed25519KeyPair::from_pkcs8(doc.as_ref()).unwrap();
            let public_key = pair.public_key().as_ref().to_vec();

            let this = Arc::new(Mutex::new(Self {
                users: HashMap::new(),
                encoding_key,
                public_key,
            }));

            let router = Router::new()
                .route(
                    "/public-key",
                    get(|extract::State(state): extract::State<Arc<Mutex<Self>>>| async move {
                        state.lock().unwrap().public_key.clone()
                    }),
                )
                .route(
                    "/auth/key",
                    get(|extract::State(state): extract::State<Arc<Mutex<Self>>>, TypedHeader(bearer): TypedHeader<Authorization<Bearer>> | async move {
                        let state = state.lock().unwrap();

                        if let Some(tier) = state.users.get(bearer.token()) {
                            let claim = Claim::new(bearer.token().to_string(), (*tier).into(), *tier, *tier);
                            let token = claim.into_token(&state.encoding_key)?;
                            Ok(serde_json::to_vec(&ConvertResponse { token }).unwrap())
                        } else {
                            Err(StatusCode::NOT_FOUND)
                        }
                    }),
                )
                .with_state(this.clone());

            tokio::spawn(async move {
                axum::Server::bind(&address)
                    .serve(router.into_make_service())
                    .await
                    .unwrap();
            });

            this
        }
    }

    /// Helper struct to wrap a bunch of commands to run against gateway's API
    pub struct TestGateway {
        router: Router,
        authorization: Authorization<Bearer>,
        service: Arc<GatewayService>,
        sender: Sender<BoxedTask>,
        world: World,
    }

    impl TestGateway {
        /// Try to create a project with a given user and return the request response
        pub async fn try_user_create_project(
            &mut self,
            project_name: &str,
            authorization: &Authorization<Bearer>,
        ) -> StatusCode {
            self.router
                .call(
                    Request::builder()
                        .method("POST")
                        .uri(format!("/projects/{project_name}"))
                        .header("Content-Type", "application/json")
                        .body("{\"idle_minutes\": 3}".into())
                        .unwrap()
                        .with_header(authorization),
                )
                .await
                .unwrap()
                .status()
        }

        /// Try to create a project and return the request response
        pub async fn try_create_project(&mut self, project_name: &str) -> StatusCode {
            self.try_user_create_project(project_name, &self.authorization.clone())
                .await
        }

        /// Create a new project using the given user and return its helping wrapper
        pub async fn user_create_project(
            &mut self,
            project_name: &str,
            authorization: &Authorization<Bearer>,
        ) -> TestProject {
            let status_code = self
                .try_user_create_project(project_name, authorization)
                .await;

            assert_eq!(
                status_code,
                StatusCode::OK,
                "could not create {project_name}"
            );

            let mut this = TestProject {
                authorization: authorization.clone(),
                project_name: project_name.to_string(),
                router: self.router.clone(),
                pool: self.world.pool(),
                service: self.service.clone(),
                sender: self.sender.clone(),
            };

            this.wait_for_state(project::State::Ready).await;

            this
        }

        /// Create a new project in the test world and return its helping wrapper
        pub async fn create_project(&mut self, project_name: &str) -> TestProject {
            self.user_create_project(project_name, &self.authorization.clone())
                .await
        }

        /// Get authorization bearer for a new user
        pub fn new_authorization_bearer(
            &self,
            user: &str,
            tier: AccountTier,
        ) -> Authorization<Bearer> {
            self.world.create_authorization_bearer(user, tier)
        }
    }

    impl AsyncTestContext for TestGateway {
        async fn setup() -> Self {
            let world = World::new().await;

            let (service, sender) = world.service().await;

            let router = world.router(service.clone(), sender.clone());
            let authorization = world.create_authorization_bearer("neo", AccountTier::Basic);

            Self {
                router,
                authorization,
                service,
                sender,
                world,
            }
        }

        async fn teardown(self) {}
    }

    /// Helper struct to wrap a bunch of commands to run against a test project
    pub struct TestProject {
        router: Router,
        authorization: Authorization<Bearer>,
        project_name: String,
        pool: SqlitePool,
        service: Arc<GatewayService>,
        sender: Sender<BoxedTask>,
    }

    impl TestProject {
        /// Wait a few seconds for the project to enter the desired state
        pub async fn wait_for_state(&mut self, state: project::State) {
            let mut tries = 0;
            let project_name = &self.project_name;

            loop {
                let resp = self
                    .router
                    .call(
                        Request::get(format!("/projects/{project_name}"))
                            .with_header(&self.authorization)
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::OK);
                let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
                let project: project::Response = serde_json::from_slice(&body).unwrap();

                if project.state == state {
                    break;
                }

                tries += 1;
                if tries > 12 {
                    panic!("timed out waiting for state {state}");
                }

                sleep(Duration::from_secs(1)).await;
            }
        }

        /// Is this project still available - aka has it been deleted
        pub async fn is_missing(&mut self) -> bool {
            let project_name = &self.project_name;

            let resp = self
                .router
                .call(
                    Request::get(format!("/projects/{project_name}"))
                        .with_header(&self.authorization)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            resp.status() == StatusCode::NOT_FOUND
        }

        /// Destroy / stop a project. Like `cargo shuttle project stop`
        pub async fn destroy_project(&mut self) {
            let TestProject {
                router,
                authorization,
                project_name,
                ..
            } = self;

            router
                .call(
                    Request::builder()
                        .method("DELETE")
                        .uri(format!("/projects/{project_name}"))
                        .body(Body::empty())
                        .unwrap()
                        .with_header(authorization),
                )
                .map_ok(|resp| {
                    assert_eq!(resp.status(), StatusCode::OK);
                })
                .await
                .unwrap();

            self.wait_for_state(project::State::Destroyed).await;
        }

        /// Send a request to the router for this project
        pub async fn router_call(&mut self, method: Method, sub_path: &str) -> StatusCode {
            let project_name = &self.project_name;

            self.router
                .call(
                    Request::builder()
                        .method(method)
                        .uri(format!("/projects/{project_name}{sub_path}"))
                        .body(Body::empty())
                        .unwrap()
                        .with_header(&self.authorization),
                )
                .map_ok(|resp| resp.status())
                .await
                .unwrap()
        }

        /// Just deploy the code at the path and don't wait for it to finish
        pub async fn just_deploy(&mut self, path: &str) {
            let path = canonicalize(path).expect("deploy path to be valid");
            let name = path.file_name().unwrap().to_str().unwrap();
            let enc = GzEncoder::new(Vec::new(), Compression::fast());
            let mut tar = tar::Builder::new(enc);

            for dir_entry in read_dir(&path).unwrap() {
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

            let project_name = &self.project_name;
            let deployment_req = rmp_serde::to_vec(&DeploymentRequest {
                data: bytes,
                no_test: true,
                ..Default::default()
            })
            .expect("to serialize DeploymentRequest as a MessagePack byte vector");

            self.router
                .call(
                    Request::builder()
                        .method(Method::POST)
                        .header("Transfer-Encoding", "chunked")
                        .uri(format!("/projects/{project_name}/services/{project_name}"))
                        .body(deployment_req.into())
                        .unwrap()
                        .with_header(&self.authorization),
                )
                .map_ok(|resp| {
                    assert_eq!(resp.status(), StatusCode::OK);
                })
                .await
                .unwrap();
        }

        /// Deploy the code at the path to the project and wait for it to finish
        pub async fn deploy(&mut self, path: &str) {
            self.just_deploy(path).await;

            let project_name = &self.project_name;

            // Wait for deployment to be up
            let mut tries = 0;

            loop {
                let resp = self
                    .router
                    .call(
                        Request::get(format!("/projects/{project_name}/services/{project_name}"))
                            .with_header(&self.authorization)
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::OK);
                let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
                let service: service::Summary = serde_json::from_slice(&body).unwrap();

                if service.deployment.is_some() {
                    break;
                }

                tries += 1;
                // We should consider making a mock deployer image to be able to "deploy" (aka fake deploy) things instantly for tests
                if tries > 240 {
                    panic!("timed out waiting for deployment");
                }

                sleep(Duration::from_secs(1)).await;
            }
        }

        /// Stop a service running in a project
        pub async fn stop_service(&mut self) {
            let TestProject {
                router,
                authorization,
                project_name,
                ..
            } = self;

            router
                .call(
                    Request::builder()
                        .method("DELETE")
                        .uri(format!("/projects/{project_name}/services/{project_name}"))
                        .body(Body::empty())
                        .unwrap()
                        .with_header(authorization),
                )
                .map_ok(|resp| {
                    assert_eq!(resp.status(), StatusCode::OK);
                })
                .await
                .unwrap();
        }

        /// Puts the project in a new state
        pub async fn update_state(&self, state: Project) {
            let TestProject {
                project_name, pool, ..
            } = self;

            let state = sqlx::types::Json(state);

            query("UPDATE projects SET project_state = ?1 WHERE project_name = ?2")
                .bind(&state)
                .bind(project_name)
                .execute(pool)
                .await
                .expect("test to update project state");
        }

        /// Run one iteration of health checks for this project
        pub async fn run_health_check(&self) {
            let handle = self
                .service
                .new_task()
                .project(self.project_name.parse().unwrap())
                .send(&self.sender)
                .await
                .expect("to send one ambulance task");

            // We wait for the check to be done before moving on
            handle.await
        }
    }

    impl AsyncTestContext for TestProject {
        async fn setup() -> Self {
            let mut world = TestGateway::setup().await;

            world.create_project("matrix").await
        }

        async fn teardown(mut self) {
            let dangling = !self.is_missing().await;

            if dangling {
                self.router_call(Method::DELETE, "/delete").await;
                eprintln!("test left a dangling project which you might need to clean manually");
            }
        }
    }

    #[tokio::test]
    async fn end_to_end() {
        let world = World::new().await;
        let service = Arc::new(
            GatewayService::init(
                world.args(),
                world.pool(),
                "".into(),
                Box::<PermissionsMock>::default(),
            )
            .await
            .unwrap(),
        );
        let worker = Worker::new();

        let (log_out, mut log_in) = channel(256);
        tokio::spawn({
            let sender = worker.sender();
            async move {
                while let Some(work) = log_in.recv().await {
                    sender
                        .send(work)
                        .await
                        .map_err(|_| "could not send work")
                        .unwrap();
                }
            }
        });

        let api_client = World::client(world.args.control);
        let api = ApiBuilder::new()
            .with_service(Arc::clone(&service))
            .with_sender(log_out.clone())
            .with_default_routes()
            .with_auth_service(world.context().auth_uri, "dummykey".to_string())
            .binding_to(world.args.control);

        let user = UserServiceBuilder::new()
            .with_service(Arc::clone(&service))
            .with_task_sender(log_out)
            .with_public(world.fqdn())
            .with_user_proxy_binding_to(world.args.user);

        let _gateway = tokio::spawn(async move {
            tokio::select! {
                _ = worker.start() => {},
                _ = api.serve() => {},
                _ = user.serve() => {}
            }
        });

        // Allow the spawns to start
        tokio::time::sleep(Duration::from_secs(1)).await;

        let neo_key = world.create_user("neo", AccountTier::Basic);

        let authorization = Authorization::bearer(&neo_key).unwrap();

        println!("Creating the matrix project");
        api_client
            .request(
                Request::post("/projects/matrix")
                    .with_header(&authorization)
                    .header("Content-Type", "application/json")
                    .body("{\"idle_minutes\": 3}".into())
                    .unwrap(),
            )
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
            })
            .await
            .unwrap();

        timed_loop!(wait: 1, max: 12, {
            let project: project::Response = api_client
                .request(
                    Request::get("/projects/matrix")
                        .with_header(&authorization)
                        .body(Body::empty())
                        .unwrap(),
                )
                .map_ok(|resp| {
                    assert_eq!(resp.status(), StatusCode::OK);
                    serde_json::from_slice(resp.body()).unwrap()
                })
                .await
                .unwrap();

            if project.state == project::State::Ready {
                break;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        });

        println!("get matrix project status");
        api_client
            .request(
                Request::get("/projects/matrix/status")
                    .with_header(&authorization)
                    .body(Body::empty())
                    .unwrap(),
            )
            .map_ok(|resp| assert_eq!(resp.status(), StatusCode::OK))
            .await
            .unwrap();

        println!("delete matrix project");
        api_client
            .request(
                Request::delete("/projects/matrix")
                    .with_header(&authorization)
                    .body(Body::empty())
                    .unwrap(),
            )
            .map_ok(|resp| assert_eq!(resp.status(), StatusCode::OK))
            .await
            .unwrap();

        timed_loop!(wait: 1, max: 20, {
            let resp = api_client
                .request(
                    Request::get("/projects/matrix")
                        .with_header(&authorization)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let resp = serde_json::from_slice::<project::Response>(resp.body().as_slice()).unwrap();
            if matches!(resp.state, project::State::Destroyed) {
                break;
            }
        });

        // Attempting to delete already Destroyed project will return Destroyed
        api_client
            .request(
                Request::delete("/projects/matrix")
                    .with_header(&authorization)
                    .body(Body::empty())
                    .unwrap(),
            )
            .map_ok(|resp| {
                assert_eq!(resp.status(), StatusCode::OK);
                let resp =
                    serde_json::from_slice::<project::Response>(resp.body().as_slice()).unwrap();
                assert_eq!(resp.state, project::State::Destroyed);
            })
            .await
            .unwrap();
    }
}
