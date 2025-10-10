# Database Migration Plan: SQLite → PostgreSQL on Railway

## Executive Summary

This document outlines the plan to migrate the Tatteau application from SQLite to PostgreSQL hosted on Railway.

**Current State:**
- Database: SQLite (local file-based)
- Size: 25MB
- Tables: 30 tables
- Data: 19,118 artists, 72,008 images, 1 user
- Connection pattern: Synchronous (rusqlite)

**Target State:**
- Database: PostgreSQL (Railway-hosted)
- Connection pattern: Async (sqlx)
- Connection pooling: Enabled
- Environment-based configuration

**Estimated Effort:** 25-37 hours (3-5 days of focused work)

---

## Phase 1: Infrastructure Setup (3-4 hours)

### ✅ TODO: Setup Railway PostgreSQL Instance
- [ ] Create Railway account/project
- [ ] Provision PostgreSQL database
- [ ] Note down connection credentials
- [ ] Configure environment variables:
  ```
  DATABASE_URL=postgresql://user:password@host:port/database
  ```
- [ ] Test connection from local machine

### ✅ TODO: Update Dependencies in Cargo.toml
**Files to modify:**
- `web/Cargo.toml`
- `data-ingestion/Cargo.toml`

**Changes:**
```toml
# Remove or make optional
rusqlite = { version = "0.34.0", features = ["bundled"], optional = true }

# Add new dependencies
[dependencies]
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "macros"] }
```

### ✅ TODO: Export Current SQLite Schema
```bash
cd /home/chris/personal/code/tatteau
sqlite3 tatteau.db .schema > sqlite_schema_backup.sql
```

---

## Phase 2: Schema Migration (2-3 hours)

### ✅ TODO: Convert Schema to PostgreSQL
Create `postgresql_schema.sql` with conversions:

**Conversion Rules:**
```sql
-- SQLite → PostgreSQL

-- Primary Keys
INTEGER PRIMARY KEY AUTOINCREMENT → SERIAL PRIMARY KEY

-- Data Types
TEXT → VARCHAR or TEXT (both work)
INTEGER → INTEGER or BIGINT
REAL → NUMERIC or DOUBLE PRECISION
BOOLEAN → BOOLEAN (native type)

-- Timestamps
TIMESTAMP DEFAULT CURRENT_TIMESTAMP → TIMESTAMP DEFAULT NOW()

-- JSON fields (recommended optimization)
TEXT (storing JSON) → JSONB
```

**Key Tables to Convert:**
1. artists
2. artists_images
3. artists_images_styles
4. locations
5. users
6. user_favorites
7. booking_requests
8. styles
9. (+ 22 more tables)

### ✅ TODO: Create PostgreSQL Schema
```bash
# Connect to Railway PostgreSQL
psql $DATABASE_URL

# Run schema creation
\i postgresql_schema.sql

# Verify tables
\dt
```

---

## Phase 3: Data Migration (2-3 hours)

### ✅ TODO: Option A - Use pgloader (Recommended)
```bash
# Install pgloader
sudo apt-get install pgloader  # or brew install pgloader

# Run migration
pgloader tatteau.db postgresql://user:password@host:port/database

# Verify row counts
psql $DATABASE_URL -c "SELECT COUNT(*) FROM artists;"
psql $DATABASE_URL -c "SELECT COUNT(*) FROM artists_images;"
```

### ✅ TODO: Option B - Manual CSV Export/Import
```bash
# Export from SQLite
sqlite3 tatteau.db <<EOF
.headers on
.mode csv
.output artists.csv
SELECT * FROM artists;
.output artists_images.csv
SELECT * FROM artists_images;
-- Repeat for all tables
EOF

# Import to PostgreSQL
psql $DATABASE_URL -c "\COPY artists FROM 'artists.csv' CSV HEADER"
# Repeat for all tables
```

### ✅ TODO: Verify Data Integrity
```sql
-- Check row counts match
SELECT 'artists', COUNT(*) FROM artists
UNION ALL
SELECT 'artists_images', COUNT(*) FROM artists_images
UNION ALL
SELECT 'users', COUNT(*) FROM users;

-- Verify foreign key constraints
SELECT conname, conrelid::regclass, confrelid::regclass
FROM pg_constraint
WHERE contype = 'f';
```

---

## Phase 4: Connection Pool Infrastructure (2-3 hours)

### ✅ TODO: Create Database Connection Module
**File:** `web/src/db/pool.rs`

```rust
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::sync::OnceLock;

static DB_POOL: OnceLock<Pool<Postgres>> = OnceLock::new();

pub async fn init_pool() -> Result<(), sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    DB_POOL.set(pool).ok();
    Ok(())
}

pub fn get_pool() -> &'static Pool<Postgres> {
    DB_POOL.get().expect("Database pool not initialized")
}
```

### ✅ TODO: Update web/src/db/mod.rs
```rust
pub mod entities;
pub mod favorites_repository;
pub mod repository;
pub mod search_repository;
pub mod pool;  // Add this

pub use pool::{init_pool, get_pool};
```

### ✅ TODO: Initialize Pool at Application Startup
**File:** `web/src/main.rs` (or server setup)

```rust
#[tokio::main]
async fn main() {
    // Initialize database pool
    db::init_pool().await.expect("Failed to initialize database pool");

    // ... rest of server setup
}
```

---

## Phase 5: Repository Migration (16-20 hours)

This is the largest effort. Migrate repositories one at a time.

### ✅ TODO: Migrate web/src/db/favorites_repository.rs (2-3 hours)
**Scope:** 6 connection points

**Pattern to follow:**
```rust
// OLD - rusqlite (sync)
pub fn add_favorite(user_id: i32, artists_images_id: i32) -> SqliteResult<i64> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT OR IGNORE INTO user_favorites (user_id, artists_images_id)
         VALUES (?1, ?2)",
        params![user_id, artists_images_id],
    )?;

    Ok(conn.last_insert_rowid())
}

// NEW - sqlx (async)
pub async fn add_favorite(user_id: i32, artists_images_id: i32) -> Result<i64, sqlx::Error> {
    let pool = get_pool();

    let result = sqlx::query!(
        "INSERT INTO user_favorites (user_id, artists_images_id)
         VALUES ($1, $2)
         ON CONFLICT DO NOTHING
         RETURNING id",
        user_id,
        artists_images_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.id as i64).unwrap_or(0))
}
```

**Functions to migrate:**
- [ ] `add_favorite()`
- [ ] `remove_favorite()`
- [ ] `is_favorited()`
- [ ] `get_user_favorites()`
- [ ] `get_user_favorite_count()`
- [ ] `toggle_favorite()`
- [ ] `get_user_favorites_with_details()`

### ✅ TODO: Migrate web/src/db/search_repository.rs (2-3 hours)
**Scope:** 3 connection points

**Functions to migrate:**
- [ ] `search_artists_by_criteria()`
- [ ] `get_artist_with_portfolio()`

### ✅ TODO: Migrate web/src/db/repository.rs (8-10 hours)
**Scope:** 38 connection points - largest file (1847 lines)

**Strategy:** Break into logical groups:

**Artist Operations:**
- [ ] `insert_artist()`
- [ ] `get_artist_by_id()`
- [ ] `update_artist()`
- [ ] `delete_artist()`
- [ ] `get_all_artists()`

**Location Operations:**
- [ ] `insert_location()`
- [ ] `get_location_by_id()`

**Image Operations:**
- [ ] `insert_artist_image()`
- [ ] `get_images_by_artist()`

**Style Operations:**
- [ ] `insert_style()`
- [ ] `get_styles()`
- [ ] `link_artist_style()`
- [ ] `link_image_style()`

**Booking Operations:**
- [ ] `create_booking_request()`
- [ ] `get_booking_requests()`
- [ ] `update_booking_status()`

**User Operations:**
- [ ] `create_user()`
- [ ] `get_user_by_email()`
- [ ] `create_artist_user()`
- [ ] `verify_artist_credentials()`

**Business Hours/Availability:**
- [ ] `get_business_hours()`
- [ ] `update_business_hours()`
- [ ] `create_recurring_rule()`

**Subscription/Questionnaire:**
- [ ] `get_subscription_tiers()`
- [ ] `create_artist_subscription()`
- [ ] `get_questionnaire_questions()`

### ✅ TODO: Migrate web/src/server.rs (6-8 hours)
**Scope:** 34 connection points

**Current pattern:**
```rust
let db_path = Path::new("tatteau.db");
let conn = Connection::open(db_path)?;
```

**Replace with:**
```rust
let pool = get_pool();
```

**Server functions to update:**
- [ ] `get_artist_by_id_server()`
- [ ] `get_artists_by_style()`
- [ ] `get_tattoo_posts_by_style()`
- [ ] `register_user()`
- [ ] `login_user()`
- [ ] `register_artist()`
- [ ] `login_artist()`
- [ ] `create_booking()`
- [ ] `get_artist_bookings()`
- [ ] And ~25 more server functions...

**Important:** All `#[server]` functions are already async, just need to add `.await` to queries.

---

## Phase 6: Data Ingestion Crate Migration (3-4 hours)

### ✅ TODO: Migrate data-ingestion/src/repository.rs
**Scope:** Similar pattern to web repositories

**Functions to migrate:**
- [ ] Location insertion
- [ ] Artist insertion
- [ ] Image insertion
- [ ] Style extraction storage

### ✅ TODO: Update Scraper Actions
**Files:**
- [ ] `src/actions/scraper.rs`
- [ ] `src/actions/style_extraction.rs`
- [ ] `src/actions/google_api_ingestion/driver.rs`

---

## Phase 7: Query Syntax Updates (3-4 hours)

### ✅ TODO: Update Parameter Placeholders

**Find and replace pattern:**
```bash
# Search for rusqlite query patterns
rg "execute\(" --type rust
rg "query_row\(" --type rust
rg "query_map\(" --type rust
```

**Common conversions:**

| SQLite (rusqlite) | PostgreSQL (sqlx) |
|-------------------|-------------------|
| `?1, ?2, ?3` | `$1, $2, $3` |
| `params![a, b]` | Direct binding: `a, b` |
| `.execute(...)? ` | `.execute(...).await?` |
| `.query_row(...)? ` | `.fetch_one(...).await?` |
| `.query_map(...)?` | `.fetch_all(...).await?` |
| `last_insert_rowid()` | `RETURNING id` clause |

### ✅ TODO: Handle INSERT OR IGNORE
PostgreSQL uses `ON CONFLICT`:
```sql
-- SQLite
INSERT OR IGNORE INTO table VALUES (...)

-- PostgreSQL
INSERT INTO table VALUES (...)
ON CONFLICT DO NOTHING
```

### ✅ TODO: Update Date/Time Functions
Most are compatible, but verify:
- `CURRENT_TIMESTAMP` → `NOW()` (both work)
- `datetime('now')` → `NOW()`

---

## Phase 8: Testing & Validation (4-8 hours)

### ✅ TODO: Unit Testing
- [ ] Test each repository function individually
- [ ] Verify data types match expectations
- [ ] Test edge cases (NULL values, empty results)

### ✅ TODO: Integration Testing
**Test each major flow:**
- [ ] User registration and login
- [ ] Artist registration and login
- [ ] Browsing tattoo posts
- [ ] Searching artists by style/location
- [ ] Favoriting/unfavoriting images
- [ ] Creating booking requests
- [ ] Artist availability management
- [ ] Business hours configuration

### ✅ TODO: Performance Testing
- [ ] Test connection pool under load
- [ ] Verify query performance
- [ ] Check for N+1 query problems
- [ ] Monitor connection pool usage

### ✅ TODO: Data Integrity Checks
```sql
-- Verify foreign key relationships
SELECT COUNT(*) FROM user_favorites uf
LEFT JOIN artists_images ai ON uf.artists_images_id = ai.id
WHERE ai.id IS NULL;
-- Should return 0

-- Verify all users exist
SELECT COUNT(*) FROM user_favorites uf
LEFT JOIN users u ON uf.user_id = u.id
WHERE u.id IS NULL;
-- Should return 0
```

---

## Phase 9: Deployment (2-3 hours)

### ✅ TODO: Environment Configuration
**Railway Environment Variables:**
```
DATABASE_URL=postgresql://...
JWT_SECRET=... (change from hardcoded value)
OPENAI_API_KEY=...
```

### ✅ TODO: Update .env Files
```bash
# .env.development
DATABASE_URL=postgresql://localhost:5432/tatteau_dev

# .env.production (Railway)
DATABASE_URL=<railway-provided-url>
```

### ✅ TODO: Update Deployment Scripts
- [ ] Ensure `cargo build --release` includes PostgreSQL features
- [ ] Update Docker files if applicable
- [ ] Configure Railway build settings

### ✅ TODO: Database Backup Strategy
```bash
# Automated backups on Railway
# Manual backup command:
pg_dump $DATABASE_URL > backup_$(date +%Y%m%d).sql
```

---

## Phase 10: Rollback Plan (Just in Case)

### ✅ TODO: Keep SQLite as Fallback Option
**Option 1:** Feature flags
```toml
[features]
default = ["postgres"]
postgres = ["dep:sqlx"]
sqlite = ["dep:rusqlite"]
```

**Option 2:** Keep old code in separate branch
```bash
git checkout -b pre-postgres-migration
git commit -am "Checkpoint before PostgreSQL migration"
git push origin pre-postgres-migration
```

### ✅ TODO: Document Rollback Procedure
1. Revert to `pre-postgres-migration` branch
2. Restore SQLite database from backup
3. Redeploy with SQLite configuration

---

## Migration Checklist Summary

### Pre-Migration
- [ ] Create Railway PostgreSQL instance
- [ ] Export SQLite schema and data backups
- [ ] Create git checkpoint branch
- [ ] Update dependencies in Cargo.toml

### Schema & Data
- [ ] Convert schema to PostgreSQL format
- [ ] Create tables in PostgreSQL
- [ ] Migrate data using pgloader
- [ ] Verify data integrity

### Code Migration
- [ ] Create connection pool infrastructure
- [ ] Migrate `favorites_repository.rs`
- [ ] Migrate `search_repository.rs`
- [ ] Migrate `repository.rs`
- [ ] Migrate `server.rs`
- [ ] Migrate `data-ingestion` crate
- [ ] Update all query syntax

### Testing
- [ ] Unit test all repository functions
- [ ] Integration test all user flows
- [ ] Performance test under load
- [ ] Verify data integrity

### Deployment
- [ ] Configure environment variables
- [ ] Deploy to Railway
- [ ] Run smoke tests in production
- [ ] Monitor for errors

---

## Key Code Patterns Reference

### Connection Pattern
```rust
// OLD
use rusqlite::{Connection, params};
let conn = Connection::open("tatteau.db")?;

// NEW
use sqlx::PgPool;
let pool = get_pool();
```

### Query Pattern
```rust
// OLD - Insert
conn.execute(
    "INSERT INTO users (name, email) VALUES (?1, ?2)",
    params![name, email]
)?;

// NEW - Insert
sqlx::query!(
    "INSERT INTO users (name, email) VALUES ($1, $2)",
    name,
    email
)
.execute(pool)
.await?;

// OLD - Select One
let user: User = conn.query_row(
    "SELECT id, name, email FROM users WHERE id = ?1",
    params![id],
    |row| Ok(User {
        id: row.get(0)?,
        name: row.get(1)?,
        email: row.get(2)?,
    })
)?;

// NEW - Select One (with macro)
let user = sqlx::query_as!(
    User,
    "SELECT id, name, email FROM users WHERE id = $1",
    id
)
.fetch_one(pool)
.await?;

// OLD - Select Many
let mut stmt = conn.prepare("SELECT id, name FROM users")?;
let users = stmt.query_map([], |row| {
    Ok(User {
        id: row.get(0)?,
        name: row.get(1)?,
    })
})?
.collect::<Result<Vec<_>, _>>()?;

// NEW - Select Many
let users = sqlx::query_as!(
    User,
    "SELECT id, name FROM users"
)
.fetch_all(pool)
.await?;
```

---

## Resources

- [sqlx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [PostgreSQL to Railway Guide](https://docs.railway.app/databases/postgresql)
- [pgloader Documentation](https://pgloader.readthedocs.io/)
- [SQLite to PostgreSQL Migration Guide](https://www.postgresql.org/docs/current/migration.html)

---

## Notes for Future Agents

- This migration is estimated at 25-37 hours total
- Work incrementally: migrate one repository at a time
- Keep comprehensive tests to catch regressions
- The connection pool pattern is critical for performance
- All `#[server]` functions are already async, simplifying migration
- Parameter placeholder changes (`?` → `$1`) are the most tedious part
- Consider using search/replace with regex for common patterns

---

**Document Version:** 1.0
**Created:** 2025-10-10
**Last Updated:** 2025-10-10
**Status:** Ready for implementation
