// Database queries for playlists and playlist items
// Handles the Sqlite references and returns model types
// The 'data' column stores items in JSON, deserialise these  into PlaylistItem variants

use crate::models::{Playlist, PlaylistItem};
use sqlx::SqlitePool;

// Feth all playlist items for a location, ordered by position
pub async fn get_playlist(
    pool: &SqlitePool,
    location_id: &str,
) -> Result<Playlist, sqlx::Error> {
    // Fetch rows - item_type and data as strings
    let rows = sqlx::query!(
        "SELECT item_type, data FROM playlist_items WHERE location_id = ? ORDER BY position",
        location_id
    )
    .fetch_all(pool)
    .await?;

    // Deserialise each row's JSON data blob into the correct PlaylistItem vairant.
    // item_type s  build a tagged JSON object for serde deserialise.
    // item_type="image", data={"url":"...","duration_secs":10}
    // becomes {"type":"image","url":"...","duraction_secs":10}
    let items: Vec<PlaylistItem> = rows
        .into_iter()
        .filter_map(|row| {
            let mut value: serde_json::Value = serde_json::from_str(&row.data).ok()?;
            value["type"] = serde_json::Value::String(row.item_type);
            serde_json::from_value(value).ok()
        })
        .collect();

    Ok(Playlist {
        location_id: location_id.to_string(),
        items,
    })
}


// Replace a location's entire playlist.
// Deletes all existing items then inserts the new ones in order.
// Wrapped in a transaction so it's all-or-nothing.
pub async fn set_playlist(
    pool: &SqlitePool,
    location_id: &str,
    items: Vec<PlaylistItem>,
) -> Result<(), sqlx::Error> {
    // Begin a transaction — if anything fails, the whole thing rolls back
    let mut tx = pool.begin().await?;

    // Delete existing items for this location
    sqlx::query!("DELETE FROM playlist_items WHERE location_id = ?", location_id)
        .execute(&mut *tx)
        .await?;

    // Insert each new item with its position index
    for (position, item) in items.iter().enumerate() {
        // Get the type tag string from the enum variant
        let item_type = match &item {
            PlaylistItem::Url { .. }       => "url",
            PlaylistItem::Image { .. }     => "image",
            PlaylistItem::Video { .. }     => "video",
            PlaylistItem::Slideshow { .. } => "slideshow",
            PlaylistItem::Pdf { .. }       => "pdf",
        };

        // Serialise the item to JSON, then strip the "type" field —
        // we store it separately in item_type column to allow indexed queries
        let mut value = serde_json::to_value(item).expect("PlaylistItem serialisation failed");
        if let Some(obj) = value.as_object_mut() {
            obj.remove("type");
        }
        let data = value.to_string();
        let pos = position as i64;

        sqlx::query!(
            "INSERT INTO playlist_items (location_id, position, item_type, data) VALUES (?, ?, ?, ?)",
            location_id,
            pos,
            item_type,
            data
        )
        .execute(&mut *tx)
        .await?;
    }

    // Commit the transaction
    tx.commit().await?;
    Ok(())
}