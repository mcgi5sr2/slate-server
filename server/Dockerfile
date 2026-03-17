# ── Stage 1: Build ────────────────────────────────────────────────────────────
# Use the official Rust image to compile the server binary.
# We build from the workspace root so Cargo can see all crates.
FROM rust:latest AS builder

WORKDIR /build

# Copy the entire workspace
COPY . .

# Build only the server crate in release mode
# SQLX_OFFLINE=true uses the committed .sqlx/ cache — no live DB needed at build time
ENV SQLX_OFFLINE=true
RUN cargo build --release -p slate-server

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
# Use a minimal Debian image — no Rust toolchain needed at runtime
FROM debian:trixie-slim

WORKDIR /app

# Install CA certificates for HTTPS requests and SQLite runtime deps
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /build/target/release/slate-server /app/slate-server

# Copy static files for the admin UI and kiosk page
# Note: migrations are baked into the binary at compile time via sqlx::migrate!
COPY --from=builder /build/server/static /app/server/static

# Create data directories — these will be mounted as volumes in production
RUN mkdir -p /app/data/db /app/data/uploads

# Expose the server port
EXPOSE 3000

CMD ["/app/slate-server"]
