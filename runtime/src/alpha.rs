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
use shuttle_common::{extract_propagation::ExtractPropagationLayer, secrets::Secret};
use shuttle_proto::runtime::{
    runtime_server::{Runtime, RuntimeServer},
    LoadRequest, LoadResponse, Ping, Pong, StartRequest, StartResponse, StopReason, StopRequest,
    StopResponse, SubscribeStopRequest, SubscribeStopResponse, VersionInfo,
};
use shuttle_service::{ResourceFactory, Service};
use tokio::sync::{
    broadcast::{self, Sender},
    mpsc, oneshot,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

use crate::__internals::{Loader, Runner};

pub async fn start(
    port: u16,
    loader: impl Loader + Send + 'static,
    runner: impl Runner + Send + 'static,
) {
    // where to serve the gRPC control layer
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);

    let mut server_builder = Server::builder()
        .http2_keepalive_interval(Some(Duration::from_secs(60)))
        .layer(ExtractPropagationLayer);

    let router = {
        let alpha = Alpha::new(loader, runner);

        let svc = RuntimeServer::new(alpha);
        server_builder.add_service(svc)
    };

    match router.serve(addr).await {
        Ok(_) => println!("router completed on its own"),
        Err(e) => panic!("Error while serving address {addr}: {e}"),
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
    kill_tx: Mutex<Option<oneshot::Sender<String>>>,
    loader: Mutex<Option<L>>,
    runner: Mutex<Option<R>>,
    /// The current state of the runtime, which is used by the ECS task to determine if the runtime
    /// is healthy.
    state: Arc<Mutex<State>>,
}

impl<L, R> Alpha<L, R> {
    pub fn new(loader: L, runner: R) -> Self {
        let (stopped_tx, _stopped_rx) = broadcast::channel(10);

        Self {
            stopped_tx,
            kill_tx: Mutex::new(None),
            loader: Mutex::new(Some(loader)),
            runner: Mutex::new(Some(runner)),
            state: Arc::new(Mutex::new(State::Unhealthy)),
        }
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

        *self.state.lock().unwrap() = State::Loading;

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

        let (kill_tx, kill_rx) = tokio::sync::oneshot::channel();
        *self.kill_tx.lock().unwrap() = Some(kill_tx);

        let handle = tokio::runtime::Handle::current();

        // start service as a background task with a kill receiver
        tokio::spawn(async move {
            let mut background = handle.spawn(service.bind(service_address));

            tokio::select! {
                res = &mut background => {
                    match res {
                        Ok(_) => {
                            println!("service stopped all on its own");
                            let _ = stopped_tx
                                .send((StopReason::End, String::new()))
                                .map_err(|e| println!("{e}"));
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
                        },
                    }
                },
                message = kill_rx => {
                    match message {
                        Ok(_) => {
                            let _ = stopped_tx
                                .send((StopReason::Request, String::new()))
                                .map_err(|e| println!("{e}"));
                        }
                        Err(_) => println!("the kill sender dropped")
                    };

                    println!("will now abort the service");
                    background.abort();
                    background.await.unwrap().expect("to stop service");
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

    async fn version(&self, _requset: Request<Ping>) -> Result<Response<VersionInfo>, Status> {
        Ok(Response::new(VersionInfo {
            version: crate::VERSION_STRING.to_owned(),
        }))
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
