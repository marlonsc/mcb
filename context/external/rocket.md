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

## Common Pitfalls

- Performing blocking I/O directly in handlers instead of async/offloaded paths.
- Putting orchestration/business logic in route functions instead of service layer.
- Inconsistent guard usage that bypasses centralized auth/validation behavior.

## References

- https://rocket.rs/guide/v0.5
- https://docs.rs/rocket/latest/rocket/
- https://github.com/rwf2/Rocket
- `docs/adr/026-routing-refactor-rocket-poem.md`
- `docs/adr/011-http-transport-request-response-pattern.md`
