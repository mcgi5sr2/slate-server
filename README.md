# Slate Show

A self-hosted digital signage system. Manage content and playlists from a central server — display them on any device with a browser.

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│                   Server (k3s VM)                │
│                                                  │
│  ┌──────────────┐     ┌────────────────────────┐ │
│  │  Axum API    │     │  SQLite (metadata)     │ │
│  │  /api/...    │────▶│  locations             │ │
│  │  /kiosk/...  │     │  playlist_items        │ │
│  │  /admin/...  │     └────────────────────────┘ │
│  │  /uploads/.. │     ┌────────────────────────┐ │
│  └──────────────┘     │  Disk (media files)    │ │
│                       │  data/uploads/         │ │
│                       └────────────────────────┘ │
└─────────────────────────────────────────────────┘
          │                        │
          │ HTTP poll (ETag)        │ HTTP (media files)
          ▼                        ▼
┌─────────────────┐      ┌─────────────────┐
│  Kiosk Browser  │      │  Admin Browser  │
│  /kiosk/{id}    │      │  /admin/        │
│  Chromium kiosk │      │  Any browser    │
└─────────────────┘      └─────────────────┘
```

- **Server** — Axum + SQLite. Stores metadata in SQLite, media files on disk. Serves the admin UI and kiosk display pages.
- **Kiosk** — Any browser in kiosk mode. Polls the server for playlist changes using ETags (304 Not Modified when unchanged). Streams media directly from the server.
- **Admin UI** — Browser-based. Manage locations, upload media, build playlists.

---

## Running Locally

### Prerequisites
- Rust (stable)
- SQLx CLI: `cargo install sqlx-cli --no-default-features --features sqlite`

### Setup

```bash
# Clone the repo
git clone <repo-url>
cd slate_show

# Create .env
cp .env.example .env   # or create manually — see Environment Variables below

# Run the server
cargo run -p slate-server
```

The server will:
- Create `data/db/slate.db` on first run
- Run database migrations automatically
- Serve on `http://localhost:3000`

### Access
| URL | Description |
|-----|-------------|
| `http://localhost:3000/admin/` | Admin UI |
| `http://localhost:3000/kiosk/{location_id}` | Kiosk display page |
| `http://localhost:3000/api/...` | REST API |

---

## Environment Variables

All variables are loaded from `.env` at the workspace root.

| Variable | Example | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:data/db/slate.db` | SQLite database path |
| `UPLOADS_DIR` | `./data/uploads` | Directory for uploaded media files |
| `STATIC_DIR` | `./server/static` | Directory for admin/kiosk static files |

---

## API Reference

### Locations

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/locations` | List all locations |
| `GET` | `/api/locations/{id}` | Get a single location |
| `POST` | `/api/locations` | Create a location |
| `DELETE` | `/api/locations/{id}` | Delete a location and its playlist |

**Create location body:**
```json
{ "id": "reception", "name": "Reception Desk" }
```

### Playlists

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/playlist/{location_id}` | Get playlist (supports ETag / 304) |
| `POST` | `/api/playlist/{location_id}` | Replace entire playlist |

**Set playlist body:**
```json
{
  "items": [
    { "type": "image", "url": "/uploads/reception/logo.jpg", "duration_secs": 10 },
    { "type": "video", "url": "/uploads/reception/promo.mp4" },
    { "type": "url",   "url": "http://grafana.internal/d/abc", "duration_secs": 30 }
  ]
}
```

### File Upload

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/upload` | Upload a media file |

**Multipart fields:** `location_id`, `name` (slug, no extension), `file`

Files are stored at `{UPLOADS_DIR}/{location_id}/{name}.{ext}` and served at `/uploads/{location_id}/{name}.{ext}`.

Uploading with the same `name` overwrites the existing file — playlist URLs remain stable.

---

## Kiosk Setup

On a kiosk machine with Chromium installed:

```bash
chromium --kiosk \
  --disk-cache-size=2147483648 \
  http://<server-ip>:3000/kiosk/reception
```

--disk-cache-size is optional depending on how large and how many files you want the kiosk to handle in local cache

The kiosk page:
- Polls `/api/playlist/reception` every 10 seconds
- Sends `If-None-Match` header — server replies `304 Not Modified` if unchanged
- Cycles through items fullscreen with crossfade transitions
- Continues showing the current playlist if the server is temporarily unreachable

---

## Playlist Item Types

| Type | Fields | Notes |
|------|--------|-------|
| `image` | `url`, `duration_secs` | JPEG, PNG, GIF supported |
| `video` | `url` | Advances when video ends. Muted for autoplay. |
| `url` | `url`, `duration_secs` | Fullscreen iframe. Site must allow embedding. |

---

## Roadmap

### Phase 2
- PDF upload — pre-render pages to images at upload time
- Slideshow type — multiple images, auto-advance per slide
- Native Rust kiosk client (`kiosk/` crate) with local file caching
- OAuth / auth token support for embedded dashboards (PowerBI, SSO)

### Future Considerations
- Chromium cache tuning (`--disk-cache-size`) for video-heavy deployments
- Explicit `Cache-Control` headers on uploaded media
- GIF single-loop detection (currently loops for full `duration_secs`)
- Kubernetes deployment manifests (PVC for uploads, ConfigMap for env)

---

## Project Structure

```
slate_show/
├── server/               # Axum API server
│   ├── src/
│   │   ├── main.rs       # Server entry point, router setup
│   │   ├── store.rs      # AppState (DB pool, config paths)
│   │   ├── models.rs     # Shared data types
│   │   ├── db/           # SQLx database queries
│   │   └── routes/       # Axum route handlers
│   └── static/           # Admin UI + kiosk display page
│       ├── index.html    # Location manager
│       ├── playlist.html # Playlist editor
│       ├── kiosk.html    # Kiosk display (served via /kiosk/{id})
│       ├── app.js        # Shared API client JS
│       ├── theme.js      # Theme switcher (light/dark/monokai)
│       └── style.css     # Shared stylesheet with CSS theme variables
├── kiosk/                # Phase 2: native Rust kiosk client (placeholder)
├── migrations/           # SQLx migrations
├── data/
│   ├── db/               # SQLite database
│   └── uploads/          # Uploaded media files ({location_id}/{name}.ext)
└── .sqlx/                # SQLx offline query cache (committed to git)
```
