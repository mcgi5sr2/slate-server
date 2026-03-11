-- locations table
-- stores each physical display location
-- the id is human-readable slug used by the kiosk as LOCATION_ID env var
CREATE TABLE locations (
    id TEXT PRIMARY KEY NOT NULL, -- e.g "reception"
    name TEXT NOT NULL            -- "Reception Desk Main Building"
);

-- playlist_items table
-- Each row is one item in a locations playlist
-- Items are odered by the `position` column, the lower number is shown first
--
-- All asse URLs point ot files on disk on this server
-- Store is URLs (not file paths) so fetch can be by HTTP
-- Files are store on disk, not as blobs in database
--
-- The `item_type` column determines which other columns are relevant:
--   url        → url, duration_secs
--   image      → url, duration_secs
--   video      → url
--   slideshow  → urls (JSON array), seconds_per_slide
--   pdf        → page_urls (JSON array), seconds_per_page
CREATE TABLE playlist_items (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    location_id       TEXT    NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    position          INTEGER NOT NULL,     -- display order, 0-indexed
    item_type         TEXT    NOT NULL,     -- "url" | "image" | "video" | "slideshow" | "pdf"

    -- Used by: url, image, video
    url               TEXT,

    --Use by: url, image (time per advance)
    duration_secs      INTEGER,

    -- Used by: slideshow, pdf (JSON array of urls)
    urls               TEXT,

    --Used by: slideshow
    seconds_per_slide  INTEGER,

    --used by: pdf
    seconds_per_page   INTEGER
);

-- Index so fetching a locations playlist is faster
CREATE INDEX idx_playlist_items_location ON playlist_items(location_id, position);