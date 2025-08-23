# Tatteau

A workspace containing two crates:
- `data-ingestion`: Data ingestion and processing
- `web`: Web application built with Leptos

## Prerequisites

- Rust nightly toolchain (1.88.0 or later)
  ```bash
  rustup default nightly
  ```
  Note: The project requires nightly Rust 1.88.0+ due to dependency requirements
- cargo-leptos (for web development)
  ```bash
  cargo install cargo-leptos --locked
  ```
- Docker and Docker Compose (for deployment)
  ```bash
  # Install Docker: https://docs.docker.com/get-docker/
  # Install Docker Compose: https://docs.docker.com/compose/install/
  ```

## Building and Testing

### Building All Crates

To build all crates in the workspace:
```bash
cargo build
```

### Building Individual Crates

#### Data Ingestion Crate
```bash
cargo build -p data-ingestion
```

#### Web Crate
The web crate requires both SSR and hydration features:

```bash
cargo build -p web --features "ssr hydrate"
```

### Running Tests

To run tests for all crates:
```bash
cargo test
```

To run tests for a specific crate:

#### Data Ingestion Crate
```bash
cargo test -p data-ingestion
```

#### Web Crate
```bash
cargo test -p web --features "ssr hydrate"
```

### Development

For development, you can use `cargo check` instead of `cargo build` for faster compilation:

```bash
# Check all crates
cargo check

# Check data-ingestion crate
cargo check -p data-ingestion

# Check web crate
cargo check -p web --features "ssr hydrate"
```

## Development Server

To run the web application in development mode:

```bash
cd web
cargo leptos watch
```

This will start the development server with hot reloading at `http://localhost:3000`.

## Docker Deployment

### Quick Start

Deploy the application using the provided script:

```bash
# Development deployment
./deploy.sh

# Production deployment (with nginx reverse proxy)
./deploy.sh --env production

# Fresh build (no cache)
./deploy.sh --fresh-build
```

### Manual Docker Commands

```bash
# Build the Docker image
docker build -t tatteau-web .

# Run with Docker Compose
docker-compose up -d

# For production with nginx
docker-compose --profile production up -d
```

### Environment Variables

The following environment variables can be configured:

- `LEPTOS_SITE_ADDR`: Server address (default: `0.0.0.0:3000`)
- `RUST_LOG`: Log level (default: `info`)
- `LEPTOS_OUTPUT_NAME`: Build output name (default: `web`)
- `LEPTOS_SITE_ROOT`: Site root directory (default: `site`)

### Database

The application uses SQLite with the database file `tatteau.db`. In Docker deployment, the database is mounted as a volume to persist data between container restarts.

### Production Considerations

- Use the nginx reverse proxy for production (`docker-compose --profile production up -d`)
- Configure SSL certificates in the nginx configuration
- Set up proper backup strategies for the SQLite database
- Monitor application logs with `docker-compose logs -f`

## Application Features

The Tatteau platform includes:

- **Homepage**: Landing page with navigation to key features
- **Artist Discovery**: Map-based exploration of tattoo artists
- **Get Matched**: Quiz-based artist recommendation system
- **Artist Profiles**: Detailed artist information and portfolios
- **Booking System**: Appointment scheduling with artists
- **Style Gallery**: Browse tattoo styles and artwork 