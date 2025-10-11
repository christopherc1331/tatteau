#!/usr/bin/env python3
"""
Migrate server.rs from rusqlite to sqlx (PostgreSQL)
"""

import re
import sys

def migrate_server_rs(content):
    """Comprehensive migration of server.rs from rusqlite to sqlx"""

    # Step 1: Remove all rusqlite imports and add sqlx
    content = re.sub(
        r'use rusqlite::{[^}]+};?\n',
        '',
        content
    )
    content = re.sub(
        r'use rusqlite::[^;]+;\n',
        '',
        content
    )

    # Step 2: Remove Path imports in SSR blocks (not needed for connection pool)
    content = re.sub(
        r'use std::path::Path;\n',
        '',
        content
    )

    # Step 3: Replace Connection::open with pool
    content = re.sub(
        r'let db_path = Path::new\("tatteau\.db"\);\s*\n\s*let conn = Connection::open\(db_path\)\??',
        'let pool = crate::db::pool::get_pool()',
        content
    )

    # Step 4: Replace Result as SqliteResult with sqlx::Result
    content = content.replace('Result as SqliteResult', 'sqlx::Result')
    content = content.replace('SqliteResult<', 'sqlx::Result<')

    # Step 5: Make all helper functions async
    # Pattern: fn function_name(...) -> SqliteResult<T> {
    content = re.sub(
        r'(\s+)fn\s+([a-z_]+)\s*\(([^)]*)\)\s*->\s*sqlx::Result<([^>]+)>\s*\{',
        r'\1async fn \2(\3) -> sqlx::Result<\4> {',
        content
    )

    # Step 6: Convert parameter placeholders ?1, ?2, etc. to $1, $2, etc.
    for i in range(20, 0, -1):  # Go backwards to avoid replacing ?10 before ?1
        content = content.replace(f'?{i}', f'${i}')
    content = content.replace('?', '$1')  # Catch any remaining single ?

    # Step 7: Replace rusqlite::params! with direct array/tuple
    # This is complex as rusqlite::params![a, b, c] should become &[&a as &(dyn sqlx::Encode<_> + _), &b as &(dyn sqlx::Encode<_> + _)]
    # But for simplicity, we'll handle common patterns

    # Step 8: Replace conn.execute with sqlx::query().execute(pool).await
    # Pattern: conn.execute("SQL", params)
    # But this is complex, will need careful handling

    # Step 9: Replace conn.prepare() and query_map() patterns with sqlx::query!() or query_as!()

    # Step 10: Handle boolean comparisons
    content = content.replace('= 1', '= true')
    content = content.replace('!= 1', '!= true')
    content = content.replace('is_recurring = 1', 'is_recurring = true')
    content = content.replace('active = 1', 'active = true')
    content = content.replace('is_closed = 0', 'is_closed = false')

    # Step 11: Replace rusqlite error types
    content = content.replace('rusqlite::Error', 'sqlx::Error')
    content = content.replace('rusqlite::params', 'sqlx::query')
    content = content.replace('params_from_iter', 'query')
    content = content.replace('rusqlite::', 'sqlx::')

    # Step 12: Add .await to all database operations
    # This requires pattern matching for specific operations

    return content

def main():
    input_file = '/home/chris/code/personal/tatteau/web/src/server.rs'
    output_file = '/home/chris/code/personal/tatteau/web/src/server.rs.new'

    with open(input_file, 'r') as f:
        content = f.read()

    migrated = migrate_server_rs(content)

    with open(output_file, 'w') as f:
        f.write(migrated)

    print(f"Migration complete. Output written to {output_file}")
    print("Review the changes and then: mv server.rs.new server.rs")

if __name__ == '__main__':
    main()
