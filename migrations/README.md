# Database Migrations

This directory contains PostgreSQL migration scripts for the Tatteau database.

## How to Apply Migrations

### Using Railway CLI

```bash
# Connect to your Railway PostgreSQL database
railway connect postgres

# Then run the migration
\i migrations/fix_type_mismatches.sql
```

### Using psql directly

```bash
# Get your DATABASE_URL from Railway
export DATABASE_URL="postgresql://..."

# Apply the migration
psql $DATABASE_URL -f migrations/fix_type_mismatches.sql
```

### Using sqlx-cli

If you want to use sqlx migrations instead:

```bash
# Install sqlx-cli if needed
cargo install sqlx-cli --no-default-features --features postgres

# Create migration
sqlx migrate add fix_type_mismatches

# Copy the SQL from the .sql file to the generated migration file
# Then run migrations
sqlx migrate run --database-url $DATABASE_URL
```

## Available Migrations

### fix_type_mismatches.sql
Fixes type mismatches between PostgreSQL and Rust:
- Changes `openai_api_costs.avg_cost` and `total_cost` from REAL (FLOAT4) to DOUBLE PRECISION (FLOAT8)
- This ensures compatibility with Rust's f64 type

**Status**: Ready to apply to Railway database
