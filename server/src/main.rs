use axum::{
    Router,
    routing::{get, post},
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::env;
use std::str::FromStr;
use tower_http::services::ServeDir;

mod db;
mod models;
mod routes;
mod store;

// export AppState for route handler access
use store::AppState;

#[tokio::main]
async fn main() {
    // load .env file via dotenvy, ok() means no panic if no .env
    // in Docker, this will be handled by secrets
    dotenvy::dotenv().ok();

    // DATABASE_URL from .env
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be in the .env or environment");

    // UPLOADS_DIR from .env, file served as static assets not stored in database
    let uploads_dir =
        env::var("UPLOADS_DIR").expect("UPLOADS_DIR must be set in the .env or environment");

    // webpage path from .env
    // STATIC_DIR from .env, serves the admin UI
    let static_dir =
        env::var("STATIC_DIR").expect("STATIC_DIR must be set in the .env or environment");

    // Parse connection options from DATABASE_URL and enable file creation.
    // By default sqlx will NOT create the .db file — create_if_missing(true) is required.
    let connect_options = SqliteConnectOptions::from_str(&database_url)
        .expect("Invalid DATABASE_URL format")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .expect("Failed to connect to SQLite database");

    // Enable WAL mode so the database can be read while it writes
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(&pool)
        .await
        .expect("Failed to enable WAL mode");

    // Run any pending migrations on startup.
    // ../migrations goes from server/ up to the workspace root.
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    // AppState for easy cloning of the DB pool for each request handler via Arc
    let state = AppState::new(pool, uploads_dir.clone());

    // Build the router, /health and /uploads/*path to serve files
    // nest_service is a service that maps URL paths to files on disk
    // with_state allows any handler to request state from the axum route
    let app = Router::new()
        .route("/health", get(health))
        //location routes
        .route(
            "/api/locations",
            get(routes::locations::list).post(routes::locations::create),
        )
        .route(
            "/api/locations/{id}",
            get(routes::locations::get).delete(routes::locations::delete),
        )
        .route(
            "/api/playlist/{location_id}",
            get(routes::playlists::get).post(routes::playlists::set),
        )
        .route("/api/upload", post(routes::upload::upload))
        // nest_service for kiosk to fetch
        .nest_service("/uploads", ServeDir::new(&uploads_dir))
        // nest_service for the admin static files
        .nest_service("/admin", ServeDir::new(&static_dir))
        .with_state(state);

    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}
