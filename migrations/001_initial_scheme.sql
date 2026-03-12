CREATE TABLE locations (
    id   TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);
-- JSON package for each item in playlist
CREATE TABLE playlist_items (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    location_id  TEXT    NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    position     INTEGER NOT NULL,
    item_type    TEXT    NOT NULL,
    data         TEXT    NOT NULL  -- JSON, structure depends on item_type
);

CREATE INDEX idx_playlist_items_location ON playlist_items(location_id, position);
