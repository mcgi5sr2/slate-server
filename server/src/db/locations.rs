// Database queries for locations
// Handles the Sqlite references and returns model types

use crate::models::Location;
use sqlx::SqlitePool;

// Fetch all locations from the databse.
// returns a Vec of Location - empty Vec if non exists
pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Location>, sqlx::Error> {
    // sqlx::query_as! maps each row to Location struct
    // macro checks at compile time that SQL is valid, and matches struct and fields
    let locations = sqlx::query_as!(Location, "SELECT id, name FROM locations ORDER BY name")
        .fetch_all(pool)
        .await?;

    Ok(locations)
}

// Fetch a single location by id, returns none if no location with that id
pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Location>, sqlx::Error> {
    let location = sqlx::query_as!(Location, "SELECT id, name FROM locations WHERE id = ?", id)
        .fetch_optional(pool)
        .await?;

    Ok(location)
}

// Insert new location, returns created Location for handler to return to caller
pub async fn create(pool: &SqlitePool, id: &str, name: &str) -> Result<Location, sqlx::Error> {
    sqlx::query!("INSERT INTO locations (id, name) VALUES (?, ?)", id, name)
        .execute(pool)
        .await?;

    // return the new location
    get_by_id(pool, id)
        .await
        .map(|opt| opt.expect("Location was just inserted, must exist"))
}

// Delete location, associated playlist items also removed via ON DLETE CASCADE
pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM locations WHERE id = ?", id)
        .execute(pool)
        .await?;

    // rows_affected() tells us if a row is deleted, else return sfalse
    Ok(result.rows_affected() > 0)
}
