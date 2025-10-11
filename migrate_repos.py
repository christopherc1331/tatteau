#!/usr/bin/env python3
"""
Automated migration script from rusqlite to sqlx (PostgreSQL)
"""

import re

def convert_param_placeholders(sql):
    """Convert ?1, ?2, ... to $1, $2, ... and standalone ? to numbered placeholders"""
    # First convert numbered placeholders
    for i in range(20, 0, -1):
        sql = sql.replace(f'?{i}', f'${i}')

    # Then convert remaining ? to numbered placeholders
    counter = 1
    result = []
    i = 0
    while i < len(sql):
        if sql[i] == '?':
            # Check if it's already numbered (like ?1) - shouldn't happen after above
            if i + 1 < len(sql) and sql[i+1].isdigit():
                result.append(sql[i])
            else:
                # Find the next available number
                while f'${counter}' in sql:
                    counter += 1
                result.append(f'${counter}')
                counter += 1
        else:
            result.append(sql[i])
        i += 1
    return ''.join(result)

def migrate_file(input_path, output_path):
    with open(input_path, 'r') as f:
        content = f.read()

    # Replace imports
    content = re.sub(
        r'#\[cfg\(feature = "ssr"\)\]\s*use rusqlite::\{Connection,\s*(?:params,\s*)?Result as SqliteResult\};',
        '#[cfg(feature = "ssr")]\nuse sqlx::{PgPool, Row};',
        content
    )

    content = re.sub(
        r'#\[cfg\(feature = "ssr"\)\]\s*use rusqlite::\{Connection,\s*Result as SqliteResult\};',
        '#[cfg(feature = "ssr")]\nuse sqlx::{PgPool, Row};',
        content
    )

    content = re.sub(
        r'use rusqlite::\{Connection,\s*params\};',
        'use sqlx::{PgPool, Row};',
        content
    )

    # Add type alias after imports
    if 'type DbResult' not in content:
        content = re.sub(
            r'(#\[cfg\(feature = "ssr"\)\]\s*use sqlx::\{PgPool, Row\};)',
            r'\1\n#[cfg(feature = "ssr")]\ntype DbResult<T> = Result<T, sqlx::Error>;',
            content,
            count=1
        )

    # Remove get_db_path function
    content = re.sub(
        r'/// Get the database path.*?\nfn get_db_path\(\).*?\n\}',
        '',
        content,
        flags=re.DOTALL
    )

    # Replace SqliteResult with DbResult
    content = content.replace('SqliteResult', 'DbResult')
    content = content.replace('rusqlite::Result', 'DbResult')
    content = content.replace('rusqlite::Error', 'sqlx::Error')

    # Replace pub fn with pub async fn
    content = re.sub(
        r'(#\[cfg\(feature = "ssr"\)\]\s*pub )fn ',
        r'\1async fn ',
        content
    )

    # Remove internal use rusqlite lines
    content = re.sub(r'\s*use rusqlite::\{?[^};\n]*\}?;?\s*\n', '\n', content)
    content = re.sub(r'\s*use rusqlite::params;?\s*\n', '\n', content)

    # Replace database connection pattern
    content = re.sub(
        r'let db_path = get_db_path\(\);\s*(?:let|\/\/ Open a connection.*?\n\s*let) conn = (?:rusqlite::)?Connection::open\(db_path\)\?;',
        'let pool = crate::db::pool::get_pool();',
        content,
        flags=re.DOTALL
    )

    content = re.sub(
        r'let db_path = get_db_path\(\);[\s\S]*?let conn = Connection::open\(db_path\)\?;',
        'let pool = crate::db::pool::get_pool();',
        content
    )

    # Replace conn.execute patterns
    content = re.sub(
        r'conn\.execute\s*\(\s*"([^"]*)",\s*params!\[([^\]]*)\]\s*\)\?',
        lambda m: f'sqlx::query("{convert_param_placeholders(m.group(1))}")\n        .bind({m.group(2).replace(", ", ")\n        .bind(")})\n        .execute(pool)\n        .await?',
        content
    )

    # Replace last_insert_rowid
    content = content.replace('conn.last_insert_rowid()', 'result.last_insert_id() as i64')

    # Convert parameter placeholders in SQL strings
    content = convert_param_placeholders(content)

    with open(output_path, 'w') as f:
        f.write(content)

    print(f"Migrated {input_path} to {output_path}")

if __name__ == '__main__':
    import sys
    if len(sys.argv) != 3:
        print("Usage: python migrate_repos.py <input_file> <output_file>")
        sys.exit(1)

    migrate_file(sys.argv[1], sys.argv[2])
