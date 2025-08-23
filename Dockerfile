# Multi-stage build for Leptos web application
FROM rust:1.88-bookworm as builder

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

# Set working directory
WORKDIR /app

# Copy workspace files first for better caching
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

# Copy the database (if it exists)
COPY --chown=tatteau:tatteau tatteau.db /app/tatteau.db

# Set environment variables
ENV LEPTOS_OUTPUT_NAME="web"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_RELOAD_PORT="3001"
ENV RUST_LOG="info"

# Switch to app user
USER tatteau

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

# Run the server
CMD ["./tatteau-web"]