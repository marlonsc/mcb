# Rocket

Last updated: 2026-02-12

## Usage in MCB

Rocket (0.5) powers HTTP transport, admin endpoints, and web-facing handlers.

- Internal examples: `crates/mcb-server/src/transport/http.rs`, `crates/mcb-server/src/admin/routes.rs`.

## Key Capabilities in Use

- Async route handlers with typed request/response models.
- Managed state via `State<T>`.
- Fairings for cross-cutting behavior (headers, templates, telemetry).
- Request guards for authentication and validation.

## Best Practices

1. Keep handlers thin; delegate business logic to services.
2. Use request guards for centralized auth/validation.
3. Keep shared mutable state behind safe synchronization primitives.
4. Keep transport concerns out of domain/application layers.

## Common Pitfalls

- Blocking work in async handlers.
- Mutable shared state without synchronization.
- Business logic implemented directly in route functions.

## Official References

- https://rocket.rs/v0.5/guide/
- https://docs.rs/rocket

## GitHub References

- https://github.com/rwf2/Rocket/blob/master/core/lib/src/lib.rs
- https://github.com/rwf2/Rocket/blob/master/core/lib/src/state.rs
