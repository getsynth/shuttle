use axum_extra::extract::cookie::{self, Cookie};
use http::{Request, StatusCode};
use hyper::Body;
use serde_json::{json, Value};
use shuttle_common::claims::Claim;

use crate::helpers::app;

// This test is ignored because we are not currently using the session flow.
#[ignore]
#[tokio::test]
async fn session_flow() {
    let app = app().await;

    // Create test user
    let response = app.post_user("session-user", "basic").await;

    assert_eq!(response.status(), StatusCode::OK);

    // POST user login
    let body = serde_json::to_vec(&json! ({"account_name": "session-user"})).unwrap();
    let request = Request::builder()
        .uri("/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let response = app.send_request(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();

    let cookie = Cookie::parse(cookie).unwrap();

    assert_eq!(cookie.http_only(), Some(true));
    assert_eq!(cookie.same_site(), Some(cookie::SameSite::Strict));
    assert_eq!(cookie.secure(), Some(true));

    // Test converting the cookie to a JWT
    let request = Request::builder()
        .uri("/auth/session")
        .method("GET")
        .header("Cookie", cookie.stripped().to_string())
        .body(Body::empty())
        .unwrap();
    let response = app.send_request(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let convert: Value = serde_json::from_slice(&body).unwrap();
    let token = convert["token"].as_str().unwrap();

    let request = Request::builder()
        .uri("/public-key")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let response = app.send_request(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let public_key = hyper::body::to_bytes(response.into_body()).await.unwrap();

    let claim = Claim::from_token(token, &public_key).unwrap();

    assert_eq!(claim.sub, "session-user");

    // POST user logout
    let request = Request::builder()
        .uri("/logout")
        .method("POST")
        .header("Cookie", cookie.stripped().to_string())
        .body(Body::empty())
        .unwrap();
    let response = app.send_request(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    // Test cookie can no longer be converted to JWT
    let request = Request::builder()
        .uri("/auth/session")
        .method("GET")
        .header("Cookie", cookie.stripped().to_string())
        .body(Body::empty())
        .unwrap();
    let response = app.send_request(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
