mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{body_to_string, TestApp};
use tower::util::ServiceExt;

#[tokio::test]
async fn post_parcel_returns_200_with_100_parcels() {
    let test_app = TestApp::new();
    let req = Request::builder()
        .method("POST")
        .uri("/parcel-api/v1/parcel")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let response = test_app.router().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_to_string(response.into_body()).await;
    let parcels: serde_json::Value = serde_json::from_str(&body).expect("valid json");
    let parcels = parcels.as_array().unwrap();
    assert_eq!(parcels.len(), 100);
    assert!(parcels
        .iter()
        .any(|parcel| parcel["parcelNumber"] == "TESTPARCEL0001000000"));
}

#[tokio::test]
async fn post_parcel_accepts_populated_request_body() {
    let test_app = TestApp::new();
    let req = Request::builder()
        .method("POST")
        .uri("/parcel-api/v1/parcel")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"lastUpdated":"2026-05-15T10:00:00Z","exclude":["TESTPARCEL0001000000"]}"#,
        ))
        .unwrap();
    let response = test_app.router().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
