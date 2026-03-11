// Hashmap for holding our data sets in memory as objects
use std::collections::HashMap;
// RwLock to allow multiple references to the same object
// Arc for shared ownership, dropped with last reference
use std::sync::{Arc, RwLock};

// use our data model
use crate::models::{Location, Playlist};

// App state is our memory store for all server data
// Clone is derived as boilerplate, so we can clone the state for the request handler
// Arc makes this cheaper as it increments a counter, not copy the data
#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<RwLock<StoreInner>>,
}

//StoreInner is the data wrapped in Arc
pub struct StoreInner {
    pub locations: HashMap<String, Location>,
    pub playlists: HashMap<String, Playlist>,
}

impl AppState {
    // create new() fresh store
    //Called once at startupt
    pub fn new() -> Self {
        AppState {
            inner: Arc::new(RwLock::new(StoreInner {
                locations: HashMap::new(),
                playlists: HashMap::new(),
            })),
        }
    }
}
