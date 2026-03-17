// HTTP handlers for playlist endpoints.
// GET  /api/playlist/:location_id — fetch playlist for a kiosk (with ETag support)
// POST /api/playlist/:location_id — replace a location's playlist

use crate::db::playlists;
use crate::models::PlaylistItem;
use crate::store::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

// POST body shape, a list of items in play order
#[derive(Deserialize)]
pub struct SetPlaylistRequest {
    pub items: Vec<PlaylistItem>,
}

// GET /api/playlist/{location_id} returns the playlist as JSON
// returns json playlist, ETag-based caching
pub async fn get(
    State(state): State<AppState>,
    Path(location_id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let playlist = match playlists::get_playlist(&state.db, &location_id).await {
        Ok(p) => p,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // serialise playlist to JSON string
    let json = match serde_json::to_string(&playlist) {
        Ok(j) => j,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    //SHA256 of JSON string
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let etag = format!("\"{}\"", hex::encode(hasher.finalize()));

    // If client sent If-Non-Match header
    if headers
        .get("if-none-match")
        .is_some_and(|v| v.as_bytes() == etag.as_bytes())
    {
        return StatusCode::NOT_MODIFIED.into_response();
    }

    // Playlist chage or no ETag, return 200 w. body and ETag header
    (StatusCode::OK, [("ETag", etag)], Json(playlist)).into_response()
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
