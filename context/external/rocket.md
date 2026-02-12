# rocket

Last updated: 2026-02-12

## Executive Summary

Rocket is MCB's HTTP framework for admin API, web UI routes, SSE streams, and MCP-over-HTTP transport. It is the standard web boundary in `mcb-server`.

## Context7 + External Research

- Context7 ID: `/websites/rs_rocket`
- Guide: https://rocket.rs/guide/v0.5
- API docs: https://docs.rs/rocket/latest/rocket/
- Upstream: https://github.com/rwf2/Rocket

## Actual MCB Usage (Current Source of Truth)

### 1) Transport bootstrap and config wiring

- `crates/mcb-server/src/transport/http.rs:45`
- `crates/mcb-server/src/transport/http.rs:167`
- `crates/mcb-server/src/transport/http.rs:169`

Pattern: Rocket app is built with `rocket::custom(figment)` and mounted routes/fairings.

### 2) Admin routes and handler surface

- `crates/mcb-server/src/admin/routes.rs:8`
- `crates/mcb-server/src/admin/handlers.rs:26`
- `crates/mcb-server/src/admin/config_handlers.rs:10`
- `crates/mcb-server/src/admin/lifecycle_handlers.rs:16`

Pattern: `#[get]`/`#[post]` handlers use `State<T>` injection and `Json<T>` responses.

### 3) Guards, forms, and response types

- `crates/mcb-server/src/admin/auth.rs:20`
- `crates/mcb-server/src/admin/web/filter.rs:3`
- `crates/mcb-server/src/admin/web/entity_handlers.rs:5`

Pattern: request guards, form parsing, and status-aware responses are centralized in web/admin boundaries.

### 4) Streaming and middleware-like fairings

- `crates/mcb-server/src/admin/sse.rs:34`
- `crates/mcb-server/src/templates/fairing.rs:1`
- `crates/mcb-server/src/templates/template.rs:4`

Pattern: SSE and fairing hooks provide cross-cutting behavior without leaking into domain logic.

## ADR Alignment (Critical)

- ADR-026 (`docs/adr/026-routing-refactor-rocket-poem.md`): Rocket selected over alternatives for DX and cohesive framework model.
- ADR-011 (`docs/adr/011-http-transport-request-response-pattern.md`): transport pattern implemented on Rocket.
- ADR-007 (`docs/adr/007-integrated-web-administration-interface.md`): admin interface is Rocket-based.

## GitHub Evidence (Upstream + In-Repo)

- Upstream Rocket: https://github.com/rwf2/Rocket
- Rocket guide: https://rocket.rs/guide/v0.5
- Production example (Vaultwarden): https://github.com/dani-garcia/vaultwarden/blob/main/src/main.rs
- In-repo anchor: `crates/mcb-server/src/admin/routes.rs:69`
- In-repo anchor: `crates/mcb-server/src/transport/http.rs:281`

## Best Practices in MCB

### Handler design

Handlers in MCB follow a thin-dispatch pattern: route functions parse the incoming request, delegate to an application-layer service, and format the response. This keeps route definitions declarative and testable.

All admin handlers are co-located in `crates/mcb-server/src/admin/handlers.rs` and share a common `State<T>` injection pattern for accessing domain services.

### State injection

Rocket's `State<T>` is the primary DI surface in the server crate. Services registered through `dill`/`linkme` are exposed to handlers via managed state, ensuring a single instance per dependency.

Avoid constructing service instances inside handlers; always inject them through state.

Cross-reference: `context/external/dill.md` for IoC composition, `context/external/linkme.md` for provider registration.

### Request guard discipline

Guards (auth checks, content-type enforcement, entity resolution) should be centralized and reused, not inlined per-handler.

MCB centralizes auth guards in `crates/mcb-server/src/admin/auth.rs:20`. Any new admin endpoint must use the same guard chain.

### Response consistency

All JSON endpoints return `Json<T>` wrappers. Error responses use the server's unified error mapping (`crates/mcb-server/src/error_mapping.rs`), which maps domain `Error` variants to HTTP status codes.

Cross-reference: `context/external/thiserror.md` for the error taxonomy, `context/external/serde.md` for serialization conventions.

## Performance and Safety Considerations

### Blocking in handlers

Rocket handlers run on Tokio's async runtime. Blocking I/O (file system, subprocess, git2 calls) must be offloaded with `tokio::task::spawn_blocking` or run in a dedicated thread pool.

MCB already offloads tree-sitter parsing and git2 operations in provider implementations. Any new handler that introduces blocking I/O must follow the same pattern.

Cross-reference: `context/external/tokio.md` (blocking-boundary guidance), `context/external/git2.md`.

### Request body limits

Rocket applies body data limits per content type. MCB configures these through `figment` (`crates/mcb-infrastructure/src/config/loader.rs`). Ensure new endpoints with large payloads declare explicit limits.

### Concurrency model

Rocket 0.5 runs on Tokio and shares the same multi-threaded runtime configured in `crates/mcb/src/main.rs:52`. Handler concurrency is limited by Tokio worker threads, not Rocket-specific pools.

## Testing and Verification Guidance

### Integration test pattern

MCB tests Rocket handlers through the `rocket::local::asynchronous::Client` API in `crates/mcb-server/tests/`. Tests construct a minimal Rocket instance with mock services, then dispatch requests against it.

Example anchors:
- `crates/mcb-server/tests/unit/builder_tests.rs:33` (builder-based test app construction)
- `crates/mcb-server/tests/integration/operating_modes_integration.rs:583` (full integration setup)

### Testing guards

Request guards should have standalone unit tests separate from handler tests. This prevents guard logic from being validated only through end-to-end flows.

### Verifying route mounting

If a route is defined but not mounted in `crates/mcb-server/src/transport/http.rs`, it silently does nothing. Always verify new routes appear in the mount list.

## Operational Risk and Monitoring

| Risk | Impact | Mitigation |
|---|---|---|
| Blocking call in handler | Thread starvation, latency spikes | Offload via `spawn_blocking`; validate in code review |
| Unguarded endpoint | Auth bypass | Centralized guard chain in `admin/auth.rs` |
| Missing route mount | Silent endpoint absence | Integration test coverage; explicit mount assertion |
| Large body without limit | Memory exhaustion | Configure data limits per content type |
| Panic in handler | 500 with no context | Error mapping layer catches domain errors before panic |

Cross-reference: `context/external/tracing.md` for observability and request-level span policy.

## Migration and Version Notes

- MCB uses Rocket 0.5 (current stable).
- Rocket 0.5 replaced the custom runtime with native Tokio support, which MCB leverages.
- ADR-026 (`docs/adr/026-routing-refactor-rocket-poem.md`) documents the decision to stay on Rocket over alternatives (Poem, Axum) for DX and cohesive framework model.
- Any migration to Axum/Poem would require rewriting guards, fairings, state injection, and test infrastructure.

## Verification Checklist

- [ ] New handler uses `State<T>` injection, not manual construction
- [ ] Auth guard applied to all admin endpoints
- [ ] No blocking I/O in handler body (offloaded to `spawn_blocking`)
- [ ] Response type uses `Json<T>` or status-aware wrapper
- [ ] Route mounted in `transport/http.rs`
- [ ] Integration test exercises the new endpoint
- [ ] Error path returns domain error mapped through `error_mapping.rs`
- [ ] Body data limits configured for large-payload endpoints

## Common Pitfalls

- Performing blocking I/O directly in handlers instead of async/offloaded paths.
- Putting orchestration/business logic in route functions instead of service layer.
- Inconsistent guard usage that bypasses centralized auth/validation behavior.
- Forgetting to mount a route after defining it (silent failure).
- Returning raw strings instead of typed `Json<T>` wrappers for API endpoints.

## References

- https://rocket.rs/guide/v0.5
- https://docs.rs/rocket/latest/rocket/
- https://github.com/rwf2/Rocket
- `docs/adr/026-routing-refactor-rocket-poem.md`
- `docs/adr/011-http-transport-request-response-pattern.md`
- `docs/adr/007-integrated-web-administration-interface.md`
- `context/external/tokio.md`
- `context/external/dill.md`
- `context/external/thiserror.md`
- `context/external/serde.md`
- `context/external/tracing.md`
