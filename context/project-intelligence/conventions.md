# Conventions

Last updated: 2026-02-15 (America/Sao_Paulo)

## Rust Edition & Toolchain

- **Edition**: 2024 | **Toolchain**: nightly | **MSRV**: 1.92
- **Formatting**: `rustfmt.toml` → `max_width = 100`, `tab_spaces = 4`
- **Targets**: x86_64-linux-gnu, x86_64-apple-darwin, x86_64-pc-windows-msvc

## Workspace Lints (Cargo.toml)

```toml
unsafe_code = "deny"
missing_docs = "warn"
non_ascii_idents = "deny"
dead_code = "deny"
unused_variables = "deny"
unused_imports = "deny"
```

CI adds: `-D clippy::multiple_unsafe_ops_per_block`, `-D clippy::undocumented_unsafe_blocks`

## Naming Conventions

- **Files**: `snake_case.rs` — one module per file, `mod.rs` for directories
- **Crates**: `mcb-{layer}` (mcb-domain, mcb-server, mcb-providers, etc.)
- **Structs/Enums**: `PascalCase` (`CodeChunk`, `ObservationType`, `WorkflowState`)
- **Functions**: `snake_case` (`build_catalog`, `resolve_providers`, `dispatch_tool_call`)
- **Constants**: `SCREAMING_SNAKE_CASE` (`TOOL_DESCRIPTORS`, `EMBEDDING_PROVIDERS`)
- **Traits**: `PascalCase` with `Interface`/`Provider` suffix (`EmbeddingProvider`, `ValidationServiceInterface`)
- **Test files**: `{module}_tests.rs` in `tests/unit/` or `tests/integration/`

## Error Handling

- **Library**: `thiserror` 2.0 for domain errors, `anyhow` 1.0 for application/infra
- **Pattern**: Single `Error` enum in `mcb-domain/src/error/types.rs` with typed variants
- **Constructors**: `Error::vcs("msg")`, `Error::database("msg")` — never raw enum construction
- **Propagation**: `?` operator, no `unwrap()`/`expect()` in production code paths
- **Result alias**: `pub type Result<T> = std::result::Result<T, Error>;` per crate

## Logging

- **Library**: `tracing` 0.1 + `tracing-subscriber` 0.3 (env-filter, json)
- **Levels**: `info` default, configurable via `logging.level`
- **File output**: Optional via `tracing-appender` with rotation
- **Pattern**: Structured fields: `tracing::info!(provider = %name, "Provider resolved")`

## Testing

- **Frameworks**: Built-in `#[test]` + `rstest` 0.26 (parametrized) + `mockall` 0.14 + `insta` 1.41 (snapshots)
- **E2E**: Playwright (TypeScript) for admin UI (`tests/e2e/`)
- **Organization**: `tests/unit/`, `tests/integration/`, `tests/golden/` per crate
- **Helpers**: `tempfile`, `serial_test` for isolation
- **Scopes**: `make test SCOPE=unit|golden|startup|integration|e2e|all`
- **Count**: 1700+ tests across workspace

## Build & Quality Gates

```bash
make build          # Debug (or RELEASE=1)
make test           # All tests (SCOPE=..., THREADS=N)
make lint           # clippy + fmt check (FIX=1 to auto-fix, MCB_CI=1 for strict)
make validate       # Architecture rule enforcement (mcb-validate)
make check          # fmt --check + lint + test + validate
make audit          # cargo-audit security scan
make coverage       # cargo-tarpaulin
```

All PRs must pass: `lint` + `test` + `validate` + `audit` + zero `unwrap()`/`expect()`.

## Import Patterns

- Re-exports at crate root: `pub use entities::*;`, `pub use error::{Error, Result};`
- Domain types imported as: `use mcb_domain::{CodeChunk, Error, Result};`
- Provider traits from domain: `use mcb_domain::ports::providers::EmbeddingProvider;`
- Never `use mcb_application::ports::providers` from providers crate (CI-enforced)

## Documentation

- **Module docs**: `//!` doc comments at top of each `lib.rs` and `mod.rs` with tables
- **ADRs**: 48 ADRs in `docs/adr/` with numbered naming (`001-modular-crates-architecture.md`)
- **Inline**: `///` on all public items (enforced by `missing_docs = "warn"`)
- **Update rule**: Update `docs/` when architecture or behavior changes

## Git & Workflow

- **Commits**: Conventional commits (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`)
- **Branches**: Feature branches → PR → main
- **Issue tracking**: Beads (`bd create`, `bd ready`, `bd close`, `bd sync`)
- **CI**: GitHub Actions, matrix (Linux/macOS/Windows × stable/beta)

## Safety Rules

- No `as any` / `@ts-ignore` / type suppression
- No `unwrap()` / `expect()` in production (CI-enforced)
- No `unsafe_code` (workspace deny)
- No silent failure paths (empty catch blocks)
- No destructive git operations unless explicitly requested

## Context File Conventions

- MVI principle: <200 lines per file
- Include `Last updated`, source paths, `Update Notes` section
- Searchable headings, minimal prose, tables preferred

## Sources

- `Cargo.toml` (lints), `rustfmt.toml`, `rust-toolchain.toml`
- `Makefile`, `make/quality.mk`, `make/dev.mk`
- `.github/workflows/ci.yml`, `crates/mcb-domain/src/error/types.rs`

## Update Notes

- 2026-02-15: Full harvest rewrite — comprehensive conventions from codebase analysis.
- 2026-02-12: Pointed to docs/developer/CONTRIBUTING.md as source of truth.
