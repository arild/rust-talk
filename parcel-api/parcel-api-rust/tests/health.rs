mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{body_to_string, TestApp};
use tower::util::ServiceExt;

#[tokio::test]
async fn status_endpoint_returns_200() {
    let test_app = TestApp::new();
    let req = Request::builder()
        .uri("/parcel-api/check/status")
        .body(Body::empty())
        .unwrap();
    let response = test_app.router().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_to_string(response.into_body()).await;
    assert!(body.contains("parcel-api is on air"));
}

#[tokio::test]
async fn health_endpoint_returns_200_with_banner_and_memory() {
    let test_app = TestApp::new();
    let req = Request::builder()
        .uri("/parcel-api/check")
        .body(Body::empty())
        .unwrap();
    let response = test_app.router().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_to_string(response.into_body()).await;
    assert!(body.contains("Memory:"));
    assert!(body.contains("Version:"));
}
