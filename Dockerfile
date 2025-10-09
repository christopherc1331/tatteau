# Multi-stage build for Leptos web application using cargo-chef pattern
# Stage 1: Planner - generates dependency recipe
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY shared-types ./shared-types
COPY web ./web
COPY data-ingestion ./data-ingestion
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - caches dependencies and builds app
FROM chef AS builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install wasm32 target for client-side compilation
RUN rustup target add wasm32-unknown-unknown

# Install cargo-leptos for building the web app
RUN cargo install cargo-leptos --locked

WORKDIR /app

# Copy recipe and build dependencies (cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code and build application
COPY Cargo.toml Cargo.lock ./
COPY shared-types ./shared-types
COPY web ./web
COPY data-ingestion ./data-ingestion

# Build the web application
WORKDIR /app/web
RUN cargo leptos build --release

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app tatteau

# Set working directory
WORKDIR /app

# Copy the server binary
COPY --from=builder --chown=tatteau:tatteau /app/web/target/server/release/web /app/tatteau-web

# Copy the site files
COPY --from=builder --chown=tatteau:tatteau /app/web/target/site /app/site

# Copy the database (if it exists) - for local development only
# In production, the database will be on the mounted volume
COPY --chown=tatteau:tatteau tatteau.db /app/tatteau.db 2>/dev/null || true

# Set environment variables
ENV LEPTOS_OUTPUT_NAME="web"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_RELOAD_PORT="8081"
ENV RUST_LOG="info"
ENV DATABASE_PATH="/app/data/tatteau.db"

# Create data directory for volume mount
RUN mkdir -p /app/data && chown tatteau:tatteau /app/data

# Switch to app user
USER tatteau

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/ || exit 1

# Run the server
CMD ["./tatteau-web"]