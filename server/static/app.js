// app.js — all API calls for the Slate Show admin UI
// This file is shared between index.html and playlist.html

// Base URL for the API — empty string means same host/port as the page
const API = '';

// ─── Locations ───────────────────────────────────────────────────────────────

// Fetch all locations from the server and render them into the #locations div
async function loadLocations() {
    // fetch() makes an HTTP request and returns a Promise
    // await pauses until the response arrives
    const res = await fetch(`${API}/api/locations`);

    // Parse the JSON body into a JS array
    const locations = await res.json();

    // Get the container div from the HTML
    const container = document.getElementById('locations');

    // If there are no locations, show a message
    if (locations.length === 0) {
        container.innerHTML = '<p>No locations yet. Add one below.</p>';
        return;
    }

    // Build the HTML for each location and join into one string
    // map() transforms each item in the array into an HTML string
    container.innerHTML = locations.map(loc => `
        <div class="location">
            <!-- Clicking the name goes to the playlist editor for this location -->
            <a href="playlist.html?id=${loc.id}">${loc.name}</a>
            <span>${loc.id}</span>
            <!-- onclick calls deleteLocation() with this location's id -->
            <button class="danger" onclick="deleteLocation('${loc.id}')">Delete</button>
        </div>
    `).join('');
}

// Create a new location by POSTing to the API
// Returns the fetch response so the caller can check .ok
async function createLocation(id, name) {
    const res = await fetch(`${API}/api/locations`, {
        method: 'POST',
        // Tell the server we're sending JSON
        headers: { 'Content-Type': 'application/json' },
        // Convert the JS object to a JSON string for the request body
        body: JSON.stringify({ id, name }),
    });
    return res;
}

// Delete a location by its id
async function deleteLocation(id) {
    // Ask the user to confirm before deleting
    if (!confirm(`Delete location "${id}"? This will also delete its playlist.`)) return;

    await fetch(`${API}/api/locations/${id}`, { method: 'DELETE' });

    // Reload the list after deletion
    loadLocations();
}

// ─── Playlists ────────────────────────────────────────────────────────────────

// Fetch the playlist for a location and render it
async function loadPlaylist(locationId) {
    const res = await fetch(`${API}/api/playlist/${locationId}`);
    const playlist = await res.json();
    return playlist;
}

// Save a playlist for a location — sends the full items array
async function savePlaylist(locationId, items) {
    const res = await fetch(`${API}/api/playlist/${locationId}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ items }),
    });
    return res;
}

// ─── File Upload ──────────────────────────────────────────────────────────────

// Upload a file with a given name slug
// Returns the URL of the uploaded file, or null on failure
async function uploadFile(name, file) {
    // FormData is the browser's way of sending multipart form data
    const form = new FormData();
    form.append('name', name);
    form.append('file', file);

    const res = await fetch(`${API}/api/upload`, {
        method: 'POST',
        // Note: do NOT set Content-Type header here — the browser sets it
        // automatically with the correct multipart boundary
        body: form,
    });

    if (!res.ok) return null;

    const data = await res.json();
    // data.url is the path like "/uploads/my-image.jpg"
    return data.url;
}
