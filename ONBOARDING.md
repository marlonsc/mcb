# Onboarding Guide: MCB (Memory Context Browser)

## Overview

MCB is a high-performance **Model Context Protocol (MCP) server** written in Rust.
It gives AI coding agents persistent memory, semantic code search, and deep
repository understanding through the standard MCP protocol. Think of it as a
long-term memory and search backend that Claude, Cursor, or any MCP client can
query while working on code.

## Tech Stack

| Layer | Technology | Version / Notes |
| ----- | --------- | --------------- |
| Language | Rust | 1.92+ (edition 2024) |
| Async runtime | Tokio | 1.x |
| Web framework | Axum + Tower | 0.8 / 0.5 |
| App framework | Loco.rs | 0.16.4 (forked in `third-party/`) |
| ORM / DB | SeaORM + SeaQuery | 2.0.0-rc.38 (forked in `third-party/`) |
| Databases | SQLite (default), PostgreSQL (runtime selectable) | via SeaORM |
| Vector stores | Milvus, EdgeVec, Qdrant, Pinecone, Encrypted | provider model |
| Embeddings | FastEmbed, Ollama, OpenAI, VoyageAI, Gemini, Anthropic | provider model |
| MCP SDK | rmcp | 1.4 |
| Protocol | MCP 2024-11-05 | JSON-RPC 2.0 over stdio or HTTP |
| Testing | cargo test / cargo-nextest, insta, mockall, Playwright (E2E) | - |
| CI/CD | GitHub Actions | `.github/workflows/ci.yml` |
| Build | Make + Cargo | `Makefile` is the canonical interface |

## Architecture

MCB is a **Rust workspace monorepo** with strict **Clean Architecture**
dependency rules. The 7 first-party crates form inward-only layers:

```text
┌─────────────────────────────────────────────┐
│  crates/mcb          CLI binary / Loco app  │
│       │                                     │
│       ▼                                     │
│  crates/mcb-server   MCP protocol & handlers│
│       │                                     │
│       ▼                                     │
│  crates/mcb-infra    DI, config, cache, log │
│       │                                     │
│       ▼                                     │
│  crates/mcb-domain   entities, ports, errors│
│       │                                     │
│       ▼                                     │
│  crates/mcb-utils    leaf utilities         │
└─────────────────────────────────────────────┘
         ▲                        ▲
    crates/mcb-providers     crates/mcb-validate
  (adapters: DB, embeddings,  (architecture rule engine)
   vector store, git, parsers)
```

- **mcb-domain** has **zero** internal `mcb-*` dependencies; it defines ports
  (traits) and entities.
- **mcb-providers** implements domain ports and depends only on `mcb-domain` +
  `mcb-utils`.
- **mcb-infrastructure** wires everything together via an `AppContext`
  composition root and `linkme` distributed slices for provider discovery.
- **mcb-server** exposes 24 MCP tools grouped into 9 handler families.
- **mcb-validate** enforces architecture rules (layer boundaries, forbidden
  imports, etc.).

## Key Entry Points

| Entry | Path | Purpose |
| ----- | ---- | ------- |
| CLI binary | `crates/mcb/src/main.rs` | `mcb serve` and `mcb validate` subcommands |
| Loco app hooks | `crates/mcb/src/loco_app.rs` | Application boot, initializers, lifecycle |
| MCP server bootstrap | `crates/mcb/src/initializers/mcp_server.rs` | Composes the MCP server into Loco |
| Server crate | `crates/mcb-server/src/lib.rs` | Handlers, tools, transport, auth, composition |
| Domain ports | `crates/mcb-domain/src/ports/` | Contracts for repositories, providers, services |
| Domain entities | `crates/mcb-domain/src/entities/` | Core data models (memory, code, projects, sessions) |
| Provider adapters | `crates/mcb-providers/src/` | SeaORM repositories, embedding clients, vector stores |
| Validation rules | `config/mcb-validate.toml` | Architecture boundary checks |
| Runtime config | `config/development.yaml`, `config/test.yaml`, `config/production.yaml` | Loco + MCB-specific settings |

## Directory Map

| Directory | Purpose |
| --------- | ------- |
| `crates/` | 7 first-party workspace crates (see Architecture) |
| `config/` | Loco runtime configs + validation rules |
| `docs/` | Architecture docs, ADRs, MCP tool schema, operations guides |
| `tests/` | Golden/integration tests, E2E Playwright tests, fixtures |
| `scripts/` | Build/release helpers, hooks, codegen, docs generation |
| `makefiles/` | Make dispatch macros (`dispatch.mk`, `ui.mk`) |
| `third-party/` | Forked dependencies (SeaORM, Loco, EdgeVec, etc.) — **do not edit** |
| `book/` / `book.toml` | mdBook documentation site |
| `k8s/`, `systemd/`, `Dockerfile` | Deployment artifacts |
| `assets/admin/` | Static admin UI served by Axum |

## Request Lifecycle

A typical MCP tool call flows like this:

1. **Transport** (`crates/mcb-server/src/transport/`) receives JSON-RPC over
   stdio or HTTP.
2. **Tools registry** (`crates/mcb-server/src/tools/`) maps the 24 public tool
   names to the 9 handler families.
3. **Handler** (`crates/mcb-server/src/handlers/`) validates arguments with the
   schema in `crates/mcb-server/src/args/`, then calls a domain service through
   a port trait.
4. **Service / use case** (`crates/mcb-domain/src/ports/services/`) orchestrates
   business logic.
5. **Repository / provider adapter** (`crates/mcb-providers/src/`) talks to
   SQLite/PostgreSQL (SeaORM), vector store, embedding service, or git.
6. **Response** is formatted and returned through the MCP transport.

The composition root in `crates/mcb-server/src/composition.rs` resolves concrete
adapters from the Loco `AppContext` so handlers never import providers directly.

## Conventions

- **File naming**: Rust modules use `snake_case.rs`; test files mirror source
  paths or use `tests/<scope>/<name>_<kind>.rs`.
- **Module naming**: `mcb_domain::entities::project`, `mcb_server::handlers`.
- **Types**: Entities are `PascalCase`; value objects live in
  `crates/mcb-domain/src/value_objects/`.
- **Error handling**: `thiserror`-based `mcb_domain::error::Error` + `Result<T>`
  alias. Production code avoids `unwrap`/`expect`; clippy warns on them.
- **Async**: `async/await` on Tokio; repository/provider traits are `Send + Sync`.
- **Dependency injection**: Prefer domain ports + `AppContext` injection; do not
  import concrete providers from handlers.
- **Provider discovery**: Use `linkme` distributed slices (`register_tool!`,
  provider registry macros) rather than manual inventories.
- **Linting**: Very strict clippy lints in `Cargo.toml` (`workspace.lints.clippy`).
- **Git workflow**: Conventional commits (`feat`, `fix`, `refactor`, `docs`,
  `test`, `chore`, `perf`, `ci`). Pre-commit/pre-push hooks run `guard`, fmt,
  clippy, tests, and `validate`.
- **Task tracking**: Work is tracked in **beads** (`bd`). Run `bd ready` to find
  actionable items and `bd update <id> --claim` before editing.

## Common Tasks

| Task | Command |
| ---- | ------- |
| Build debug | `make build` |
| Build release | `make build RELEASE=1` |
| Run dev server | `make dev WHAT=run` |
| Run all tests | `make test` |
| Run unit tests only | `make test SCOPE=unit` |
| Run golden tests | `make test SCOPE=golden` |
| Lint + format check | `make check WHAT=lint` |
| Architecture validation | `make check WHAT=validate` |
| Full CI gate | `make ci` or `make check WHAT=all` |
| Auto-fix formatting | `make fix WHAT=fmt` |
| Docs lint | `make docs WHAT=lint` |
| Banned-pattern scan | `make guard` |
| Pre-commit hook | `make hook WHAT=pre-commit` |

## Where to Look

| I want to... | Look at... |
| ------------ | --------- |
| Add or change an MCP tool | `crates/mcb-server/src/args/`, `crates/mcb-server/src/handlers/`, `docs/MCP_TOOLS.md` |
| Add a domain entity | `crates/mcb-domain/src/entities/` |
| Add a repository / DB access | `crates/mcb-domain/src/ports/repositories/` + `crates/mcb-providers/src/database/seaorm/` |
| Add an embedding or vector provider | `crates/mcb-domain/src/ports/providers/` + `crates/mcb-providers/src/` |
| Change architecture rules | `config/mcb-validate.toml` + `crates/mcb-validate/src/` |
| Change runtime config | `config/development.yaml`, `config/test.yaml`, `config/production.yaml` + `crates/mcb-infrastructure/src/config.rs` |
| Add a test | Matching crate `tests/` directory or `tests/golden/` for end-to-end MCP scenarios |
| Update docs | `docs/` and `book/src/`; run `make docs WHAT=lint` |

## Next Steps

1. Read [`AGENTS.md`](./AGENTS.md) — it is the single source of truth for
   agent rules, architecture, commands, beads workflow, and Git policy.
2. Read [`README.md`](./README.md) for a high-level feature overview.
3. Read [`docs/architecture/ARCHITECTURE.md`](./docs/architecture/ARCHITECTURE.md)
   for the full architectural picture.
4. Run `make build` and `make test SCOPE=unit` to verify your environment.
