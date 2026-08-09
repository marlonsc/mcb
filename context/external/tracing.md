# tracing Quick Reference

## Use in This Repo

- request and operation-level observability
- diagnostics for indexing/search/session workflows

## Preferred Patterns

- use structured fields for high-cardinality safety
- propagate trace context across async boundaries
- keep span names stable and domain-oriented

## Validation Reminders

- avoid PII/secrets in logs
- prevent unbounded cardinality labels
- ensure error paths include trace identifiers

> Updated 2026-02-12
