// upload handler
// POST /api/upload accepts multipart form, with name slug and file
// save to {UPLOADS_DIR}/{location_id}/{name}.{ext}, overwrite if exists. name slug is what decides if overwritten
// Returns the public URL the file can be accessed at

use crate::store::AppState;
use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs;

//Response body the URL the uploaded fi8le can be accessed at
#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,                    // empty for pdfs
    pub page_urls: Option<Vec<String>>, // for pdfs
    pub file_type: String,              //"image", "video", or "pdf"
}

//POST /api/upload
// expects: 1location_id1, `name` (slug) and `file`
pub async fn upload(State(state): State<AppState>, mut multipart: Multipart) -> impl IntoResponse {
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
            _ => {} // ignore unknown fileds
        }
    }

    // check we got both fields
    let (Some(location_id), Some(name), Some(bytes), Some(ext)) =
        (location_id, name, file_bytes, extension)
    else {
        return (
            StatusCode::BAD_REQUEST,
            "Missing location_id, name, or file field",
        )
            .into_response();
    };

    // if its pdf rend to PNG
    if ext == "pdf" {
        return match render_pdf(&state.uploads_dir, &location_id, &name, bytes).await {
            Ok(page_urls) => (StatusCode::OK, Json(UploadResponse {
                url: String::new(),
                page_urls: Some(page_urls),
                file_type: "pdf".to_string(),
            })).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,e).into_response(),
        };
    }

    // fix up the location_id name slug to only allow normal chars
    if !location_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return (StatusCode::BAD_REQUEST, "Invalid location_id").into_response();
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid name: use only letters, numbers, hyphens, underscores",
        )
            .into_response();
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
    (StatusCode::OK, Json(UploadResponse { 
        url,
        page_urls: None,
        file_type: ext.to_string(),
     })).into_response()
}

// Render PDF to PNG images with pdftoppm
// {uploads}/{location_id}/pdf-{name}/page-{n}.png
// Returns a Vec of public URLS for each page, or err string
pub async fn render_pdf(
    uploads_dir: &str,
    location_id: &str,
    name: &str,
    pdf_bytes: Vec<u8>,
) -> Result<Vec<String>, String> {
    // create output path
    let output_dir = PathBuf::from(uploads_dir)
        .join(location_id)
        .join(format!("pdf-{}", name));

    // create dir if needs be
    fs::create_dir_all(&output_dir)
    .await
    .map_err(|e| format!("Failed to create output dir: {}",e))?;

    //write the PDF bytes to a temp file for pdftoppm
    let pdf_path = output_dir.join("source.pdf");
    fs::write(&pdf_path, &pdf_bytes)
        .await
        .map_err(|e| format!("Failed to write PDF: {}", e))?;

    //make the out prefix for pdftoppm to append page numbers
    let output_prefix = output_dir.join("page").to_string_lossy().to_string();

    // pdftoppm render all pages to png
    // -r 150: resolution in DPI
    let output = tokio::process::Command::new("pdftoppm")
        .arg("-png")
        .arg("-r").arg("150")
        .arg(pdf_path.to_str().unwrap())
        .arg(&output_prefix)
        .output()
        .await
        .map_err(|e| format!("Failed to run pdftoppm: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pdftoppm failed: {}", stderr));
    }

    //Read output dir for PNG files
    // pdftoppm names them page-1.png etcetc
    let mut entries = fs::read_dir(&output_dir)
        .await
        .map_err(|e| format!("Failed to read output dir: {}", e))?;

    let mut pages: Vec<String> = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        // PNG files, skip source pdf
        if path.extension().and_then(|e| e.to_str()) == Some("png") {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            pages.push(format!("/uploads/{}/pdf-{}/{}", location_id, name, filename));
        }
    }

    //sort pages by filename
    pages.sort();

    Ok(pages)
}