# moka

Last updated: 2026-02-12

## Usage in MCB

- High-performance in-memory caching for frequently accessed data.
- Async cache integration in provider/infrastructure paths.

## Key Capabilities in Use

- `moka::future::Cache` for async workflows.
- TTL and TTI expiration controls.
- Capacity-based eviction policies.

## Best Practices

1. Always set `max_capacity` to avoid unbounded memory growth.
2. Use TTL/TTI based on data freshness requirements.
3. Use loader patterns (`get_with`) to reduce cache stampedes.

## Common Pitfalls

- Infinite cache lifetime causes stale data and memory issues.
- Missing invalidation strategy in multi-source data flows.

## Official References

- https://docs.rs/moka

## GitHub References

- https://github.com/moka-rs/moka/blob/main/examples/async_example.rs
- https://github.com/ben-manes/caffeine
