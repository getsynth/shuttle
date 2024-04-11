use std::{
    collections::BTreeMap,
    iter::FromIterator,
    net::{Ipv4Addr, SocketAddr},
    ops::{Deref, DerefMut},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Context;
use async_trait::async_trait;
use core::future::Future;
use shuttle_common::{extract_propagation::ExtractPropagationLayer, secrets::Secret};
use shuttle_proto::{
    runtime::{
        runtime_server::{Runtime, RuntimeServer},
        LoadRequest, LoadResponse, StartRequest, StartResponse, StopReason, StopRequest,
        StopResponse, SubscribeStopRequest, SubscribeStopResponse,
    },
    runtime::{Ping, Pong},
};
use shuttle_service::{ResourceFactory, Service};
use tokio::sync::{
    broadcast::{self, Sender},
    mpsc,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

use crate::args::args;
use crate::print_version;

// uses custom macro instead of clap to reduce dependency weight
args! {
    pub struct Args {
        // The port to open the gRPC control layer on.
        // The address to expose for the service is given in the StartRequest.
        "--port" => pub port: u16,
    }
}

pub async fn start(loader: impl Loader + Send + 'static, runner: impl Runner + Send + 'static) {
    // `--version` overrides any other arguments.
    if std::env::args().any(|arg| arg == "--version") {
        print_version();
        return;
    }

    let args = match Args::parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Runtime received malformed or incorrect args, {e}");
            let help_str = "[HINT]: Run your Shuttle app with `cargo shuttle run`";
            let wrapper_str = "-".repeat(help_str.len());
            eprintln!("{wrapper_str}\n{help_str}\n{wrapper_str}");
            return;
        }
    };

    println!("{} {} executable started", crate::NAME, crate::VERSION);

    // this is handled after arg parsing to not interfere with --version above
    #[cfg(feature = "setup-tracing")]
    {
        use colored::Colorize;
        use tracing_subscriber::prelude::*;

        colored::control::set_override(true); // always apply color

        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().without_time())
            .with(
                // let user override RUST_LOG in local run if they want to
                tracing_subscriber::EnvFilter::try_from_default_env()
                    // otherwise use our default
                    .or_else(|_| tracing_subscriber::EnvFilter::try_new("info,shuttle=trace"))
                    .unwrap(),
            )
            .init();

        println!(
            "{}",
            "Shuttle's default tracing subscriber is initialized!".yellow(),
        );
        println!("To disable it and use your own, check the docs: https://docs.shuttle.rs/configuration/logs");
    }

    // where to serve the gRPC control layer
    let addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), args.port);

    let mut server_builder = Server::builder()
        .http2_keepalive_interval(Some(Duration::from_secs(60)))
        .layer(ExtractPropagationLayer);

    // A channel we can use to kill the runtime if it does not become healthy in time.
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let router = {
        let alpha = Alpha::new(loader, runner, tx);

        let svc = RuntimeServer::new(alpha);
        server_builder.add_service(svc)
    };

    tokio::select! {
        res = router.serve(addr) => {
            match res{
                Ok(_) => panic!("router shut down"),
                Err(e) => panic!("Error while serving address {addr}: {e}")
            }
        }
        res = rx => {
            match res{
                Ok(_) => panic!("Received runtime kill signal"),
                Err(e) => panic!("Receiver error: {e}")
            }
        }
    }
}

pub enum State {
    Unhealthy,
    Loading,
    Running,
}

pub struct Alpha<L, R> {
    // Mutexes are for interior mutability
    stopped_tx: Sender<(StopReason, String)>,
    loader: Mutex<Option<L>>,
    runner: Mutex<Option<R>>,
    /// The current state of the runtime, which is used by the ECS task to determine if the runtime
    /// is healthy.
    state: Arc<Mutex<State>>,
    runtime_kill_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

impl<L, R> Alpha<L, R> {
    pub fn new(loader: L, runner: R, runtime_kill_tx: tokio::sync::oneshot::Sender<()>) -> Self {
        let (stopped_tx, _stopped_rx) = broadcast::channel(10);

        Self {
            stopped_tx,
            kill_tx: Mutex::new(None),
            loader: Mutex::new(Some(loader)),
            runner: Mutex::new(Some(runner)),
            state: Arc::new(Mutex::new(State::Unhealthy)),
            runtime_kill_tx: Mutex::new(Some(runtime_kill_tx)),
        }
    }
}

#[async_trait]
pub trait Loader {
    async fn load(self, factory: ResourceFactory) -> Result<Vec<Vec<u8>>, shuttle_service::Error>;
}

#[async_trait]
impl<F, O> Loader for F
where
    F: FnOnce(ResourceFactory) -> O + Send,
    O: Future<Output = Result<Vec<Vec<u8>>, shuttle_service::Error>> + Send,
{
    async fn load(self, factory: ResourceFactory) -> Result<Vec<Vec<u8>>, shuttle_service::Error> {
        (self)(factory).await
    }
}

#[async_trait]
pub trait Runner {
    type Service: Service;

    async fn run(self, resources: Vec<Vec<u8>>) -> Result<Self::Service, shuttle_service::Error>;
}

#[async_trait]
impl<F, O, S> Runner for F
where
    F: FnOnce(Vec<Vec<u8>>) -> O + Send,
    O: Future<Output = Result<S, shuttle_service::Error>> + Send,
    S: Service,
{
    type Service = S;

    async fn run(self, resources: Vec<Vec<u8>>) -> Result<Self::Service, shuttle_service::Error> {
        (self)(resources).await
    }
}

#[async_trait]
impl<L, R, S> Runtime for Alpha<L, R>
where
    L: Loader + Send + 'static,
    R: Runner<Service = S> + Send + 'static,
    S: Service + 'static,
{
    async fn load(&self, request: Request<LoadRequest>) -> Result<Response<LoadResponse>, Status> {
        let LoadRequest {
            secrets,
            project_name,
            env,
            ..
        } = request.into_inner();

        // Sorts secrets by key
        let secrets = BTreeMap::from_iter(secrets.into_iter().map(|(k, v)| (k, Secret::new(v))));

        let factory = ResourceFactory::new(project_name, secrets, env.parse().unwrap());

        let loader = self.loader.lock().unwrap().deref_mut().take().unwrap();

        // send to new thread to catch panics
        let v = match tokio::spawn(loader.load(factory)).await {
            Ok(res) => match res {
                Ok(v) => v,
                Err(error) => {
                    println!("loading service failed: {error:#}");
                    return Ok(Response::new(LoadResponse {
                        success: false,
                        message: error.to_string(),
                        resources: vec![],
                    }));
                }
            },
            Err(error) => {
                if error.is_panic() {
                    let panic = error.into_panic();
                    let msg = match panic.downcast_ref::<String>() {
                        Some(msg) => msg.to_string(),
                        None => match panic.downcast_ref::<&str>() {
                            Some(msg) => msg.to_string(),
                            None => "<no panic message>".to_string(),
                        },
                    };
                    println!("loading service panicked: {msg}");
                    return Ok(Response::new(LoadResponse {
                        success: false,
                        message: msg,
                        resources: vec![],
                    }));
                } else {
                    println!("loading service crashed: {error:#}");
                    return Ok(Response::new(LoadResponse {
                        success: false,
                        message: error.to_string(),
                        resources: vec![],
                    }));
                }
            }
        };

        println!("setting current state to healthy");
        *self.state.lock().unwrap() = State::Loading;

        let state = self.state.clone();

        // Ensure that the runtime is set to unhealthy if it doesn't reach the running state after
        // it has sent a load response, so that the ECS task will fail.
        tokio::spawn(async move {
            // Note: The timeout is quite low as we are not actually provisioning resources after
            // sending the load response.
            tokio::time::sleep(Duration::from_secs(60)).await;
            if !matches!(state.lock().unwrap().deref(), State::Running) {
                let runtime_kill_tx = self
                    .runtime_kill_tx
                    .lock()
                    .unwrap()
                    .deref_mut()
                    .take()
                    .unwrap();
                println!("the runtime failed to enter the running state before timing out");

                runtime_kill_tx.send(()).unwrap();
            }
        });

        Ok(Response::new(LoadResponse {
            success: true,
            message: String::new(),
            resources: v,
        }))
    }

    async fn start(
        &self,
        request: Request<StartRequest>,
    ) -> Result<Response<StartResponse>, Status> {
        let StartRequest { ip, resources } = request.into_inner();
        let service_address = SocketAddr::from_str(&ip)
            .context("invalid socket address")
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let runner = self.runner.lock().unwrap().deref_mut().take().unwrap();

        let stopped_tx = self.stopped_tx.clone();

        // send to new thread to catch panics
        let service = match tokio::spawn(runner.run(resources)).await {
            Ok(res) => match res {
                Ok(service) => service,
                Err(error) => {
                    println!("starting service failed: {error:#}");
                    let _ = stopped_tx
                        .send((StopReason::Crash, error.to_string()))
                        .map_err(|e| println!("{e}"));
                    return Ok(Response::new(StartResponse {
                        success: false,
                        message: error.to_string(),
                    }));
                }
            },
            Err(error) => {
                if error.is_panic() {
                    let panic = error.into_panic();
                    let msg = match panic.downcast_ref::<String>() {
                        Some(msg) => msg.to_string(),
                        None => match panic.downcast_ref::<&str>() {
                            Some(msg) => msg.to_string(),
                            None => "<no panic message>".to_string(),
                        },
                    };

                    println!("loading service panicked: {msg}");
                    let _ = stopped_tx
                        .send((StopReason::Crash, msg.to_string()))
                        .map_err(|e| println!("{e}"));
                    return Ok(Response::new(StartResponse {
                        success: false,
                        message: msg,
                    }));
                }
                println!("loading service crashed: {error:#}");
                let _ = stopped_tx
                    .send((StopReason::Crash, error.to_string()))
                    .map_err(|e| println!("{e}"));
                return Ok(Response::new(StartResponse {
                    success: false,
                    message: error.to_string(),
                }));
            }
        };

        println!("Starting on {service_address}");

        let handle = tokio::runtime::Handle::current();

        // start service as a background task with a kill receiver
        tokio::spawn(async move {
            let mut background = handle.spawn(service.bind(service_address));

            match background.await {
                Ok(_) => {
                    println!("service stopped all on its own");
                    let _ = stopped_tx
                        .send((StopReason::End, String::new()))
                        .map_err(|e| println!("{e}"));
                }
                Err(error) => {
                    if error.is_panic() {
                        let panic = error.into_panic();
                        let msg = match panic.downcast_ref::<String>() {
                            Some(msg) => msg.to_string(),
                            None => match panic.downcast_ref::<&str>() {
                                Some(msg) => msg.to_string(),
                                None => "<no panic message>".to_string(),
                            },
                        };

                        println!("service panicked: {msg}");
                        let _ = stopped_tx
                            .send((StopReason::Crash, msg))
                            .map_err(|e| println!("{e}"));
                    } else {
                        println!("service crashed: {error}");
                        let _ = stopped_tx
                            .send((StopReason::Crash, error.to_string()))
                            .map_err(|e| println!("{e}"));
                    }
                }
            }
        });

        let message = StartResponse {
            success: true,
            ..Default::default()
        };

        *self.state.lock().unwrap() = State::Running;

        Ok(Response::new(message))
    }

    async fn stop(&self, _request: Request<StopRequest>) -> Result<Response<StopResponse>, Status> {
        let kill_tx = self.kill_tx.lock().unwrap().deref_mut().take();

        if let Some(kill_tx) = kill_tx {
            if kill_tx.send("stopping deployment".to_owned()).is_err() {
                println!("the kill receiver dropped");
                return Err(Status::internal("failed to stop deployment"));
            }

            Ok(Response::new(StopResponse { success: true }))
        } else {
            println!("failed to stop deployment");

            Ok(tonic::Response::new(StopResponse { success: false }))
        }
    }

    type SubscribeStopStream = ReceiverStream<Result<SubscribeStopResponse, Status>>;

    async fn subscribe_stop(
        &self,
        _request: Request<SubscribeStopRequest>,
    ) -> Result<Response<Self::SubscribeStopStream>, Status> {
        let mut stopped_rx = self.stopped_tx.subscribe();
        let (tx, rx) = mpsc::channel(1);

        // Move the stop channel into a stream to be returned
        tokio::spawn(async move {
            while let Ok((reason, message)) = stopped_rx.recv().await {
                tx.send(Ok(SubscribeStopResponse {
                    reason: reason as i32,
                    message,
                }))
                .await
                .unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn health_check(&self, _request: Request<Ping>) -> Result<Response<Pong>, Status> {
        if matches!(self.state.lock().unwrap().deref(), State::Unhealthy) {
            println!("runtime health check failed");
            return Err(Status::unavailable(
                "runtime has not reached a healthy state",
            ));
        }

        Ok(Response::new(Pong {}))
    }
}
