// The data base pool

// RwLock to allow multiple references to the same object
use sqlx::SqlitePool;

// App state is our memory store for all server data 
// Clone is derived as boilerplate, so we can clone the state for the request handler
// Arc makes this cheaper as it increments a counter, not copy the data
#[derive(Clone)]
pub struct AppState {
    // Database connection pool, all route handlers use this to query SQLite
    pub db: SqlitePool,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        AppState { db: pool }
    }
}
