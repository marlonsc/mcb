# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
It is the **single source of truth** for all AI agents. See [`AGENTS.md`](AGENTS.md) for the agent config index.

## Project

MCB (Memory Context Browser) — a high-performance MCP server giving AI coding agents persistent memory, semantic code search, and architecture validation. Rust edition 2024, nightly toolchain, MSRV 1.92. Version 0.2.1-dev.

## Build & Quality Commands

```bash
make build                    # Debug build (RELEASE=1 for release)
make test                     # All workspace tests (1700+)
make test SCOPE=unit          # Unit tests only (fast feedback)
make test SCOPE=golden        # Golden acceptance tests
make test SCOPE=startup       # DDL/init smoke test
make test SCOPE=integration   # Integration tests
make test SCOPE=e2e           # Playwright browser tests
make lint                     # clippy + fmt check (-D warnings)
make lint FIX=1               # Auto-fix fmt + clippy
make lint MCB_CI=1            # CI-strict (Rust 2024 lints)
make validate                 # Architecture rule enforcement (QUICK=1 for fast)
make check                    # Full gate: fmt --check + lint + test + validate
make audit                    # cargo-audit + cargo-udeps
make coverage                 # cargo-tarpaulin HTML report
make fmt                      # Format Rust + Markdown (mutating)
```

Run a single test: `cargo test -p mcb-server --test unit -- test_name`

Always use `make` targets — never raw `cargo` commands in CI.

## Architecture (Non-Negotiable)

Clean Architecture with strict inward-only dependency flow, enforced at compile time:

```
mcb (CLI facade binary)
  -> mcb-server       (MCP protocol, Rocket HTTP, handlers, admin UI)
    -> mcb-infrastructure (DI/linkme+Handle, config/Figment, cache/Moka, logging/tracing)
      -> mcb-application  (use cases, service orchestration)
        -> mcb-domain     (entities, port traits, errors — ZERO infra deps)
  -> mcb-providers    (adapters: embedding, vector store, DB, git, language parsers)
  -> mcb-validate     (rule engine, AST analysis, metrics — dev-only)
```

### Dependency Rules

- `mcb-domain`: Zero internal dependencies. No `sqlx`, `rocket`, `git2`.
- `mcb-application`: Depends only on `mcb-domain`. No direct provider usage.
- `mcb-providers`: Implements domain port traits. Depends on `mcb-domain` + `mcb-application`.
- `mcb-infrastructure`: Wires everything via DI. Can see all layers except server.
- `mcb-server`: Entry point. Accesses services via DI, never directly via providers.

### Key Patterns

**Port/Adapter**: Port traits defined in `mcb-domain/src/ports/`, adapters in `mcb-providers/`.

**DI (2-layer, ADR-050)**:
1. `linkme` distributed slices — compile-time auto-discovery of providers
2. `AppContext` — manual composition root in `mcb-infrastructure/src/di/bootstrap.rs`
3. Handle pattern (`RwLock<Arc<dyn Trait>>`) — live provider hot-swap

**Provider registration**: `#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]` with function pointer factories. `extern crate mcb_providers` in `main.rs` forces linkme registrations to link.

**MCP tools**: 9 tools registered via `linkme::distributed_slice(TOOL_DESCRIPTORS)`: `index`, `search`, `validate`, `memory`, `session`, `agent`, `project`, `vcs`, `entity`. Each tool = Args struct (`args/`) + Handler (`handlers/`) + Schema (`schemars`).

## Implementation Patterns

### Adding a New Provider

1. Define/reuse port trait in `mcb-domain/src/ports/providers/`
2. Implement adapter in `mcb-providers/src/{category}/{name}.rs`
3. Register via `#[distributed_slice(EMBEDDING_PROVIDERS)]` (or relevant slice)
4. DI auto-discovers via linkme — no manual catalog edits
5. Add config in `config/default.toml` under `[providers.{category}]`

### Adding a New MCP Tool

1. Define handler in `mcb-server/src/handlers/{tool_name}.rs`
2. Register in `mcb-server/src/tools/registry.rs` via `ToolRegistry::register()`
3. Handler receives `Arc<dyn ServicePort>` via DI — never construct services directly
4. Return `McpResult<JsonValue>` — errors propagate via `McpError`

### Adding a New Service

1. Define use case in `mcb-application/src/services/`
2. Expose via port trait in `mcb-domain/src/ports/`
3. Register in DI catalog: `mcb-infrastructure/src/di/mod.rs`
4. Inject into handlers via `Catalog::get::<dyn PortTrait>()`

## Error Handling

- **thiserror** 2.0 for domain errors, **anyhow** 1.0 for application/infra
- Single `Error` enum in `mcb-domain/src/error/types.rs` with 25+ typed variants
- Use constructors: `Error::vcs("msg")`, `Error::database("msg")` — never raw enum construction
- Propagate with `?` operator. Result alias: `pub type Result<T> = std::result::Result<T, Error>;`
- No `unwrap()`/`expect()` in production code paths (`unsafe_code = "deny"` workspace-wide)

## Workspace Lints

```toml
unsafe_code = "deny"
missing_docs = "warn"
non_ascii_idents = "deny"
dead_code = "deny"
unused_variables = "deny"
unused_imports = "deny"
```

CI adds: `-D clippy::multiple_unsafe_ops_per_block`, `-D clippy::undocumented_unsafe_blocks`

## Coding Conventions

- **Edition**: 2024 | **Formatting**: `max_width = 100`, 4 spaces, `rustfmt.toml`
- **Imports**: `std` -> external crates -> `mcb_*` workspace crates -> local modules
- **Re-exports**: Crate root re-exports (`pub use entities::*;`, `pub use error::{Error, Result};`)
- **Naming**: Types `PascalCase`, functions `snake_case`, constants `SCREAMING_SNAKE_CASE`, crates `mcb-*`
- **Traits**: `PascalCase` with suffix (`EmbeddingProvider`, `ValidationServiceInterface`)
- **Test files**: `{module}_tests.rs` in `tests/unit/` or `tests/integration/`
- **Logging**: `tracing` with structured fields: `tracing::info!(provider = %name, "Provider resolved")`
- **Async**: Tokio full + `#[async_trait]` + `futures`/`rayon` for CPU-bound

## Testing

- **Frameworks**: `#[tokio::test]`, `rstest` 0.26 (parametrized), `mockall` 0.14, `insta` 1.41 (snapshots), `serial_test` 3.2, Playwright (E2E)
- **Layout**: `crates/*/tests/unit/`, `tests/integration/`, `tests/golden/`, `tests/e2e/`
- **Helpers**: `tempfile` for filesystem, `serial_test` for global state isolation

## Quality Gates (All PRs Must Pass)

- `make lint` — zero clippy warnings, consistent formatting
- `make test` — all tests green
- `make validate` — zero architecture violations (mcb-validate)
- No `unwrap()`/`expect()` in production, no `#[allow(unused)]` hiding issues

## Architecture Violation Codes

| Code | Rule | Fix |
|------|------|-----|
| CA001 | Layer violation | Move import to correct layer |
| CA002 | Circular dependency | Use DI/ports to break cycle |
| CA007 | Port duplication | Move trait to `mcb-domain` |
| ORG016 | File placement | Move file to correct directory |
| ORG019 | Ignore pattern | Update validation config |

## Git Workflow

- **Branches**: `main` (stable), `release/v*` (active), `fix/*`, `ci/*`
- **Commits**: Conventional Commits — `feat(scope): description`, `fix:`, `refactor:`, `docs:`
- **CI**: GitHub Actions matrix (Linux/macOS/Windows x stable/beta): lint -> test -> startup-smoke -> validate -> golden -> audit -> coverage -> release-build

## Change Philosophy

- **Minimum viable change**: surgical edits, reuse existing patterns, macros, and library features
- **Never**: bypass problems, hardcode values, remove functionality, hide errors, suppress warnings
- **Always**: search exhaustively for reusable code before writing new, fix all warnings and clippy issues, leave tests passing after every change cycle
- **MVI 200**: every source file should stay under ~200 lines; split into submodules when growing
- **Config via `ConfigLoader`**: no hardcoded paths or fallback values

## Multi-Agent Coordination Doctrine

This is the **single source of truth** for how multiple agents/sessions work the same repo. `AGENTS.md`
and `.claude/skills/orchestrate/SKILL.md` point here — they do not restate it. Tracking is **beads only**
(`bd`); never TodoWrite/markdown for shared work. The execution loop is the `orchestrate` skill.

1. **Claim before edit.** Never modify a file without an owned bead (`bd update <id> --claim`). Check
   `bd ready` / `bd blocked` first. No claim → no edit. This prevents two agents competing on one file.
2. **Don't excuse yourself.** Another agent's change is not a reason to abandon your task — converge on it
   through the bead and the agreed pattern. Coordinate by beads + these rules, not by stepping aside.
3. **Never revert another agent's code.** Evolve together. Conflict → discuss in the bead (`bd update --notes`,
   `bd human`), never `git checkout`/overwrite their in-flight work. Uncommitted changes you didn't author
   are off-limits; scope your commits to your own files.
4. **Never deviate from an agreed pattern** — not even as a temporary or local fix. A deviation requires an
   explicit bead + operator sign-off (`bd human`). No silent pattern drift.
5. **Professional and honest.** Evidence before assertion — no green claim without timestamped `make check`
   output. Report failures plainly (LEI SUPREMA: resolve at the root, never hide).
6. **Manage context.** Canonical source order: this `CLAUDE.md` → `AGENTS.md` → relevant ADR → the bead.
   Use Scope/Serena and `bd memories <kw>` before re-reading whole files. Fewer sources, less divergence.
7. **No degradation.** Never permit data loss or unavailability. Anything irreversible (drop, migration,
   downtime) is **negotiated with the operator first** (breaking-glass), never assumed.
8. **Converge fast.** After tests go green, close the bead and integrate immediately. Don't let
   `in_progress`/`ready` age — `bd stale`/`bd preflight` must stay clean.
9. **Don't idle-wait.** Monitor actively; on an impasse, breaking-glass to the operator (`bd human` /
   AskUserQuestion) rather than blocking or guessing.
10. **Don't drift from the plan.** Always return to the active plan and bead. The objective lives in the
    bead; don't start unrelated work and forget the goal.

## Key Documentation (git-tracked)

- [AGENTS.md](AGENTS.md) — AI agent configuration index (all agents reference this file)
- [Architecture](docs/architecture/ARCHITECTURE.md) — layers, crate map, dependency flow
- [Clean Architecture](docs/architecture/CLEAN_ARCHITECTURE.md) — layer rules, extension guide
- [Architecture Boundaries](docs/architecture/ARCHITECTURE_BOUNDARIES.md) — dependency rules, violations
- [ADRs](docs/adr/) — 48 Architecture Decision Records
- [Contributing](docs/developer/CONTRIBUTING.md) — dev setup, coding standards, PR process
- [MCP Tools](docs/MCP_TOOLS.md) — full tool API schemas
- [Configuration](docs/CONFIGURATION.md) — all environment variables and config options
- [Roadmap](docs/developer/ROADMAP.md) — version plans and feature timeline
