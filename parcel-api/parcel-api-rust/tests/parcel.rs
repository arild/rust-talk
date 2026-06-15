mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{body_to_string, TestApp};
use tower::util::ServiceExt;

#[tokio::test]
async fn get_parcels_returns_200_with_100_parcels() {
    let test_app = TestApp::new();
    let req = Request::builder()
        .method("GET")
        .uri("/parcel-api/parcel")
        .body(Body::empty())
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
