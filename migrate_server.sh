#!/bin/bash

# Backup the original file
cp web/src/server.rs web/src/server.rs.backup

# File to modify
FILE="web/src/server.rs"

# 1. Replace rusqlite imports with sqlx
sed -i 's/use rusqlite::{Connection, Result as SqliteResult};/use sqlx::Row;/g' "$FILE"
sed -i 's/use rusqlite::{params, Connection};/use sqlx::Row;/g' "$FILE"
sed -i 's/use rusqlite::{params_from_iter, Connection};/use sqlx::Row;/g' "$FILE"
sed -i 's/use rusqlite::Connection;/use sqlx::Row;/g' "$FILE"
sed -i 's/use rusqlite::Result as SqliteResult;/use sqlx::Row;/g' "$FILE"

# 2. Replace Connection::open with pool
sed -i 's/let db_path = Path::new("tatteau\.db");//g' "$FILE"
sed -i 's/let conn = Connection::open(db_path)?;/let pool = crate::db::pool::get_pool();/g' "$FILE"

# 3. Replace SqliteResult with sqlx::Result
sed -i 's/SqliteResult</Result<, sqlx::Error>/g' "$FILE"

# 4. Change sync functions to async
sed -i 's/fn query_\([a-z_]*\)(/async fn query_\1(/g' "$FILE"

# 5. Replace conn. with sqlx::query and add await
# This is complex and needs manual review

echo "Initial automated replacements complete. Manual review required for:"
echo "- conn.prepare() -> sqlx::query()"
echo "- .query_row() -> .fetch_one(pool).await"
echo "- .query_map() -> .fetch_all(pool).await"
echo "- .execute() -> .execute(pool).await"
echo "- Parameter placeholders ?1, ?2 -> $1, $2"
echo "- row.get(0) -> row.get(\"column_name\")"
