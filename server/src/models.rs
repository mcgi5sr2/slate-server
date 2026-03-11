// Use Serialize and Deserialze from serde
// This converst our rust struct/s to JSON, and vice versa
use serde::{Deserialize, Serialize};

// Kiosks identify by str named location as id
// but also str named location `name` to make it clearer to users which location
pub struct Location {
    pub id: String, // for the machine god
    pub name: String, // for humies
}

// Ordered list for play order
// Kiosk will fetch struct to cycle through
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub location_id: String, // whose playlist is this by id
    pub items: Vec<PlaylistItem>, // ordered list
}

// PlaylistItem is an enum which we tag with the type field ie gazzghull "orc"
// when serialised the 'type' field will tell the kiosk how to handle it
// Serialize turns Rust variant names lowercase, so we snake case them.

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlaylistItem {
    // for web pages, to display in webview
    Url {
        url: String,
        duration_secs: u32
    },

    // Image file hosted on server
    Image {
        url: String,
        duraction_secs: u32,
    },

    // video file, plays full duration then advances
    Video {
        url: String,
    },

    // Set of images, cycled at fixed rate
    Slideshow{
        urls:Vec<String>,
        seconds_per_slide: u32,
    },

    // PDF, server will pre render to images list
    // one URL per page
    Pdf {
        page_urls: Vec<String>,
        seconds_per_page: u32,
    },
}