# tracing

Last updated: 2026-02-12

## Usage in MCB

- Structured logging across server, infrastructure, providers, and validators.
- Instrumentation with spans and contextual fields.
- Integration with `tracing-subscriber` and optional file appenders.

## Key Capabilities in Use

- Level-based events (`error`, `warn`, `info`, `debug`, `trace`).
- Span-based execution context with `#[instrument]`.
- Environment-based filtering via `RUST_LOG` / `EnvFilter`.

## Best Practices

1. Log structured fields (`service`, `operation`, `entity_id`) instead of formatted strings.
2. Use `skip_all` or selective skip in `#[instrument]` to avoid expensive captures.
3. Prefer non-blocking appenders in async runtimes.
4. Keep log volume controlled on hot paths.

## Common Pitfalls

- Unbounded logs in tight loops.
- Accidental leakage of sensitive values in debug output.
- Blocking log sinks in async execution paths.

## Official References

- https://docs.rs/tracing
- https://docs.rs/tracing-subscriber
- https://tokio.rs/tokio/topics/tracing

## GitHub References

- https://github.com/availproject/avail-light/blob/main/fat/src/main.rs
- https://github.com/netdata/netdata/blob/master/src/crates/netdata-log-viewer/journal-viewer-plugin/src/main.rs
