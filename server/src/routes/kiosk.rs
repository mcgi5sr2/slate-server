// serves the kiosk display page for location
// GET /kiosk/{location_id} returns kiosk.html as HTML
// location_id read by JS from url path
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
};
use tokio::fs;
use crate::store::AppState;

pub async fn serve(State(state): State<AppState>) -> impl IntoResponse {
    //read kiosk.html from /static/
    let path = format!("{}/kiosk.html", state.static_dir);
    match fs::read_to_string(&path).await {
        Ok(html) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            html,
        ).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}