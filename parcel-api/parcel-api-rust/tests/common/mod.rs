#![allow(dead_code)]

use std::sync::Arc;

use parcel_api::{app::AppState, config::AppConfig, service::StubParcelService};

pub struct TestApp {
    pub state: Arc<AppState>,
}

impl TestApp {
    pub fn new() -> Self {
        let config = AppConfig::load_with_profile("__nope__");
        let state = Arc::new(AppState {
            parcels: StubParcelService::load(&config.parcel_data_dir),
            config: config.clone(),
        });
        Self { state }
    }

    pub fn router(&self) -> axum::Router {
        parcel_api::build_app(self.state.clone())
    }
}

pub async fn body_to_string(body: axum::body::Body) -> String {
    use http_body_util::BodyExt;
    let collected = body.collect().await.expect("collect body");
    String::from_utf8(collected.to_bytes().to_vec()).expect("utf8 body")
}
