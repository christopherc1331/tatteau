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

## Deployment

### Fly.io Deployment (Recommended for Production)

Tatteau is configured for deployment on Fly.io with SQLite database persistence.

#### Prerequisites
1. [Fly.io account](https://fly.io/app/sign-up) (free tier available)
2. Fly.io CLI installed:
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

#### Option 1: Deploy from GitHub (Easiest)

1. Go to [Fly.io Launch](https://fly.io/launch)
2. Click "Launch from GitHub"
3. Sign in with GitHub and select the `tatteau` repository
4. Fly.io will auto-detect the Dockerfile and deploy
5. **Important:** Create the database volume when prompted:
   ```bash
   flyctl volumes create tatteau_data --size 1
   ```

#### Option 2: Deploy from Command Line

```bash
# Login to Fly.io
flyctl auth login

# Initialize and deploy
./deploy.sh init     # Creates the app
flyctl volumes create tatteau_data --size 1  # Creates database volume
./deploy.sh deploy   # Deploys the app

# (Optional) Restore local database to production
./deploy.sh db-restore tatteau.db
```

#### Database Management

**Backup production database:**
```bash
./deploy.sh db-backup  # Downloads to tatteau-backup-YYYYMMDD-HHMMSS.db
```

**Sync production DB for local querying:**
```bash
./sync-db.sh  # Creates tatteau-live.db for DataGrip/DB Browser
```

**Restore database to production:**
```bash
./deploy.sh db-restore <backup-file.db>
```

**Query production database remotely:**
```bash
# SSH into container with SQLite
flyctl ssh console -C "sqlite3 /app/data/tatteau.db"

# Run specific query
flyctl ssh console -C "sqlite3 /app/data/tatteau.db 'SELECT COUNT(*) FROM artists;'"
```

#### Continuous Deployment

The repository includes GitHub Actions for automatic deployment:
- Pushes to `main` branch automatically deploy to Fly.io
- Add `[skip ci]` to commit message to skip deployment

**Setup CI/CD:**
1. Get your Fly.io API token:
   ```bash
   flyctl auth token
   ```
2. Add to GitHub repository secrets as `FLY_API_TOKEN`:
   - Go to Settings → Secrets and variables → Actions
   - New repository secret: `FLY_API_TOKEN`

#### Monitoring

```bash
# View logs
./deploy.sh logs

# Check app status
./deploy.sh status

# SSH into container
./deploy.sh ssh

# Open app in browser
flyctl open
```

See [DEPLOYMENT.md](./DEPLOYMENT.md) for detailed deployment documentation.

### Docker Deployment (Local/Self-Hosted)

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

#### Database Schema

The application includes several key tables:

**Core Tables:**
- `artists` - Artist profiles and information
- `locations` - Geographic locations for artists
- `styles` - Tattoo style categories
- `bookings` - Appointment bookings between clients and artists
- `artist_images` - Artist portfolio images and Instagram posts

**Error Logging Table:**
- `error_logs` - Comprehensive error tracking and monitoring

##### Error Logs Table Structure

```sql
CREATE TABLE error_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    error_type TEXT NOT NULL,           -- 'client', 'server', 'database'
    level TEXT NOT NULL,                -- 'error', 'warning', 'info'
    message TEXT NOT NULL,              -- Error message
    stack_trace TEXT,                   -- Stack trace if available
    user_agent TEXT,                    -- Client user agent
    url TEXT,                          -- URL where error occurred
    user_id INTEGER,                   -- Associated user ID (optional)
    session_id TEXT,                   -- Session identifier
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    additional_context TEXT            -- JSON string with extra context
);
```

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

## Error Handling & Monitoring

The Tatteau platform includes a comprehensive Universal Error Logging system for production-level error monitoring, debugging, and reliability.

### Error Handling Features

#### Automatic Error Boundaries
The application includes React-style error boundaries that automatically catch and handle component-level errors:

```rust
use crate::components::error_boundary::ErrorBoundary;

// Wrap components in error boundaries
view! {
    <ErrorBoundary>
        <YourComponent />
    </ErrorBoundary>
}
```

#### Client-Side Error Logging
Errors occurring in the browser are automatically captured and sent to the server:

- **Component crashes**: Caught by error boundaries
- **JavaScript errors**: Captured with full context
- **Network failures**: Logged with request details
- **User interactions**: Context preserved for debugging

#### Server-Side Error Logging
Server errors are logged with comprehensive context:

- **Database errors**: Connection and query failures
- **API errors**: Request processing failures  
- **Authentication errors**: Login and session issues
- **File system errors**: Asset loading problems

### Error Log Data Structure

Each error log contains:

- **Error Classification**: client, server, or database
- **Severity Level**: error, warning, or info
- **Context Information**: URL, user agent, session data
- **Stack Traces**: Full error stack when available
- **User Context**: Associated user ID and session
- **Timestamps**: Precise error occurrence time
- **Additional Data**: Custom context as JSON

### Error Monitoring API

#### Logging Errors Programmatically

```rust
use crate::components::error_boundary::log_component_error;
use crate::server::{log_client_error, log_server_error};

// Client-side error logging
log_component_error(
    "Component crashed during render".to_string(),
    Some("TypeError: Cannot read property...".to_string()),
    Some("/artist/dashboard".to_string())
).await;

// Server-side error logging  
log_server_error(
    "Database connection failed".to_string(),
    Some("Connection timeout after 30s".to_string()),
    Some(user_id)
).await;
```

#### Retrieving Error Logs

```rust
use crate::server::get_error_logs;

// Get recent errors with filtering
let recent_errors = get_error_logs(
    Some("client".to_string()),  // Error type filter
    Some("error".to_string()),   // Level filter  
    Some(100)                    // Limit results
).await?;
```

### Production Error Monitoring

#### Error Log Analysis
- **Database queries**: Filter errors by type, level, timeframe
- **Trend analysis**: Monitor error frequency and patterns
- **User impact**: Track errors by user sessions
- **Performance**: Identify performance-related errors

#### Error Response Strategy
1. **Automatic logging**: All errors logged to database
2. **User-friendly display**: Professional error messages shown to users
3. **Graceful degradation**: Application continues functioning
4. **Development feedback**: Detailed error info in development mode

#### Monitoring Best Practices
- **Regular review**: Check error logs for recurring issues
- **Performance correlation**: Link errors to performance metrics
- **User experience**: Monitor errors that affect user workflows
- **Database maintenance**: Clean up old error logs periodically

### Development Debugging

#### Error Boundary Testing
```bash
# Enable detailed error logging in development
RUST_LOG=debug cargo leptos watch
```

#### Database Error Inspection
```bash
# View recent errors directly in SQLite
sqlite3 tatteau.db "SELECT * FROM error_logs ORDER BY timestamp DESC LIMIT 10;"

# Filter by error type
sqlite3 tatteau.db "SELECT * FROM error_logs WHERE error_type='client' AND level='error';"
```

This error handling system provides production-ready monitoring capabilities, ensuring issues are caught, logged, and can be resolved quickly while maintaining a smooth user experience. 