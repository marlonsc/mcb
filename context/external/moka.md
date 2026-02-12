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

## Best Practices in MCB

### Bounded capacity always

Every Moka cache instance in MCB must set `max_capacity`. The cache provider (`crates/mcb-providers/src/cache/moka.rs:56`) configures capacity at construction time. An unbounded cache is a memory leak waiting to happen.

### TTL policy

MCB uses TTL-based expiration for cached data that has a known freshness window. The `MokaCacheProvider` supports configurable TTL at construction. Choose TTL based on the volatility of the cached data:
- Index metadata: longer TTL (minutes)
- Search results: shorter TTL (seconds)
- Configuration: medium TTL or event-driven invalidation

### Explicit invalidation on writes

When the source of truth changes (e.g., re-indexing, configuration update), the cache must be explicitly invalidated. Do not rely solely on TTL for correctness — TTL only bounds staleness.

MCB's cache provider exposes `invalidate` and `clear` operations through the `CacheProvider` trait.

Cross-reference: `context/external/async-trait.md` for the port interface pattern.

### Serialization format

MCB caches values as `Vec<u8>` (`moka::future::Cache<String, Vec<u8>>`). Callers serialize/deserialize through the `CacheProvider` trait boundary. This keeps the cache provider agnostic to value types.

Cross-reference: `context/external/serde.md` for serialization conventions.

### DI registration

The Moka provider is registered through MCB's provider registry and selected at runtime. The `CacheProviderResolver` (`crates/mcb-infrastructure/src/di/`) handles provider selection and initialization.

Cross-reference: `context/external/dill.md` for IoC composition, `context/external/linkme.md` for auto-registration.

## Performance and Safety Considerations

### Async cache API

MCB uses `moka::future::Cache` (the async-compatible variant). This integrates naturally with Tokio without blocking worker threads. The sync `moka::sync::Cache` is not used.

### Eviction timing

Moka performs lazy eviction — entries are not removed at exact TTL expiry. Eviction happens on subsequent access or background maintenance tasks. Do not assume cache entries disappear instantly after TTL.

### Memory estimation

Moka estimates memory usage based on entry count and configured capacity. For large cached values (e.g., serialized search results), the actual memory footprint may exceed the nominal capacity. Monitor memory usage in production.

### Concurrent access

Moka is designed for high-concurrency access. Multiple Tokio tasks can read/write the same cache instance concurrently without external locking. MCB shares cache instances through `Arc<dyn CacheProvider>`.

## Testing and Verification Guidance

### Mock cache for unit tests

MCB can test service logic without Moka by mocking the `CacheProvider` trait. This is preferred for unit tests that validate business logic rather than cache behavior.

### Integration testing with real cache

For tests that validate caching behavior (hit rates, TTL expiry, eviction), use the real `MokaCacheProvider` with a small `max_capacity` and short TTL.

### Cache miss path coverage

Always test both cache-hit and cache-miss paths. Services should degrade gracefully when the cache is empty or unavailable.

## Operational Risk and Monitoring

| Risk | Impact | Mitigation |
|---|---|---|
| Unbounded cache | Memory exhaustion | Always set max_capacity; reject configs without it |
| Stale cache after source update | Incorrect results | Explicit invalidation on writes; bounded TTL |
| Eviction storm under load | Latency spike from cold cache | Pre-warm cache on startup for known-hot keys |
| Memory underestimation | OOM in production | Monitor actual memory; adjust capacity based on value sizes |
| Cache provider misconfiguration | Silent fallback to no-cache or wrong provider | Validate provider config at startup |

Cross-reference: `context/external/tracing.md` for instrumenting cache hit/miss rates.

## Migration and Version Notes

- MCB uses moka (current stable).
- Moka's async API requires the `future` feature flag, which MCB enables.
- ADR-005 (`docs/adr/005-context-cache-support.md`) established Moka as the in-memory cache baseline.
- ADR-018 (`docs/adr/018-hybrid-caching-strategy.md`) positions Moka as the L1 cache in a multi-tier strategy.
- ADR-035 (`docs/adr/035-context-scout.md`) uses Moka TTL for context discovery freshness.
- No migration planned. Moka is actively maintained and stable.

## Verification Checklist

- [ ] `max_capacity` set on every cache instance
- [ ] TTL configured appropriate to data volatility
- [ ] Explicit invalidation triggered on source-of-truth writes
- [ ] Async cache API used (`moka::future::Cache`), not sync
- [ ] Cache miss path tested alongside cache hit path
- [ ] Memory usage monitored in production
- [ ] Provider registered through DI and selectable at runtime

## Common Pitfalls

- Omitting `max_capacity` and allowing unbounded memory growth.
- Treating TTL expiration as immediate; cleanup timing is asynchronous.
- Missing invalidation policy when underlying source of truth changes.
- Using sync cache API in async context (blocks Tokio threads).
- Caching large values without accounting for memory in capacity planning.

## References

- https://docs.rs/moka/latest/moka/
- https://github.com/moka-rs/moka
- `docs/adr/005-context-cache-support.md`
- `docs/adr/018-hybrid-caching-strategy.md`
- `docs/adr/035-context-scout.md`
- `context/external/async-trait.md`
- `context/external/dill.md`
- `context/external/linkme.md`
- `context/external/serde.md`
- `context/external/tracing.md`
- `context/external/tokio.md`
