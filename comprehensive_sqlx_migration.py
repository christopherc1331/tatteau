#!/usr/bin/env python3
"""
Comprehensive migration of server.rs from rusqlite to sqlx (PostgreSQL)

This script handles the following conversions:
1. Remove all rusqlite imports
2. Remove Path imports
3. Replace Connection::open with pool
4. Convert all sync functions to async
5. Replace parameter placeholders (?1 -> $1)
6. Replace query_row with sqlx equivalents
7. Replace query_map with fetch_all
8. Replace execute with sqlx execute
9. Add .await to all database calls
10. Update boolean comparisons
"""

import re
import sys

def migrate_rusqlite_to_sqlx(content):
    """Main migration function"""

    # 1. Remove all rusqlite imports
    content = re.sub(r'^[\ \t]*use rusqlite::.*?;\n', '', content, flags=re.MULTILINE)

    # 2. Remove std::path::Path imports in SSR sections (replace with empty line to keep structure)
    content = re.sub(r'^[\ \t]*use std::path::Path;\n', '', content, flags=re.MULTILINE)

    # 3. Replace Connection::open patterns
    # Pattern: let db_path = Path::new("tatteau.db");\n        let conn = Connection::open(db_path)?;
    content = re.sub(
        r'let db_path = Path::new\("tatteau\.db"\);\s*\n\s*let conn = Connection::open\(db_path\)\?;',
        'let pool = crate::db::pool::get_pool();',
        content
    )
    content = re.sub(
        r'let db_path = Path::new\("tatteau\.db"\);\s*\n\s*let conn = Connection::open\(db_path\)',
        'let pool = crate::db::pool::get_pool()',
        content
    )

    # 4. Replace Result as SqliteResult -> sqlx::Result
    content = content.replace('Result as SqliteResult', 'sqlx::Result')
    content = content.replace('SqliteResult<', 'sqlx::Result<')

    # 5. Make helper functions async
    # Pattern: fn name(...) -> sqlx::Result<T> {
    content = re.sub(
        r'(\s+)fn\s+([a-z_]+)\s*\(([^\)]*)\)\s*->\s*sqlx::Result<',
        r'\1async fn \2(\3) -> sqlx::Result<',
        content
    )

    # 6. Replace parameter placeholders (go backwards to avoid ?10 -> $10 -> $$10)
    for i in range(20, 0, -1):
        content = content.replace(f'?{i}', f'${i}')

    # Now handle standalone ? which should become $1 (but be careful)
    # This is tricky - we need context. For now, skip.

    # 7. Replace conn.prepare() patterns
    content = content.replace('conn.prepare(', 'sqlx::query(')
    content = content.replace('let mut stmt = sqlx::query(', 'let rows = sqlx::query(')

    # 8. Replace conn.execute() patterns
    # This is complex as execute syntax is different in sqlx
    # conn.execute("SQL", params![a, b]) -> sqlx::query!("SQL").bind(a).bind(b).execute(pool).await?

    # 9. Replace boolean comparisons
    content = re.sub(r'=\s*1([^0-9])', r'= true\1', content)
    content = re.sub(r'!=\s*1([^0-9])', r'!= true\1', content)
    content = content.replace('is_recurring = true', 'is_recurring = 1')  # Keep SQL as-is
    content = content.replace('active = true', 'active = 1')  # Keep SQL as-is

    # 10. Replace rusqlite error types
    content = content.replace('rusqlite::Error', 'sqlx::Error')
    content = content.replace('rusqlite::params', 'sqlx::query')

    # 11. Add .await after database operations (this needs careful handling)
    # We'll skip for now as it's too complex without AST parsing

    return content

def main():
    input_file = '/home/chris/code/personal/tatteau/web/src/server.rs.backup'
    output_file = '/home/chris/code/personal/tatteau/web/src/server_migrated.rs'

    print(f"Reading {input_file}...")
    with open(input_file, 'r') as f:
        content = f.read()

    print(f"Original file: {len(content)} characters, {content.count(chr(10))} lines")

    print("Performing migration...")
    migrated = migrate_rusqlite_to_sqlx(content)

    print(f"Migrated file: {len(migrated)} characters, {migrated.count(chr(10))} lines")

    with open(output_file, 'w') as f:
        f.write(migrated)

    print(f"\nMigration written to {output_file}")
    print("\nNOTE: This is a partial migration. Manual review and completion required for:")
    print("  - Adding .await to all database calls")
    print("  - Converting query_row to fetch_one")
    print("  - Converting query_map to fetch_all")
    print("  - Converting execute to sqlx execute patterns")
    print("  - Replacing row.get(0) with row.get::<Type, _>(\"column_name\")")
    print("  - Converting helper function calls to await")

if __name__ == '__main__':
    main()
