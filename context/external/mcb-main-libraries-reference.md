# MCB Main Libraries Reference

Last updated: 2026-02-12
Scope: master index for core external libraries used in MCB and their documentation anchors.
Sources: repository code paths, ADRs, official docs, and curated OSS references.

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

## How to Use This Index

1. Start here to identify the correct library guide for your change.
2. Read the specific guide before editing related code.
3. Use this file only as routing/context; keep deep guidance in per-library docs.

## Core Libraries (What and Why)

| Library | Primary Role in MCB | Internal Hotspots |
|---|---|---|
| `tokio` | async runtime, task orchestration, async sync primitives | `crates/mcb-server/src/init.rs`, `crates/mcb-application/src/use_cases/` |
| `serde` / `serde_json` | serialization for domain, transport, MCP payloads, config | `crates/mcb-domain/src/entities/`, `crates/mcb-server/src/handlers/` |
| `sqlx` | SQLite access layer, schema/bootstrap, repositories | `crates/mcb-providers/src/database/sqlite/` |
| `thiserror` | typed error taxonomy and boundary contracts | `crates/mcb-domain/src/error/mod.rs`, `crates/mcb-server/src/error_mapping.rs` |
| `async-trait` | async trait interfaces (ports/providers) | `crates/mcb-domain/src/ports/` |
| `tracing` | structured logs, instrumentation, diagnostics | `crates/mcb-infrastructure/src/logging.rs`, `crates/mcb-server/src/handlers/` |
| `figment` | layered config loading (TOML + env) | `crates/mcb-infrastructure/src/config/loader.rs` |
| `linkme` | distributed registration for providers | `crates/mcb-domain/src/registry/`, `crates/mcb-providers/src/*` |
| `dill` | IoC container and runtime service wiring | `crates/mcb-infrastructure/src/di/catalog.rs` |
| `git2` | VCS operations and repository metadata | `crates/mcb-providers/src/git/git2_provider.rs` |
| `tree-sitter` (+ highlight) | parsing/chunking/highlighting | `crates/mcb-infrastructure/src/services/highlight_service.rs` |
| `rocket` | HTTP server/admin transport | `crates/mcb-server/src/transport/http.rs` |
| `handlebars` | server-side template rendering | `crates/mcb-server/src/templates/engine/handlebars_engine.rs` |
| `rmcp` | MCP protocol server/tool contracts | `crates/mcb-server/src/mcp_server.rs` |

## Guide Status (Depth Snapshot)

| Guide | Current depth | Notes |
|---|---|---|
| `tokio.md` | expanded | runtime, concurrency, blocking-boundary guidance |
| `serde.md` | expanded | contract evolution and compatibility guidance |
| `sqlx.md` | expanded | repository boundaries and query discipline |
| `thiserror.md` | expanded | typed taxonomy and boundary mapping |
| `tracing.md` | expanded | instrumentation and logging safety |
| `rmcp.md` | expanded | protocol-layer deep analysis |
| `figment.md` | medium-high | strong ADR/context analysis |
| `linkme.md` | medium-high | registration and linker behavior |
| `dill.md` | medium-high | composition-root and IoC decisions |

## Boundary-Critical Rules (Cross-Library)

- `tokio`: no blocking work in async hot paths; isolate with `spawn_blocking` where needed.
- `sqlx`: keep SQLx in provider/infrastructure persistence boundaries; no domain leakage.
- `rocket`: maintain transport-layer concerns in server crate; avoid domain contamination.
- `figment`: keep deterministic merge precedence and post-extract validation strict.
- `linkme` + `dill`: compile-time provider registration + explicit runtime composition only.
- `thiserror`: typed contracts in libraries; map external errors early and consistently.
- `tracing`: preserve structured context, avoid sensitive-field leaks, and control log cardinality.
- `tree-sitter`: isolate heavy parsing/highlighting and guard performance-sensitive paths.
- `rmcp`: keep tool handlers thin and deterministic; maintain schema/runtime compatibility.

## Recommended Reading Order by Change Type

- Persistence or repository change: `sqlx.md` -> `thiserror.md` -> `tokio.md`
- MCP/tooling change: `rmcp.md` -> `tracing.md` -> `thiserror.md`
- Admin/API transport change: `rocket.md` -> `tracing.md` -> `serde.md`
- Provider registration/wiring change: `linkme.md` -> `dill.md` -> `figment.md`

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
