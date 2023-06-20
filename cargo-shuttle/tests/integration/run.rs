use cargo_shuttle::{Command, ProjectArgs, RunArgs, Shuttle, ShuttleArgs};
use portpicker::pick_unused_port;
use reqwest::StatusCode;
use std::{fs::canonicalize, process::exit, time::Duration};
use tokio::time::sleep;

/// creates a `cargo-shuttle` run instance with some reasonable defaults set.
async fn cargo_shuttle_run(working_directory: &str, external: bool) -> String {
    let working_directory = match canonicalize(working_directory) {
        Ok(wd) => wd,
        Err(e) => {
            // DEBUG CI (no such file): SLEEP AND TRY AGAIN?
            println!(
                "Did not find directory: {} !!! because {:?}",
                working_directory, e
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            canonicalize(working_directory).unwrap()
        }
    };

    let port = pick_unused_port().unwrap();

    let url = if !external {
        format!("http://localhost:{port}")
    } else {
        format!("http://0.0.0.0:{port}")
    };

    let run_args = RunArgs {
        port,
        external,
        release: false,
    };

    let runner = Shuttle::new().unwrap().run(ShuttleArgs {
        api_url: Some("http://shuttle.invalid:80".to_string()),
        project_args: ProjectArgs {
            working_directory: working_directory.clone(),
            name: None,
        },
        cmd: Command::Run(run_args),
    });

    tokio::spawn({
        let working_directory = working_directory.clone();
        async move {
            sleep(Duration::from_secs(10 * 60)).await;

            println!(
                "run test for '{}' took too long. Did it fail to shutdown?",
                working_directory.display()
            );
            exit(1);
        }
    });

    let runner_handle = tokio::spawn(runner);

    // Wait for service to be responsive
    let mut counter = 0;
    let client = reqwest::Client::new();
    while client.get(url.clone()).send().await.is_err() {
        if runner_handle.is_finished() {
            println!(
                "run test for '{}' exited early. Did it fail to compile/run?",
                working_directory.clone().display()
            );
            exit(1);
        }

        // reduce spam
        if counter == 0 {
            println!(
                "waiting for '{}' to start up...",
                working_directory.display()
            );
        }
        counter = (counter + 1) % 10;

        sleep(Duration::from_millis(500)).await;
    }

    url
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn rocket_hello_world() {
    let url = cargo_shuttle_run("../examples/rocket/hello-world", false).await;

    let request_text = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello, world!");
}

#[tokio::test(flavor = "multi_thread")]
async fn rocket_secrets() {
    let url = cargo_shuttle_run("../examples/rocket/secrets", false).await;

    let request_text = reqwest::Client::new()
        .get(format!("{url}/secret"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "the contents of my API key");
}

// This example uses a shared Postgres. Thus local runs should create a docker container for it.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn rocket_postgres() {
    let url = cargo_shuttle_run("../examples/rocket/postgres", false).await;
    let client = reqwest::Client::new();

    let post_text = client
        .post(format!("{url}/todo"))
        .body("{\"note\": \"Deploy to shuttle\"}")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(post_text, "{\"id\":1,\"note\":\"Deploy to shuttle\"}");

    let request_text = client
        .get(format!("{url}/todo/1"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "{\"id\":1,\"note\":\"Deploy to shuttle\"}");
}

#[tokio::test(flavor = "multi_thread")]
async fn axum_static_files() {
    let url = cargo_shuttle_run("../examples/axum/static-files", false).await;
    let client = reqwest::Client::new();

    let request_text = client
        .get(url.clone())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello, world!");

    let request_text = client
        .get(format!("{url}/assets"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(
        request_text.contains("This is an example of serving static files with axum and shuttle.")
    );
}

// note: you need `rustup target add wasm32-wasi` to make this project compile
#[tokio::test(flavor = "multi_thread")]
async fn shuttle_next() {
    let url = cargo_shuttle_run("../examples/next/hello-world", false).await;
    let client = reqwest::Client::new();

    let request_text = client.get(&url).send().await.unwrap().text().await.unwrap();

    assert_eq!(request_text, "Hello, World!");

    let post_text = client
        .post(format!("{url}/uppercase"))
        .body("uppercase this")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(post_text, "UPPERCASE THIS");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn rocket_authentication() {
    let url = cargo_shuttle_run("../examples/rocket/authentication", false).await;
    let client = reqwest::Client::new();

    let public_text = client
        .get(format!("{url}/public"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(
        public_text,
        "{\"message\":\"This endpoint is open to anyone\"}"
    );

    let private_status = client
        .get(format!("{url}/private"))
        .send()
        .await
        .unwrap()
        .status();

    assert_eq!(private_status, StatusCode::FORBIDDEN);

    let body = client
        .post(format!("{url}/login"))
        .body("{\"username\": \"username\", \"password\": \"password\"}")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let token = format!("Bearer  {}", json["token"].as_str().unwrap());

    let private_text = client
        .get(format!("{url}/private"))
        .header("Authorization", token)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(
        private_text,
        "{\"message\":\"The `Claims` request guard ensures only valid JWTs can access this endpoint\",\"user\":\"username\"}"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn actix_web_hello_world() {
    let url = cargo_shuttle_run("../examples/actix-web/hello-world", false).await;

    let request_text = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello World!");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn axum_hello_world() {
    let url = cargo_shuttle_run("../examples/axum/hello-world", false).await;

    let request_text = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello, world!");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn tide_hello_world() {
    let url = cargo_shuttle_run("../examples/tide/hello-world", false).await;

    let request_text = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello, world!");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn tower_hello_world() {
    let url = cargo_shuttle_run("../examples/tower/hello-world", false).await;

    let request_text = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello, world!");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn warp_hello_world() {
    let url = cargo_shuttle_run("../examples/warp/hello-world", false).await;

    let request_text = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello, World!");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn poem_hello_world() {
    let url = cargo_shuttle_run("../examples/poem/hello-world", false).await;

    let request_text = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello, world!");
}

// This example uses a shared Postgres. Thus local runs should create a docker container for it.
#[tokio::test(flavor = "multi_thread")]
async fn poem_postgres() {
    let url = cargo_shuttle_run("../examples/poem/postgres", false).await;
    let client = reqwest::Client::new();

    let post_text = client
        .post(format!("{url}/todo"))
        .body("{\"note\": \"Deploy to shuttle\"}")
        .header("content-type", "application/json")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(post_text, "{\"id\":1,\"note\":\"Deploy to shuttle\"}");

    let request_text = client
        .get(format!("{url}/todo/1"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "{\"id\":1,\"note\":\"Deploy to shuttle\"}");
}

// This example uses a shared MongoDb. Thus local runs should create a docker container for it.
#[tokio::test(flavor = "multi_thread")]
async fn poem_mongodb() {
    let url = cargo_shuttle_run("../examples/poem/mongodb", false).await;
    let client = reqwest::Client::new();

    // Post a todo note and get the persisted todo objectId
    let post_text = client
        .post(format!("{url}/todo"))
        .body("{\"note\": \"Deploy to shuttle\"}")
        .header("content-type", "application/json")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    // Valid objectId is 24 char hex string
    assert_eq!(post_text.len(), 24);

    let request_text = client
        .get(format!("{url}/todo/{post_text}"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "{\"note\":\"Deploy to shuttle\"}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn salvo_hello_world() {
    let url = cargo_shuttle_run("../examples/salvo/hello-world", false).await;

    let request_text = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello, world!");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn thruster_hello_world() {
    let url = cargo_shuttle_run("../examples/thruster/hello-world", false).await;

    let request_text = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello, World!");
}

#[tokio::test(flavor = "multi_thread")]
async fn rocket_hello_world_with_router_ip() {
    let url = cargo_shuttle_run("../examples/rocket/hello-world", true).await;

    let request_text = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(request_text, "Hello, world!");
}
