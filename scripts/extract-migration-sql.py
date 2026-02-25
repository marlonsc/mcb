#!/usr/bin/env python3
"""Extract SQL statements from SeaORM migration Rust file.

Parses execute_unprepared("...") calls and outputs each SQL statement
terminated with a semicolon, suitable for piping into sqlite3.
"""

import re
import sys


def extract_sql(migration_file: str) -> list[str]:
    with open(migration_file) as f:
        content = f.read()

    pattern = r'execute_unprepared\(\s*"((?:[^"\\]|\\.)*)"'
    return re.findall(pattern, content, re.DOTALL)


def main():
    if len(sys.argv) < 2:
        print("Usage: extract-migration-sql.py <migration.rs>", file=sys.stderr)
        sys.exit(1)

    sqls = extract_sql(sys.argv[1])
    if not sqls:
        print("No SQL statements found", file=sys.stderr)
        sys.exit(1)

    for sql in sqls:
        print(sql.strip() + ";")


if __name__ == "__main__":
    main()
