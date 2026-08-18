<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Infrastructure Layer

**Source**: `crates/mcb-infrastructure/src/`
**Crate**: `mcb-infrastructure`

## ↔ Code ↔ Docs cross-reference

| Direction | Link |
| --------- | ---- |
| Code → Docs | [`crates/mcb-infrastructure/src/lib.rs`](../../crates/mcb-infrastructure/src/lib.rs) links here |
| Docs → Code | [`crates/mcb-infrastructure/src/lib.rs`](../../crates/mcb-infrastructure/src/lib.rs) — crate root |
| Architecture | [`ARCHITECTURE.md`](../architecture/ARCHITECTURE.md) · [`ADR-050`](../adr/050-manual-composition-root-dill-removal.md) · [`ADR-023`](../adr/023-inventory-to-linkme-migration.md) |
| Roadmap | [`ROADMAP.md`](../developer/ROADMAP.md) |

## Overview

The infrastructure module contains the technical plumbing of the system: DI bootstrap, configuration management, logging, caching, and shared technical services.

---

## Dependency Injection

Dependency injection system using a **manual composition root (`AppContext` + `init_app()`)** with linkme registry for provider discovery and handle-based runtime switching.

### Architecture

```text
linkme (compile-time)     AppContext (bootstrap)     Handle-based
─────────────────────     ──────────────────────      ────────────
EMBEDDING_PROVIDERS  →    Resolver → init_app() →    Handle (RwLock)
(list of factories)                                       ↓
                                                    AdminService
                                                   (switch via API)
```

- **Bootstrap** ([`bootstrap.rs`](../../crates/mcb-infrastructure/src/infrastructure/mod.rs)): Application initialization.
- **Handles** ([`handles.rs`](../../crates/mcb-infrastructure/src/infrastructure/mod.rs)): RwLock wrappers for runtime switching.
- **Composition Root** ([`bootstrap.rs`](../../crates/mcb-infrastructure/src/infrastructure/mod.rs)): AppContext manual composition root configuration.

---

## Configuration

Type-safe, layered configuration management with environment variable overrides.

### Configuration Structure

- **Types** ([`types.rs`](../../crates/mcb-infrastructure/src/config/app.rs)): Hierarchical structures (`AppConfig`, `ServerConfig`, `AuthConfig`).
- **Loader** ([`loader.rs`](../../crates/mcb-infrastructure/src/config/loader.rs)): Multi-source loading (Environment + `.toml`).

👉 **Canonical Env Var Matrix**: [`ENVIRONMENT_VARIABLES.md`](../configuration/ENVIRONMENT_VARIABLES.md)

---

## Shared Technical Areas

- [`cache/`](../../crates/mcb-infrastructure/src/crypto/) - Shared caching infrastructure.
- [`logging/`](../../crates/mcb-infrastructure/src/logging.rs) - Contextual logging (Tracing/OpenTelemetry).
- [`crypto/`](../../crates/mcb-infrastructure/src/crypto/) - AES-256 and SHA-256 utilities.
- [`health.rs`](../../crates/mcb-infrastructure/src/routing/health.rs) - System health check orchestration.

## File Structure

```text
crates/mcb-infrastructure/src/
├── cache/          # Shared caching
├── config/         # Configuration loading
├── constants/      # System-wide constants
├── crypto/         # Cryptography
├── di/             # Dependency Injection root
├── logging/        # Tracing/Logging
├── routing/        # Internal message routing
├── services/       # Infrastructure services
├── utils/          # Shared utilities
└── lib.rs          # Crate entry point
```

---

### Updated 2026-02-20 - Consolidated di.md and config.md for SSOT adherence
