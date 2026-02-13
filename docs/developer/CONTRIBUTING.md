<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Contributing to Memory Context Browser

Thank you for your interest in contributing! This guide covers everything you need for MCB development.

**Last updated:**2026-02-12 |**Version:** v0.2.1

## 🚀 Getting Started

### Prerequisites

- **Rust 1.89+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: Version control system

### Setup Development Environment

```bash
git clone https://github.com/marlonsc/mcb.git
cd mcb
make build
make test       # 10,000+ test functions
make check      # Full quality pipeline
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
    └── test_utils/      ← Shared test helpers
```

### Code Structure (v0.2.1 Clean Architecture)

```text
crates/
├── mcb/                # Unified facade crate (public API)
├── mcb-domain/         # Core types, ports, entities (innermost)
├── mcb-application/    # Business services (use cases, domain services)
├── mcb-providers/      # External integrations (embedding, vector store, language)
├── mcb-infrastructure/ # Shared systems (DI, config, null adapters)
├── mcb-server/         # MCP protocol, HTTP transport, admin
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
- Run `make fmt` before committing

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
make lint MCB_CI=1 && make validate QUICK=1  # Pre-commit validation
git commit                              # Commit (pre-commit hook runs checks)
git push                                # Push
```

## 🧪 Testing

### Running Tests

```bash
make test                               # All 10,000+ test functions
make test SCOPE=unit                    # Unit tests only
cargo test test_name -- --nocapture     # Specific test with output
```

### Test Patterns

- **Integration tests** in `tests/` directory (not inline `#[cfg(test)]`)
- **Test files**: `tests/unit/*_tests.rs`, `tests/integration/*_tests.rs`
- **Test helpers**: `rstest` (params), `mockall` (mocks), `insta` (snapshots), `tempfile`
- **Real providers**: `extern crate mcb_providers` forces linkme registration
- **Mocks**: `Arc<Mutex<Vec<T>>>` state tracking in `test_utils/mock_services/`

## 🔨 Make-First Workflow

| Command | Purpose |
| --------- | --------- |
| `make build` | Build all crates |
| `make fmt` | Auto-format code |
| `make lint` | Format check + clippy |
| `make test` | All unit + integration tests |
| `make validate` | Architecture rule enforcement |
| `make check` | Full CI pipeline |
| `make audit` | Security advisory scan |

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
- [ ] Code formats correctly: `make fmt`
- [ ] No Rust lint errors: `make lint`
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
cargo clean && make build && make check
```

### Docs-only validation (no Rust build)

```bash
make docs-lint
make docs-validate QUICK=1
```

## 🚀 Code References

- **Config**: `mcb_infrastructure::config::ConfigLoader` — See [CONFIGURATION.md](../CONFIGURATION.md), [ADR-025](../adr/025-figment-configuration.md)
- **DI**: `mcb_infrastructure::di::bootstrap::init_app(config)` — See [ADR-029](../adr/029-hexagonal-architecture-dill.md)
- **Patterns**: See [PATTERNS.md](../architecture/PATTERNS.md) for implementation patterns
- **Run server**: `cargo run --bin mcb` or `make build` then run the binary

---

## Cross-References

- [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) — System overview
- [PATTERNS.md](../architecture/PATTERNS.md) — Implementation patterns
- [ROADMAP.md](./ROADMAP.md) — Project state and roadmap
- [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md) — Current state
- [DEPLOYMENT.md](../operations/DEPLOYMENT.md) — Deployment guide
- [CI_RELEASE.md](../operations/CI_RELEASE.md) — CI/CD and release process
