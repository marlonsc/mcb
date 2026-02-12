# SQLx

Last updated: 2026-02-12

## Usage in MCB

SQLx powers async SQLite persistence for sessions, memory, indexing metadata, and administrative entities.

- Internal examples: `crates/mcb-providers/src/database/sqlite/provider.rs`, `crates/mcb-providers/src/database/sqlite/executor.rs`.

## Key Capabilities in Use

- Runtime queries with `query` / `query_as`.
- Pool-based connection management via `SqlitePool`.
- Transaction boundaries for multi-step operations.
- Repository-oriented abstraction around executors.

## Best Practices

1. Use bind parameters for every dynamic value.
2. Keep transactions explicit and short-lived.
3. Validate schema and migration order during bootstrap.
4. Cover runtime queries with integration tests.

## Common Pitfalls

- Runtime query validation shifts errors from compile-time to runtime.
- Long-lived transactions can serialize SQLite writes.
- Missing WAL/tuning settings can degrade concurrent read/write behavior.

## Official References

- https://docs.rs/sqlx
- https://github.com/launchbadge/sqlx
- https://docs.rs/sqlx/latest/sqlx/sqlite/index.html

## GitHub References

- https://github.com/launchbadge/sqlx/blob/main/tests/sqlite/sqlite.rs
- https://github.com/openai/codex/blob/main/codex-rs/state/src/runtime.rs
