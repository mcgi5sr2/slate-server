use axum::{Router, routing::get};
use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use tower_http::services::ServeDir;

mod models;
mod store;
mod db;
mod routes;

// export AppState for route handler access
use store::AppState;

#[tokio::main]
async fn main() {
    // load .env file via dotenvy, ok() means no panic if no .env
    // in Docker, this will be handled by secrets
    dotenvy::dotenv().ok();

    // DATABASE_URL from .env
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be in the .env or environment");

    // Read UPLOADS_DIR, file served as static asses not stored in database
    let uploads_dir = env::var("UPLOADS_DIR")
        .expect("UPLOADS_DIR must be set in the .env or environment");

    // Create a connection pool for sqlx, as epecting multiple queries
    // hardcoded 5 concurrent writers on the server (Kiosks only READ)
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await.expect("Failed to connect to SQLite database");

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
    let state = AppState::new(pool);

    // Build the router, /health and /uploads/*path to serve files
    // nest_service is a service that maps URL paths to files on disk
    // with_state allows any handler to request state from the axum route
    let app = Router::new()
        .route("/health", get(health))
        .nest_service("/uploads", ServeDir::new(&uploads_dir))
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
