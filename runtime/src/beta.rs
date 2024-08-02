use std::{
    collections::BTreeMap,
    convert::Infallible,
    iter::FromIterator,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process::exit,
};

use anyhow::Context;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use shuttle_api_client::ShuttleApiClient;
use shuttle_common::{
    resource::{ResourceInput, ResourceState, Type},
    secrets::Secret,
};
use shuttle_service::{Environment, ResourceFactory, Service, ShuttleResourceOutput};

use crate::__internals::{Loader, Runner};

struct BetaEnvArgs {
    /// Are we running in a Shuttle deployment?
    shuttle: bool,
    project_id: String,
    project_name: String,
    env: Environment,
    /// Address to open service on
    ip: IpAddr,
    /// Port to open service on
    port: u16,
    /// Port to open health check on
    healthz_port: Option<u16>,
    /// Where to reach the required Shuttle API endpoints (mainly for provisioning)
    api_url: String,
    /// Key for the API calls (if relevant)
    api_key: Option<String>,
}

impl BetaEnvArgs {
    /// Uses primitive parsing instead of clap for reduced dependency weight.
    /// # Panics
    /// if any required arg is missing or does not parse
    fn parse() -> Self {
        Self {
            shuttle: std::env::var("SHUTTLE").is_ok(),
            project_id: std::env::var("SHUTTLE_PROJECT_ID").expect("project id env var"),
            project_name: std::env::var("SHUTTLE_PROJECT_NAME").expect("project name env var"),
            env: std::env::var("SHUTTLE_ENV")
                .expect("shuttle environment env var")
                .parse()
                .expect("invalid shuttle environment"),
            ip: std::env::var("SHUTTLE_RUNTIME_IP")
                .expect("runtime ip env var")
                .parse()
                .expect("invalid ip"),
            port: std::env::var("SHUTTLE_RUNTIME_PORT")
                .expect("runtime port env var")
                .parse()
                .expect("invalid port"),
            healthz_port: std::env::var("SHUTTLE_HEALTHZ_PORT")
                .map(|s| s.parse().expect("invalid healthz port"))
                .ok(),
            api_url: std::env::var("SHUTTLE_API").expect("api url env var"),
            api_key: std::env::var("SHUTTLE_API_KEY").ok(),
        }
    }
}

pub async fn start(loader: impl Loader + Send + 'static, runner: impl Runner + Send + 'static) {
    let BetaEnvArgs {
        shuttle,
        project_id,
        project_name,
        env,
        ip,
        port,
        healthz_port,
        api_url,
        api_key,
    } = BetaEnvArgs::parse();

    let service_addr = SocketAddr::new(ip, port);
    let client = ShuttleApiClient::new(api_url, api_key, None);

    // start a health check server if requested
    if let Some(healthz_port) = healthz_port {
        tokio::task::spawn(async move {
            let make_service = make_service_fn(|_conn| async {
                Ok::<_, Infallible>(service_fn(|_req| async move {
                    Result::<Response<Body>, hyper::Error>::Ok(Response::new(Body::empty()))
                }))
            });
            let server = Server::bind(&SocketAddr::new(Ipv4Addr::LOCALHOST.into(), healthz_port))
                .serve(make_service);

            if let Err(e) = server.await {
                eprintln!("Health check error: {}", e);
                exit(200);
            }
        });
    }

    //
    // LOADING / PROVISIONING PHASE
    //
    println!("Loading resources...");

    let secrets: BTreeMap<String, String> = match client
        .get_secrets_beta(&project_id)
        .await
        .and_then(|r| serde_json::from_value(r.data).context("failed to deserialize secrets"))
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Runtime Secret Loading phase failed: {e}");
            exit(101);
        }
    };

    // Sort secrets by key
    let secrets = BTreeMap::from_iter(secrets.into_iter().map(|(k, v)| (k, Secret::new(v))));

    let factory = ResourceFactory::new(project_name, secrets.clone(), env);
    let mut resources = match loader.load(factory).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Runtime Loader phase failed: {e}");
            exit(111);
        }
    };

    // Fail early if any byte vec is invalid json
    let values = match resources
        .iter()
        .map(|bytes| {
            serde_json::from_slice::<ResourceInput>(bytes).context("deserializing resource input")
        })
        .collect::<anyhow::Result<Vec<_>>>()
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Runtime Provisioning phase failed: {e}");
            exit(121);
        }
    };

    for (bytes, shuttle_resource) in resources
        .iter_mut()
        .zip(values)
        // ignore non-Shuttle resource items
        .filter_map(|(bytes, value)| match value {
            ResourceInput::Shuttle(shuttle_resource) => Some((bytes, shuttle_resource)),
            ResourceInput::Custom(_) => None,
        })
    {
        // Secrets don't need to be requested here since we already got them above.
        if shuttle_resource.r#type == Type::Secrets {
            *bytes = serde_json::to_vec(&ShuttleResourceOutput {
                output: serde_json::to_value(&secrets).unwrap(),
                custom: serde_json::Value::Null,
                state: Some(ResourceState::Ready),
            })
            .expect("to serialize struct");
            continue;
        }
        println!("Provisioning {}", shuttle_resource.r#type);
        loop {
            match client
                .provision_resource_beta(&project_id, shuttle_resource.clone())
                .await
            {
                Ok(output) => match output.state.clone().expect("resource to have a state") {
                    ResourceState::Provisioning | ResourceState::Authorizing => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                    }
                    ResourceState::Ready => {
                        *bytes = serde_json::to_vec(&output).expect("to serialize struct");
                        break;
                    }
                    bad_state => {
                        eprintln!(
                            "Runtime Provisioning phase failed: Received '{}' resource with state '{}'.",
                            shuttle_resource.r#type,
                            bad_state
                        );
                        exit(132);
                    }
                },
                Err(e) => {
                    eprintln!("Runtime Provisioning phase failed: {e}");
                    exit(131);
                }
            };
        }
    }

    // TODO?: call API to say running state is being entered

    if shuttle {
        // Tell sidecar to shut down.
        // Ignore error, since the endpoint does not send a response.
        let _ = client.client.get("/__shuttle/shutdown").send().await;
    }

    //
    // RESOURCE INIT PHASE
    //

    let service = match runner.run(resources).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Runtime Resource Initialization phase failed: {e}");
            exit(151);
        }
    };

    //
    // RUNNING PHASE
    //
    println!("Starting service!");

    if let Err(e) = service.bind(service_addr).await {
        eprintln!("Service encountered an error in `bind`: {e}");
        exit(1);
    }
}
