use axum::{
    middleware::from_fn,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::controller::{health, parcel};
use crate::service::StubParcelService;

pub struct AppState {
    pub parcels: StubParcelService,
    pub config: AppConfig,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controller::parcel::list_parcels,
    ),
    components(
        schemas(
            crate::controller::response::ParcelResponse,
            crate::controller::response::ParcelStatusResponse,
            crate::controller::response::DirectionResponse,
            crate::controller::response::TransportResponse,
            crate::controller::response::DimensionsResponse,
            crate::controller::response::DeliveryResponse,
            crate::controller::response::DeliveryTimeResponse,
            crate::controller::response::DeliveryWindowResponse,
            crate::controller::response::SenderResponse,
            crate::controller::response::BrandingResponse,
            crate::controller::response::ImageResponse,
            crate::controller::response::LinkResponse,
            crate::controller::response::CustomerServiceResponse,
            crate::controller::response::RecipientResponse,
            crate::controller::response::EventResponse,
            crate::controller::response::FeatureResponse,
            crate::controller::response::ProductGroupResponse,
            crate::controller::response::ProductNameResponse,
            crate::controller::response::CustomsTaxResponse,
            crate::controller::response::ParcelContent,
            crate::controller::response::CustomsPrice,
            crate::controller::response::CustomsInformationRequirements,
            crate::controller::response::RewardsResponse,
            crate::controller::response::RewardsEarningResponse,
        )
    ),
    info(
        title = "Parcel API",
        description = "Stubbed parcel API used for the Rust-vs-JVM perf-comparison talk.",
        version = "demo",
    ),
)]
pub struct ApiDoc;

pub fn build_app(state: Arc<AppState>) -> Router {
    let context_path = state.config.context_path.clone();

    let request_log = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    let inner: Router<Arc<AppState>> = Router::new()
        .route("/v1/parcel", post(parcel::list_parcels))
        .route("/check", get(health::health))
        .route("/check/status", get(health::status))
        .layer(from_fn(request_id_middleware))
        .layer(request_log);

    let with_context: Router<Arc<AppState>> = if context_path.is_empty() || context_path == "/" {
        inner.merge(SwaggerUi::new("/swagger-ui").url("/v3/api-docs", ApiDoc::openapi()))
    } else {
        let swagger_ui_path = format!("{context_path}/swagger-ui");
        let openapi_path = format!("{context_path}/v3/api-docs");
        Router::<Arc<AppState>>::new()
            .nest(&context_path, inner)
            .merge(SwaggerUi::new(swagger_ui_path).url(openapi_path, ApiDoc::openapi()))
    };

    with_context.with_state(state)
}

/// Echoes `X-Request-Id` from the request onto the response, matching the JVM service.
async fn request_id_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let mut response = next.run(request).await;

    if let Some(id) = request_id {
        if let Ok(value) = axum::http::HeaderValue::from_str(&id) {
            response.headers_mut().insert("x-request-id", value);
        }
    }
    response
}
