use axum::{extract::State, Json};
use std::sync::Arc;

use crate::app::AppState;
use crate::controller::response::ParcelResponse;

#[utoipa::path(
    get,
    path = "/parcel",
    responses(
        (status = 200, description = "All stub parcels", body = [ParcelResponse]),
    ),
)]
pub async fn list_parcels(State(state): State<Arc<AppState>>) -> Json<Vec<ParcelResponse>> {
    Json(
        state
            .parcels
            .list_parcels()
            .into_iter()
            .map(ParcelResponse::from)
            .collect(),
    )
}
