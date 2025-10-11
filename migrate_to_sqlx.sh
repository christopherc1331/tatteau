#!/bin/bash

# Backup files
cp web/src/db/repository.rs web/src/db/repository.rs.backup
cp web/src/db/search_repository.rs web/src/db/search_repository.rs.backup
cp web/src/db/favorites_repository.rs web/src/db/favorites_repository.rs.backup

echo "✓ Backed up repository files"
echo "Starting migration from rusqlite to sqlx..."
echo ""
echo "FILES TO MIGRATE:"
echo "1. web/src/db/repository.rs (1826 lines)"
echo "2. web/src/db/search_repository.rs (366 lines)"
echo "3. web/src/db/favorites_repository.rs (200 lines)"
echo ""
echo "CONVERSION PATTERNS TO APPLY:"
echo "- Replace rusqlite imports with sqlx"
echo "- Replace SqliteResult<T> with DbResult<T> = Result<T, sqlx::Error>"
echo "- Replace Connection::open() with get_pool()"
echo "- Replace prepare/query_map with sqlx::query().fetch_all()"
echo "- Replace ?1, ?2, ?3 with $1, $2, $3"
echo "- Add async to all functions"
echo "- Add .await to all database calls"
echo ""
echo "Please run manual migration or use a Python/Rust script for full conversion."
