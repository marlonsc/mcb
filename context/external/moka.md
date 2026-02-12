# moka

Last updated: 2026-02-12

## Executive Summary

Moka provides MCB's in-memory cache provider implementation, enabling low-latency lookups with bounded capacity and optional TTL behavior.

## Context7 + External Research

- Context7 ID: `/websites/rs_moka`
- Main docs: https://docs.rs/moka/latest/moka/
- Upstream: https://github.com/moka-rs/moka
- Async API docs: https://docs.rs/moka/latest/moka/future/struct.Cache.html

## Actual MCB Usage (Current Source of Truth)

### 1) Core provider implementation

- `crates/mcb-providers/src/cache/moka.rs:25`
- `crates/mcb-providers/src/cache/moka.rs:56`
- `crates/mcb-providers/src/cache/moka.rs:62`
- `crates/mcb-providers/src/cache/moka.rs:78`

Pattern: `moka::future::Cache<String, Vec<u8>>` backs async `CacheProvider` operations.

### 2) Provider registration and DI exposure

- `crates/mcb-providers/src/cache/moka.rs:180`
- `crates/mcb-providers/src/cache/mod.rs:24`

Pattern: Moka provider is exported and auto-registered for runtime selection.

### 3) Context-scout and architecture references

- `docs/adr/035-context-scout.md:1258`
- `docs/adr/035-context-scout.md:1533`

Pattern: context discovery paths call out Moka-based TTL caching as a performance guardrail.

## ADR Alignment (Critical)

- ADR-005 (`docs/adr/005-context-cache-support.md`): Moka is the default in-memory cache baseline.
- ADR-018 (`docs/adr/018-hybrid-caching-strategy.md`): Moka appears in multi-tier cache strategy.
- ADR-035 (`docs/adr/035-context-scout.md`): Moka TTL is used for contextual discovery freshness.

## GitHub Evidence (Upstream + In-Repo)

- Upstream Moka: https://github.com/moka-rs/moka
- Async example: https://github.com/moka-rs/moka/blob/main/examples/basics_async.rs
- Sync/concurrency example: https://github.com/moka-rs/moka/blob/main/examples/basics_sync.rs
- In-repo anchor: `crates/mcb-providers/src/cache/moka.rs:144`

## Common Pitfalls

- Omitting `max_capacity` and allowing unbounded memory growth.
- Treating TTL expiration as immediate; cleanup timing is asynchronous.
- Missing invalidation policy when underlying source of truth changes.

## References

- https://docs.rs/moka/latest/moka/
- https://github.com/moka-rs/moka
- `docs/adr/005-context-cache-support.md`
- `docs/adr/018-hybrid-caching-strategy.md`
- `docs/adr/035-context-scout.md`
