use reqwest::StatusCode;
use shuttle_common_tests::cargo_shuttle::cargo_shuttle_run;

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
    std::fs::copy(
        "../examples/rocket/secrets/Secrets.toml.example",
        "../examples/rocket/secrets/Secrets.toml",
    )
    .unwrap();

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

    assert!(request_text.contains("This is an example of serving"));
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
        r#"{"message":"This endpoint is open to anyone"}"#
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
        .body(r#"{"username": "username", "password": "password"}"#)
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
        r#"{"message":"The `Claims` request guard ensures only valid JWTs can access this endpoint","user":"username"}"#
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

// These examples use a shared Postgres/Mongo. Thus local runs should create a docker containers.
mod needs_docker {
    use super::*;

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
}
