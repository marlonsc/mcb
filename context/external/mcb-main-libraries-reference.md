# MCB Main Libraries Reference

Last updated: 2026-02-11
Scope: major libraries used directly in core crates.
Sources: Context7 + GitHub examples + internal code paths.

Dedicated guides:

- `context/external/tokio.md`
- `context/external/serde.md`
- `context/external/sqlx.md`
- `context/external/thiserror.md`
- `context/external/async-trait.md`
- `context/external/tracing.md`
- `context/external/figment.md`
- `context/external/linkme.md`
- `context/external/dill.md`
- `context/external/git2.md`
- `context/external/tree-sitter.md`
- `context/external/rocket.md`
- `context/external/handlebars.md`
- `context/external/rmcp.md`

## Core Libraries (What and Why)

| Library | Primary Role in MCB | Internal Hotspots |
|---|---|---|
| `tokio` | async runtime, tasks, sync primitives | `crates/mcb-server/src/transport/`, `crates/mcb-providers/src/events/` |
| `serde` / `serde_json` | serialization for domain, MCP payloads, config | `crates/mcb-domain/src/entities/`, `crates/mcb-server/src/handlers/` |
| `sqlx` | SQLite access layer and repositories | `crates/mcb-providers/src/database/sqlite/` |
| `thiserror` | typed error enums | `crates/mcb-domain/src/error/mod.rs` |
| `async-trait` | async trait interfaces (ports/providers) | `crates/mcb-domain/src/ports/` |
| `tracing` | structured logging and diagnostics | `crates/mcb-infrastructure/src/logging.rs` |
| `figment` | layered config loading (TOML + env) | `crates/mcb-infrastructure/src/config/loader.rs` |
| `linkme` | distributed registration for providers | `crates/mcb-domain/src/registry/`, `crates/mcb-providers/src/*` |
| `dill` | IoC container and runtime service wiring | `crates/mcb-infrastructure/src/di/catalog.rs` |
| `git2` | VCS operations and repository metadata | `crates/mcb-providers/src/git/git2_provider.rs` |
| `tree-sitter` (+ highlight) | parsing/chunking/highlighting | `crates/mcb-infrastructure/src/services/highlight_service.rs` |
| `rocket` | HTTP server/admin transport | `crates/mcb-server/src/transport/http.rs` |
| `handlebars` | server-side template rendering | `crates/mcb-server/src/templates/engine/handlebars_engine.rs` |
| `rmcp` | MCP protocol server/tool contracts | `crates/mcb-server/src/mcp_server.rs` |

## Best Practices by Library

- `tokio`: keep async paths non-blocking; move CPU/blocking work to `spawn_blocking`; prefer graceful shutdown strategy with timeouts.
- `sqlx`: use pooled connections, explicit transaction boundaries, and consistent migration/schema flow; avoid connection churn in hot paths.
- `rocket`: centralize state via `.manage(...)`, keep request guards narrow, and isolate transport concerns from domain services.
- `figment`: merge defaults -> override file -> env in deterministic order; validate after extraction.
- `linkme` + `dill`: register providers statically (`distributed_slice`) and resolve runtime dependencies through catalog only.
- `thiserror`: expose stable typed errors at boundaries; map external errors to domain errors early.
- `tracing`: log structured context (`service`, `action`, `result`) and avoid noisy low-value logs in loops.
- `tree-sitter`: configure language queries once and reuse highlighters/parsers when possible.
- `rmcp`: keep MCP handlers thin and route business logic to service interfaces.

## Official References

- Tokio: https://docs.rs/tokio and https://github.com/tokio-rs/tokio
- SQLx: https://docs.rs/sqlx and https://github.com/launchbadge/sqlx
- Rocket: https://docs.rs/rocket and https://github.com/rwf2/Rocket
- Serde: https://serde.rs and https://docs.rs/serde
- thiserror: https://docs.rs/thiserror
- async-trait: https://docs.rs/async-trait and https://github.com/dtolnay/async-trait
- tracing: https://docs.rs/tracing and https://docs.rs/tracing-subscriber
- figment: https://docs.rs/figment and https://github.com/SergioBenitez/Figment
- linkme: https://docs.rs/linkme and https://github.com/dtolnay/linkme
- dill: https://docs.rs/dill
- git2: https://docs.rs/git2 and https://github.com/rust-lang/git2-rs
- tree-sitter: https://docs.rs/tree-sitter and https://tree-sitter.github.io/tree-sitter/
- Handlebars (Rust): https://docs.rs/handlebars
- RMCP: https://docs.rs/rmcp
