// app.js — all API calls for the Slate Show admin UI
// This file is shared between index.html and playlist.html

// Base URL for the API — empty string means same host/port as the page
const API = '';

// ─── Locations ───────────────────────────────────────────────────────────────

// Fetch all locations from the server and render them into the #locations div
async function loadLocations() {
    const res = await fetch(`${API}/api/locations`);
    const locations = await res.json();
    const container = document.getElementById('locations');

    if (locations.length === 0) {
        container.innerHTML = '<p>No locations yet. Add one below.</p>';
        return;
    }

    // Build a row for each location including:
    // - link to playlist editor
    // - the id slug in monospace
    // - a direct kiosk view link
    // - delete button
    container.innerHTML = locations.map(loc => `
        <div class="location">
            <a href="playlist.html?id=${loc.id}">${loc.name}</a>
            <span class="location-id">${loc.id}</span>
            <a class="kiosk-link" href="kiosk.html?id=${loc.id}" target="_blank">▶ Kiosk view</a>
            <button class="danger" onclick="deleteLocation('${loc.id}')">Delete</button>
        </div>
    `).join('');
}

// Create a new location by POSTing to the API
async function createLocation(id, name) {
    const res = await fetch(`${API}/api/locations`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id, name }),
    });
    return res;
}

// Delete a location by its id
async function deleteLocation(id) {
    if (!confirm(`Delete location "${id}"? This will also delete its playlist.`)) return;
    await fetch(`${API}/api/locations/${id}`, { method: 'DELETE' });
    loadLocations();
}

// ─── Playlists ────────────────────────────────────────────────────────────────

// Fetch the playlist for a location
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

// Upload a file with a given location and name slug.
// Shows a progress bar while uploading.
// Returns the public URL of the uploaded file, or null on failure.
async function uploadFile(locationId, name, file) {
    const form = new FormData();
    form.append('location_id', locationId);
    form.append('name', name);
    form.append('file', file);

    // Show the progress bar
    const wrap = document.getElementById('upload-progress-wrap');
    const fill = document.getElementById('upload-progress-fill');
    if (wrap) wrap.style.display = 'block';
    if (fill) fill.style.width = '0%';

    // We use XMLHttpRequest instead of fetch here because fetch doesn't
    // support upload progress events. XHR lets us track bytes sent.
    return new Promise((resolve) => {
        const xhr = new XMLHttpRequest();

        // Update the progress bar as bytes are sent
        xhr.upload.addEventListener('progress', (e) => {
            if (e.lengthComputable && fill) {
                const pct = Math.round((e.loaded / e.total) * 100);
                fill.style.width = pct + '%';
            }
        });

        xhr.addEventListener('load', () => {
            // Hide the progress bar when done
            if (wrap) wrap.style.display = 'none';

            if (xhr.status === 200) {
                const data = JSON.parse(xhr.responseText);
                resolve(data.url);
            } else {
                resolve(null);
            }
        });

        xhr.addEventListener('error', () => {
            if (wrap) wrap.style.display = 'none';
            resolve(null);
        });

        xhr.open('POST', `${API}/api/upload`);
        xhr.send(form);
    });
}

// ─── Thumbnail helper ─────────────────────────────────────────────────────────

// Return an HTML string for a thumbnail or placeholder based on item type and url
function itemThumb(item) {
    if (item.type === 'image' && item.url) {
        return `<img class="item-thumb" src="${item.url}" alt="" loading="lazy">`;
    }
    const label = item.type === 'video' ? '▶ video' : '🌐 url';
    return `<div class="item-thumb-placeholder">${label}</div>`;
}

// Return an HTML badge for an item type with syntax-highlight style colouring
function itemTypeBadge(type) {
    return `<span class="item-type item-type-${type}">${type}</span>`;
}
