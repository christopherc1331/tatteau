# Multi-stage build for Leptos web application
FROM rust:1.88-bookworm AS builder

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

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY shared-types ./shared-types
COPY web ./web
COPY data-ingestion ./data-ingestion

# Build the web application
WORKDIR /app/web
# Cache bust for wasm-bindgen 0.2.104 update - 2025-10-09
RUN cargo leptos build --release && \
    echo "=== Checking build output ===" && \
    ls -la /app/target/ && \
    echo "=== Checking for server dir ===" && \
    ls -la /app/target/server/ || echo "No server dir" && \
    echo "=== Checking for release dir ===" && \
    ls -la /app/target/release/ || echo "No release dir" && \
    echo "=== Finding all 'web' executables ===" && \
    find /app/target -name "web" -type f 2>/dev/null

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app tatteau

# Set working directory
WORKDIR /app

# Copy the server binary from workspace target directory
COPY --from=builder --chown=tatteau:tatteau /app/target/server/release/web /app/tatteau-web

# Copy the site files from workspace target directory
COPY --from=builder --chown=tatteau:tatteau /app/target/site /app/site

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