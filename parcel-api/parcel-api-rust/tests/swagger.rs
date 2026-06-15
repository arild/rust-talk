mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{body_to_string, TestApp};
use tower::util::ServiceExt;

#[tokio::test]
async fn swagger_ui_index_is_reachable() {
    let test_app = TestApp::new();
    let req = Request::builder()
        .uri("/parcel-api/swagger-ui/")
        .body(Body::empty())
        .unwrap();
    let response = test_app.router().oneshot(req).await.unwrap();
    let status = response.status();
    assert!(
        status == StatusCode::OK || status.is_redirection(),
        "expected 200 or redirect, got {status}"
    );
}

#[tokio::test]
async fn openapi_json_is_reachable() {
    let test_app = TestApp::new();
    let req = Request::builder()
        .uri("/parcel-api/v3/api-docs")
        .body(Body::empty())
        .unwrap();
    let response = test_app.router().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_to_string(response.into_body()).await;
    let value: serde_json::Value = serde_json::from_str(&body).expect("valid openapi json");
    assert_eq!(value["info"]["title"], "Parcel API");
    assert!(value["paths"]["/v1/parcel"].is_object());
}
