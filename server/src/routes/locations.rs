// HTTP handlers for location endpoints
// POST /api/locations     — create a location
// GET  /api/locations     — list all locations
// GET  /api/locations/{id} — get a single location
// DELETE /api/locations/{id} — delete a location

use crate::db::locations;
use crate::store::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

// POST /api/locations body shape
#[derive(Deserialize)]
pub struct CreateLocationRequest {
    pub id: String,
    pub name: String,
}

// GET /api/locations - returns all locations as JSON
pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    match locations::get_all(&state.db).await {
        Ok(locs) => (StatusCode::OK, Json(locs)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// GET /api/locations/{id} - return single location or 404
pub async fn get(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match locations::get_by_id(&state.db, &id).await {
        Ok(Some(loc)) => (StatusCode::OK, Json(loc)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// POST /api/locations - create a new location
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateLocationRequest>,
) -> impl IntoResponse {
    match locations::create(&state.db, &body.id, &body.name).await {
        Ok(loc) => (StatusCode::CREATED, Json(loc)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// DELETE /api/locations/{id} — delete location, returns 204 or 404
pub async fn delete(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match locations::delete(&state.db, &id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
