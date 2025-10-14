-- Migration to fix PostgreSQL type mismatches with Rust expectations
-- Run this on your Railway PostgreSQL database

-- Fix float precision: Change REAL (FLOAT4) to DOUBLE PRECISION (FLOAT8)
-- for columns where Rust expects f64
ALTER TABLE openai_api_costs
  ALTER COLUMN avg_cost TYPE DOUBLE PRECISION,
  ALTER COLUMN total_cost TYPE DOUBLE PRECISION;

-- Note: For ID columns, PostgreSQL uses BIGSERIAL/BIGINT (INT8) which is correct
-- for auto-incrementing IDs. The Rust code should use i64 for these columns.
-- If you want to use i32 in Rust, you would need to change columns to INTEGER (INT4),
-- but this limits the maximum ID to ~2 billion, which may be reached for high-volume tables.
