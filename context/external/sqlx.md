# SQLx Quick Reference

## Use in This Repo

- SQLite persistence layer
- migrations and schema management
- typed query execution

## Preferred Patterns

- Keep migrations deterministic and idempotent.
- Use explicit transaction boundaries for multi-step writes.
- Prefer parameterized queries and clear error mapping.

## Validation Reminders

- Verify schema compatibility on startup.
- Check migration ordering in CI.
- Keep query paths observable with trace context.

> Updated 2026-02-12
