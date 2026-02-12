# Conventions Context

**Last updated:** 2026-02-11
**Source:** Codebase analysis — Cargo.toml, rustfmt.toml, .cargo/config.toml, Makefile, CI workflows

## Naming

| Element | Convention | Example |
|---------|-----------|---------|
| Crates | kebab-case, `mcb-` prefix | `mcb-domain`, `mcb-server` |
| Library names | snake_case | `mcb_domain`, `mcb_server` |
| Functions | snake_case | `embed_batch()`, `search_similar()` |
| Types/Traits | PascalCase | `CodeChunk`, `EmbeddingProvider` |
| Enum variants | PascalCase | `AgentType::Sisyphus` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_BATCH_SIZE` |
| Modules | snake_case | `entities/agent/`, `config/types/` |
| Test files | `*_tests.rs` | `config_tests.rs`, `cache_tests.rs` |
| Constructors | `new()` or `with_*()` | `Config::new().with_ttl(300)` |

## File Organization

```
crates/mcb-{name}/
├── src/
│   ├── lib.rs          ← Module declarations + pub use re-exports
│   ├── {domain}/
│   │   ├── mod.rs      ← Sub-module declarations + re-exports
│   │   ├── simple.rs   ← Single entity/trait per file
│   │   └── complex/    ← Multi-file module with mod.rs
│   └── constants/      ← Domain-specific constants
└── tests/
    ├── lib.rs           ← Test module root
    ├── unit.rs          ← Unit test module
    ├── integration.rs   ← Integration test module
    ├── unit/*_tests.rs  ← Individual test files
    └── test_utils/      ← Shared test helpers
```

## Import Order (enforced by rustfmt)

1. Standard library: `use std::...`
2. External crates: `use serde::{...}; use tokio::{...}`
3. Workspace crates: `use mcb_domain::{...}`
4. Local modules: `use crate::...`

## Formatting (rustfmt.toml)

- **Edition**: 2024 | **Max width**: 100 | **Tab size**: 4
- Run `make fmt` before committing

## Workspace Lints (Cargo.toml)

```toml
unsafe_code = "deny"
missing_docs = "warn"
non_ascii_idents = "deny"
dead_code = "deny"
unused_variables = "deny"
unused_imports = "deny"
```

## Error Handling

- Single `Error` enum with `#[derive(thiserror::Error)]`
- Factory methods: `Error::io("msg")`, `Error::embedding("msg")` — never construct variants directly
- `Result<T>` type alias everywhere
- No `unwrap()`/`expect()` outside tests — use `?` propagation
- `ErrorContext<T>` trait for `.context("msg")` enrichment

## Documentation

- `//!` module docs with markdown, tables, business rules
- `///` on all public items (enforced: `missing_docs = "warn"`)
- Code examples in doc comments where useful

## Testing

- Integration tests in `tests/` directory (not inline `#[cfg(test)]`)
- Test files: `tests/unit/*_tests.rs`, `tests/integration/*_tests.rs`
- Test helpers: `rstest` (params), `mockall` (mocks), `insta` (snapshots), `tempfile`
- Real providers: `extern crate mcb_providers` forces linkme registration

## Git

- **Commits**: `type(scope): description` (conventional commits)
- **Types**: `feat`, `fix`, `refactor`, `chore`, `docs`
- **Branches**: `feat/name`, `release/v0.x.y`, `main`
- Run `make fmt` before committing

## Make-First Workflow

| Command | Purpose |
|---------|---------|
| `make lint` | Format check + clippy |
| `make test` | All unit + integration tests |
| `make validate` | Architecture rule enforcement |
| `make audit` | Security advisory scan |
| `make fmt` | Auto-format code |
| `make ci` | Full CI pipeline |

## Dependency Management

- **Centralized**: All deps in `[workspace.dependencies]`
- **Features**: Explicit feature lists per crate
- **Security**: `deny.toml` for license/advisory checks
- **Profile**: LTO + single codegen unit for release

## Visibility

- `pub mod` for public modules, `pub use` for re-exports
- `pub(crate)` for internal items — private by default
- Domain exports: entities, value objects, ports, errors
- Re-export at lib.rs: `pub use entities::*;`

## Enforcement

| Convention | Tool | Level |
|-----------|------|-------|
| Formatting | rustfmt + CI | Required |
| Lints | Cargo workspace lints | Deny/Warn |
| Import order | rustfmt | Required |
| No unwrap | Lint + review | Deny |
| Doc comments | `missing_docs` | Warn |
| Commit style | Convention | Convention |
| Security | deny.toml + cargo-audit | CI |

## Related Context

- `docs/context/technical-patterns.md` — architecture patterns
- `docs/adr/019-error-handling-strategy.md` — error handling deep dive
- `docs/developer/CONTRIBUTING.md` — contributor guide

## Mirror Context

- `context/project-intelligence/conventions.md` — compact operational mirror

## Change Notes

- 2026-02-11T23:26:00-03:00 - Reconciled with `context/project-intelligence/conventions.md` and added mirror reference.
