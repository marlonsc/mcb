# thiserror and anyhow

Last updated: 2026-02-12

## Usage in MCB

MCB follows a hybrid Rust error strategy:

- `thiserror` for typed, explicit library/domain errors.
- `anyhow` mainly at application/binary boundaries for aggregation and display.

## Key Capabilities in Use

- `#[derive(thiserror::Error)]` on error enums.
- `#[from]` for safe conversion from dependency errors.
- Structured context messages with `#[error("...")]`.

## Best Practices

1. Keep typed errors in libraries (`mcb-domain`, `mcb-validate`, providers).
2. Convert external errors to domain-level errors early.
3. Preserve source chains for diagnostics.
4. Prefer `?` propagation over panics in production code.

## Common Pitfalls

- Generic error text without context is hard to diagnose.
- Overusing `Box<dyn Error>` erodes type-level handling.
- Using `anyhow` deep inside library code reduces contract clarity.

## Official References

- https://docs.rs/thiserror
- https://docs.rs/anyhow
- https://rust-lang.github.io/api-guidelines/interoperability.html#error-handling

## GitHub References

- https://github.com/paradigmxyz/reth/blob/main/crates/engine/tree/src/tree/error.rs
- https://github.com/huggingface/candle/blob/main/candle-core/src/error.rs
