// upload handler
// POST /api/upload accepts multipart form, with name slug and file
// save to {UPLOADS_DIR}/{location_id}/{name}.{ext}, overwrite if exists. name slug is what decides if overwritten
// Returns the public URL the file can be accessed at

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs;
use crate::store::AppState;

//Response body the URL the uploaded fi8le can be accessed at
#[derive(Serialize)]
pub struct UploadResponse{
    pub url: String,
}

//POST /api/upload
// expects: 1location_id1, `name` (slug) and `file` 
pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut location_id: Option<String> = None;
    let mut name: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut extension: Option<String> = None;

    //iterate over the multipart fields
    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("location_id") => {
                location_id = Some(field.text().await.unwrap_or_default());
            }
            Some("name") => {
                //read the slug
                name = Some(field.text().await.unwrap_or_default());
            }
            Some("file") => {
                //get the file extension from og filename
                extension = field
                    .file_name()
                    .and_then(|f| std::path::Path::new(f).extension())
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase());

                // read the file bytes
                file_bytes = field.bytes().await.ok().map(|b| b.to_vec());
            }
            _=> {} // ignore unknown fileds
        }
    }

    // check we got both fields
    let (Some(location_id), Some(name), Some(bytes), Some(ext)) =
        (location_id, name, file_bytes, extension) else {
        return (StatusCode::BAD_REQUEST, "Missing location_id, name, or file field").into_response();
    };

    // fix up the location_id name slug to only allow normal chars
    if !location_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return (StatusCode::BAD_REQUEST, "Invalid location_id").into_response();
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return (StatusCode::BAD_REQUEST, "Invalid name: use only letters, numbers, hyphens, underscores").into_response();
    }

    // create the dir path: {uploads_dir}/{location_id}/
    let dir: PathBuf = PathBuf::from(&state.uploads_dir).join(&location_id);

    // create the dir if its not there
    if fs::create_dir_all(&dir).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

        // create the file path: {uploads_dir}/{location_id}/{name}.{ext}
    let filename = format!("{}.{}", name, ext);
    let path = dir.join(&filename);

    // write it or overwrite if slug name exists
    if fs::write(&path, bytes).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // Return the public URL
    let url = format!("/uploads/{}/{}", location_id, filename);
    (StatusCode::OK, Json(UploadResponse { url })).into_response()

}
