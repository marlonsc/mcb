<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Contributing to Memory Context Browser

Thank you for your interest in contributing! This guide covers everything you need for MCB development.

**Last updated:** 2026-06-28 | **Version:** v0.4.0

## 🚀 Getting Started

### Prerequisites

- **Rust 1.92+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: Version control system

### Setup Development Environment

```bash
git clone https://github.com/marlonsc/mcb.git
cd mcb
make boot       # install hooks + tools
make build
make test       # full workspace tests
make check      # full quality pipeline
```

## 🔄 Development Workflow

1. **Choose Task**: Check `bd ready` for available Beads issues
2. **Create Branch**: Use descriptive names (`feat/name`, `fix/name`)
3. **Make Changes**: Follow conventions below
4. **Test Changes**: `make test`
5. **Submit PR**: Create pull request with clear description

## 📝 Naming Conventions

| Element | Convention | Example |
| --------- | ----------- | --------- |
| Crates | kebab-case, `mcb-` prefix | `mcb-domain`, `mcb-server` |
| Library names | snake_case | `mcb_domain`, `mcb_server` |
| Functions | snake_case | `embed_batch()`, `search_similar()` |
| Types/Traits | PascalCase | `CodeChunk`, `EmbeddingProvider` |
| Enum variants | PascalCase | `AgentType::Sisyphus` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_BATCH_SIZE` |
| Modules | snake_case | `entities/agent/`, `config/types/` |
| Test files | `*_tests.rs` | `config_tests.rs`, `cache_tests.rs` |
| Constructors | `new()` or `with_*()` | `Config::new().with_ttl(300)` |

## 📁 File Organization

```text
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
    └── utils/      ← Shared test helpers
```

### Code Structure (v0.4.0 Clean Architecture)

```text
crates/
├── mcb/                # CLI facade binary
├── mcb-utils/          # Shared leaf utilities (innermost, no mcb-* deps)
├── mcb-domain/         # Core types, ports, entities, errors
├── mcb-providers/      # External integrations (embedding, vector store, DB, git)
├── mcb-infrastructure/ # Shared systems (DI, config, cross-cutting services)
├── mcb-server/         # MCP protocol, HTTP transport, admin UI
└── mcb-validate/       # Architecture validation
```

### Import Order (enforced by rustfmt)

1. Standard library: `use std::...`
2. External crates: `use serde::{...}; use tokio::{...}`
3. Workspace crates: `use mcb_domain::{...}`
4. Local modules: `use crate::...`

## 🔧 Formatting & Lints

### Formatting (rustfmt.toml)

- **Edition**: 2024 | **Max width**: 100 | **Tab size**: 4
- Run `make check WHAT=fix ACT=fmt` before committing

### Workspace Lints (Cargo.toml)

```toml
unsafe_code = "deny"
missing_docs = "warn"
non_ascii_idents = "deny"
dead_code = "deny"
unused_variables = "deny"
unused_imports = "deny"
```

### Visibility Rules

- `pub mod` for public modules, `pub use` for re-exports
- `pub(crate)` for internal items — private by default
- Domain exports: entities, value objects, ports, errors
- Re-export at lib.rs: `pub use entities::*;`

## ⚠️ Error Handling

- Single `Error` enum with `#[derive(thiserror::Error)]`
- Factory methods: `Error::io("msg")`, `Error::embedding("msg")` — never construct variants directly
- `Result<T>` type alias everywhere
- No `unwrap()`/`expect()` outside tests — use `?` propagation
- `ErrorContext<T>` trait for `.context("msg")` enrichment

See [ADR-019](../adr/019-error-handling-strategy.md) for the full error handling strategy.

## 📝 Commit Messages

Use**conventional commits**:

```text
<type>(<scope>): <short description>

<body: 1-2 sentences explaining why>

Fixes #<issue-id>
```

**Types:** feat, fix, docs, style, refactor, perf, test, build, ci, chore

**Scope:** module or crate (e.g., core, cli, docs, scripts, mcb-server)

**Beads auto-close:** include `Fixes #<id>` or `Closes #<id>` in the footer.

### Commit Workflow

```bash
./scripts/commit_analyze.sh             # Analyze staged changes
make check WHAT=lint && make check WHAT=validate QUICK=1   # Pre-commit validation
git commit                              # Commit (pre-commit hook runs checks)
git push                                # Push
```

## 🧪 Testing

### Running Tests

```bash
make test                               # Full workspace test suite
make test SCOPE=unit                    # Unit tests only
make test SCOPE=integration             # Integration tests
make test SCOPE=doc                     # Doctests
cargo test -p mcb-server --test unit -- test_name --nocapture  # Specific test
```

`cargo-nextest` is used automatically when installed; otherwise falls back to `cargo test`.

### Test Patterns

- **Integration tests** in `tests/` directory (not inline `#[cfg(test)]`)
- **Test files**: `tests/unit/*_tests.rs`, `tests/integration/*_tests.rs`
- **Test helpers**: `rstest` (params), `mockall` (mocks), `insta` (snapshots), `tempfile`
- **Real providers**: `extern crate mcb_providers` forces linkme registration
- **Mocks**: `Arc<Mutex<Vec<T>>>` state tracking in `utils/mock_services/`

## 🔨 Make-First Workflow

Never call `cargo`/`git` directly. Use `make <verb> [WHAT=phase] [ACT=sub] [APPLY=Y]`.

| Command | Purpose |
| --------- | --------- |
| `make build` | Build all crates (debug) |
| `make build RELEASE=1` | Release build |
| `make check WHAT=fix ACT=fmt` | Auto-format code |
| `make check WHAT=lint` | Format check + clippy (`-D warnings`) |
| `make test` | All unit + integration tests |
| `make test SCOPE=unit` | Unit tests only |
| `make check WHAT=validate QUICK=1` | Architecture rule enforcement (quick) |
| `make check WHAT=guard` | Banned-pattern scanner (prod unwrap/panic/TODO/allow) |
| `make check WHAT=ci` | Full CI pipeline |
| `make check WHAT=audit` | Security advisory scan |
| `make build WHAT=docs ACT=lint` | Lint markdown |
| `make build WHAT=docs ACT=validate QUICK=1` | Validate docs and links |

> Destructive verbs (`commit`, `push`, `clean`, `codegen`, `release`) are DRY-RUN unless `APPLY=Y`.

## 📦 Dependency Management

- **Centralized**: All deps in `[workspace.dependencies]`
- **Features**: Explicit feature lists per crate
- **Security**: `deny.toml` for license/advisory checks
- **Profile**: LTO + single codegen unit for release

## ✅ Enforcement

| Convention | Tool | Level |
| ----------- | ------ | ------- |
| Formatting | rustfmt + CI | Required |
| Lints | Cargo workspace lints | Deny/Warn |
| Import order | rustfmt | Required |
| No unwrap | Lint + review | Deny |
| Doc comments | `missing_docs` | Warn |
| Commit style | Convention | Convention |
| Security | deny.toml + cargo-audit | CI |
| Architecture | mcb-validate | CI |

## 📋 Pull Request Guidelines

### Before Submitting

- [ ] Tests pass: `make test`
- [ ] Code formats correctly: `make check WHAT=fix ACT=fmt`
- [ ] No Rust lint errors: `make check WHAT=lint`
- [ ] Quality checks pass: `make check`
- [ ] Documentation updated if needed

### PR Description

Include: what changed, why, how to test, any breaking changes.

## 🐛 Reporting Issues

**Bug Reports**: steps to reproduce, expected vs actual behavior, environment details, error messages.

**Feature Requests**: problem description, proposed solution, use cases, alternatives considered.

## 🔧 Troubleshooting

### `make check` or `make build` fails with linker errors

```bash
make clean WHAT=build APPLY=Y && make build && make check
```

### Docs-only validation (no Rust build)

```bash
make build WHAT=docs ACT=lint
make build WHAT=docs ACT=validate QUICK=1
```

## 🚀 Code References

- **Config**: `mcb_infrastructure::config::ConfigLoader` — See [CONFIGURATION.md](../CONFIGURATION.md), [ADR-051](../adr/051-seaql-loco-platform-rebuild.md) (supersedes [ADR-025](../adr/051-seaql-loco-platform-rebuild.md))
- **DI**: `mcb_infrastructure::di::bootstrap::init_app(config)` — See [ADR-050](../adr/050-manual-composition-root-dill-removal.md) (ADR-029 superseded)
- **Patterns**: See [PATTERNS.md](../architecture/PATTERNS.md) for implementation patterns
- **Run server**: `make build` then run `./target/debug/mcb` or `./target/release/mcb`

---

## Cross-References

- [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) — System overview
- [PATTERNS.md](../architecture/PATTERNS.md) — Implementation patterns
- [FLEXT_TO_MCB_MAPPING.md](./FLEXT_TO_MCB_MAPPING.md) — FLEXT pattern translation for MCB
- [SKILL_INDEX.md](./SKILL_INDEX.md) — Project ECC skills (under `.agents/skills/`) index
- [ROADMAP.md](./ROADMAP.md) — Project state and roadmap
- [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md) — Current state
- [DEPLOYMENT.md](../operations/DEPLOYMENT.md) — Deployment guide
