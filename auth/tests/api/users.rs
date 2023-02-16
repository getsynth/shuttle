use crate::helpers::{app, ADMIN_KEY};
use axum::body::Body;
use hyper::http::{header::AUTHORIZATION, Request, StatusCode};
use serde_json::{self, Value};

#[tokio::test]
async fn post_user() {
    let app = app().await;

    // POST user without bearer token.
    let request = Request::builder()
        .uri("/user/test-user")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.send_request(request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // POST user with invalid bearer token.
    let request = Request::builder()
        .uri("/user/test-user")
        .method("POST")
        .header(AUTHORIZATION, format!("Bearer notadmin"))
        .body(Body::empty())
        .unwrap();

    let response = app.send_request(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // POST user with valid bearer token.
    let response = app.post_user("test-user").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let user: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(user["name"], "test-user");
    assert!(user["key"].to_string().is_ascii());
}

#[tokio::test]
async fn get_user() {
    let app = app().await;

    // POST user first so one exists in the database.
    let response = app.post_user("test-user").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let user: Value = serde_json::from_slice(&body).unwrap();

    // GET user without bearer token.
    let request = Request::builder()
        .uri("/user/test-user")
        .body(Body::empty())
        .unwrap();

    let response = app.send_request(request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // GET user with invalid bearer token.
    let request = Request::builder()
        .uri("/user/test-user")
        .header(AUTHORIZATION, format!("Bearer notadmin"))
        .body(Body::empty())
        .unwrap();

    let response = app.send_request(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // GET user that doesn't exist with valid bearer token.
    let request = Request::builder()
        .uri("/user/notfound")
        .header(AUTHORIZATION, format!("Bearer {ADMIN_KEY}"))
        .body(Body::empty())
        .unwrap();

    let response = app.send_request(request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // GET user with valid bearer token.
    let response = app.get_user("test-user").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let persisted_user: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(user, persisted_user);
}
