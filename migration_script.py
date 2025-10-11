#!/usr/bin/env python3
"""
Script to migrate Rust database functions from rusqlite to sqlx (PostgreSQL)
"""

import re
import sys

def migrate_function(func_text):
    """Migrate a single function from rusqlite to sqlx"""

    # Replace function signature - add async
    func_text = re.sub(r'pub fn (\w+)', r'pub async fn \1', func_text)

    # Replace SqliteResult with DbResult
    func_text = func_text.replace('SqliteResult', 'DbResult')
    func_text = func_text.replace('-> Result<', '-> DbResult<')

    # Remove rusqlite imports within functions
    func_text = re.sub(r'\s*use rusqlite::\{?[^}]*\}?;?\s*', '\n', func_text)

    # Replace database connection
    func_text = re.sub(
        r'let db_path = get_db_path\(\);?\s*let conn = Connection::open\(db_path\)\?;',
        'let pool = crate::db::pool::get_pool();',
        func_text
    )

    # Replace prepare statements with sqlx::query
    func_text = re.sub(
        r'let mut stmt = conn\.prepare\(\s*"([^"]*(?:"[^"]*")*[^"]*)"[^)]*\)\?;',
        lambda m: f'// Converted to sqlx::query\n    let query = "{m.group(1)}";',
        func_text,
        flags=re.DOTALL
    )

    # Replace parameter placeholders ?1, ?2, etc. with $1, $2, etc.
    for i in range(20, 0, -1):  # Go backwards to avoid replacing ?10 before ?1
        func_text = func_text.replace(f'?{i}', f'${i}')

    # Replace standalone ? with numbered placeholders
    # This is tricky - we need to number them sequentially

    return func_text

def main():
    if len(sys.argv) != 3:
        print("Usage: python migration_script.py <input_file> <output_file>")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]

    with open(input_file, 'r') as f:
        content = f.read()

    # Migrate imports at top of file
    content = re.sub(
        r'use rusqlite::\{Connection, Result as SqliteResult\};',
        'use sqlx::{PgPool, Row};\ntype DbResult<T> = Result<T, sqlx::Error>;',
        content
    )

    # Remove get_db_path function
    content = re.sub(
        r'/// Get the database path.*?fn get_db_path\(\).*?\n\}',
        '',
        content,
        flags=re.DOTALL
    )

    # Find and migrate each function
    # This is a simple version - more sophisticated parsing would be needed for production

    with open(output_file, 'w') as f:
        f.write(content)

    print(f"Migration complete. Output written to {output_file}")

if __name__ == '__main__':
    main()
