// HTTP handlers for playlist endpoints.
// GET  /api/playlist/:location_id — fetch playlist for a kiosk (with ETag support)
// POST /api/playlist/:location_id — replace a location's playlist

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use crate::store::AppState;
use crate::db::playlists;
use crate::models::PlaylistItem;

// POST body shape, a list of items in play order
#[derive(Deserialize)]
pub struct SetPlaylistRequest {
    pub items: Vec<PlaylistItem>,
}

// GET /api/playlist/{location_id} returns the playlist as JSON
pub async fn get(
    State(state): State<AppState>,
    Path(location_id): Path<String>,
) -> impl IntoResponse {
    match playlists::get_playlist(&state.db, &location_id).await {
        Ok(playlist) => (StatusCode::OK, Json(playlist)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// POST /api/playlist/{location_id} - replace the playlist
pub async fn set(
    State(state): State<AppState>,
    Path(location_id): Path<String>,
    Json(body): Json<SetPlaylistRequest>,
) -> impl IntoResponse {
    match playlists::set_playlist(&state.db, &location_id, body.items).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}