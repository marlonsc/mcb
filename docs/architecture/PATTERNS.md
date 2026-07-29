<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Technical Patterns Reference

**Last updated:** 2026-02-12
**Source:** Codebase analysis across 6 crates (v0.2.1)

This document captures the recurring implementation patterns used throughout MCB. For the full architecture overview, see [ARCHITECTURE.md](./ARCHITECTURE.md). For boundary rules, see [ARCHITECTURE_BOUNDARIES.md](./ARCHITECTURE_BOUNDARIES.md).

---

## Architecture: Clean Architecture + Hexagonal

**Dependency direction:** `mcb-server → mcb-infrastructure → mcb-providers → mcb-domain`
Providers (`mcb-providers`) implement domain ports — never depend upstream.

```text
mcb-domain         → Entities, ports (traits), errors, value objects, macros, registry
mcb-infrastructure → DI (linkme+Handle), config, crypto, logging, health, routing
mcb-providers      → Embedding, vector store, cache, database, git, language, events
mcb-server         → MCP handlers, admin UI (Handlebars), transport (stdio/HTTP), hooks
mcb-validate       → Architecture rules, AST analysis, linters, metrics
```

## Two-Layer DI: linkme → Handle (ADR-050)

MCB uses a two-layer dependency injection pattern that combines compile-time discovery with runtime flexibility:

1. **linkme** — Compile-time discovery via `#[distributed_slice]`. Each provider registers itself at compile time, so the runtime can discover all available implementations without manual wiring.

2. **Handle\<T\>** — `RwLock<Arc<dyn T>>` for runtime provider switching (`di/handle.rs`). Handles allow hot-swapping provider implementations without restarting the server — useful for failover and configuration changes.

Services are wired in `init_app()` (`bootstrap.rs`), which queries linkme registries via resolvers, resolves providers from config, and assembles the `AppContext` with explicit field assignment.

**Why two layers?** linkme discovers *what's available*, and Handle enables *runtime switching*. Adding a new provider requires only a `#[distributed_slice]` annotation — zero changes to wiring code.

## Provider Registration

```rust
// Registry macro → entry struct + distributed_slice + resolve/list
impl_registry!(embedding, EmbeddingProvider, EmbeddingConfigContainer);

// Self-registration per provider
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OPENAI: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "openai", description: "...", factory: openai_embedding_factory,
};
```

Registries: `embedding`, `vector_store`, `cache`, `database`, `language` in `mcb-domain/src/registry/`

### Adding a new provider

1. Implement the trait (e.g., `EmbeddingProvider`)
2. Create a factory function that takes config and returns `Arc<dyn Trait>`
3. Add a `#[distributed_slice]` entry with name, description, and factory
4. The registry automatically picks it up — no manual registration needed

## Trait Patterns

- All async traits: `#[async_trait]` + `Send + Sync` for `Arc<dyn Trait>` usage
- Composition: `VectorStoreProvider` extends `VectorStoreAdmin + VectorStoreBrowser`
- Port hierarchy: `ports/{providers, services, repositories, infrastructure, admin, browse}`

**Why `Send + Sync`?** All providers are shared across async tasks via `Arc<dyn Trait>`. The `Send + Sync` bounds ensure thread safety. The `#[async_trait]` macro handles the lifetime complexity of async trait methods.

## Error Handling

MCB uses a centralized error strategy (see [ADR-019](../adr/019-error-handling-strategy.md)):

- **Single enum**: `mcb-domain/src/error/mod.rs` with `#[derive(thiserror::Error)]`
- **Factory methods**: `Error::io("msg")`, `Error::embedding("msg")` — never construct variants directly. This ensures consistent error messages and makes it easy to add context.
- **Context trait**: `ErrorContext<T>` adds `.context("msg")` (`mcb-infrastructure/src/error_ext.rs`). This enriches errors as they propagate up the call stack without losing the original cause.
- **Rule**: No `unwrap()`/`expect()` outside tests. Always `?` propagation. This is enforced by workspace lints (`deny(dead_code)`, `deny(unused_variables)`).

```rust
// CORRECT: Use factory methods
Err(Error::embedding("OpenAI API returned 429"))

// CORRECT: Add context during propagation
let result = provider.embed(text).await.context("embedding user query")?;

// WRONG: Never construct variants directly
Err(Error::ProviderError { message: "...".into() })

// WRONG: Never panic in production code
let result = provider.embed(text).await.unwrap();
```

## Macros (`mcb-domain/src/macros.rs`)

| Macro | Purpose | Example |
| ------- | --------- | --------- |
| `impl_from_str!` | Case-insensitive enum parsing from config strings | `impl_from_str!(ProjectType, Cargo, Npm, Python)` |
| `impl_registry!` | Full registry boilerplate (entry, slice, resolve, list) | `impl_registry!(embedding, EmbeddingProvider, EmbeddingConfigContainer)` |
| `table!`, `col!`, `index!` | Schema DDL with less boilerplate | `table!("code_chunks")`, `col!("file_path")` |
| `define_id!` | Strong-typed ID newtype with From/Into/Display/AsRef | `define_id!(SessionId)`, `define_id!(OrgId)` |

**Why macros?** These patterns repeat across 5+ provider types and 10+ entities. The macros eliminate 50-100 lines of boilerplate per usage while ensuring consistency. Changes to the pattern propagate automatically.

## Module Organization

- **lib.rs as hub**: Declares modules, re-exports key types. Every crate follows this pattern — `lib.rs` is the public API surface.
- **File-per-trait**: Each major trait in own file (`embedding.rs`, `vector_store.rs`). This keeps files focused and makes it easy to find implementations.
- **Functional grouping**: `ports/{providers, services, repositories, infrastructure, admin, browse}`. Ports are organized by their role, not by the concrete implementation.

## Configuration

Configuration is loaded from Loco environment-based YAML files (see [ADR-051](../adr/051-seaql-loco-platform-rebuild.md); supersedes [ADR-025](../adr/archive/superseded-025-figment-configuration.md)):

- **Hierarchical**: `AppConfig → {ProvidersConfig, ServerConfig, AuthConfig}`
- **Loader**: Environment-based YAML (`config/{env}.yaml`, e.g. `config/development.yaml`)
- **Hot-reload**: `ConfigWatcher` monitors changes (`config/watcher.rs`)
- **Validation**: Config is validated at startup before providers are initialized

```yaml
# config/development.yaml (under settings:)
providers:
  embedding:
    provider: openai
  vector_store:
    provider: edgevec
server:
  network:
    port: 8080
```

## Testing Patterns

MCB uses a layered testing approach:

- **Mocks**: `Arc<Mutex<Vec<T>>>` state tracking in `utils/mock_services/`. Mocks record all calls for assertion. Used when you need to verify interaction patterns.
- **Real providers**: `extern crate mcb_providers` forces linkme registration in integration tests. This ensures the full provider discovery chain works end-to-end.
- **Fixtures**: Shared data in `utils` modules per crate. Fixtures provide consistent test data across unit and integration tests.
- **Test layout**: Integration tests in `tests/` directory (not inline `#[cfg(test)]`). Test files follow `tests/unit/*_tests.rs`, `tests/integration/*_tests.rs` pattern.
- **Tools**: `rstest` (parameterized), `mockall` (auto-mocks), `insta` (snapshots), `tempfile` (temp dirs)

## v0.2.1 No-Feature Standardization Contract

This cycle is architecture optimization only.

### Allowed Changes

- Deduplicate declarations and remove redundant conversions.
- Normalize API naming (`id` is identifier; do not overload `name` as ID).
- Tighten validation and fail early.

### Disallowed Changes

- New features (endpoints/commands/providers/config surface).
- Compatibility shims and dual-path behavior.

### Banned Patterns (fast-fail)

- Raw `String`/`Uuid` as domain IDs where strong ID types exist.
- Port traits declared outside `mcb-domain/src/ports/**`.
- DTO-to-domain `From`/`Into` mapping inside domain modules.
- API responses forwarding internal `.to_string()` errors directly.
- Legacy alias modules kept only for backward compatibility.

## Key Patterns Summary

| Pattern | Key Location | When to Use |
| --------- | ------------- | ------------- |
| Clean Architecture | `*/lib.rs` | All new features |
| linkme registration | `mcb-domain/src/registry/` | New provider types |
| Two-layer DI (linkme+Handle) | `mcb-infrastructure/src/di/` | Service wiring |
| Error factories | `mcb-domain/src/error/mod.rs` | All error handling |
| Handle\<T\> | `di/handle.rs` | Runtime-switchable providers |
| define_id! | `value_objects/ids.rs` | New domain IDs |
| EntityCrudAdapter | `mcb-server/src/admin/crud_adapter.rs` | Admin CRUD entities |
| Decorator | `mcb-infrastructure/src/di/modules/use_cases/` | Cross-cutting concerns |
| impl_registry! | `mcb-domain/src/macros.rs` | New provider registries |

## Related Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) — Full system architecture
- [ARCHITECTURE_BOUNDARIES.md](./ARCHITECTURE_BOUNDARIES.md) — Boundary enforcement rules
- [CLEAN_ARCHITECTURE.md](./CLEAN_ARCHITECTURE.md) — Clean architecture principles
- [../modules/domain.md](../modules/domain.md) — Domain entity model
- [../modules/providers.md](../modules/providers.md) — Provider implementations
- [../developer/CONTRIBUTING.md](../developer/CONTRIBUTING.md) — Coding conventions

---

### Updated 2026-02-12 — Reflects v0.2.1 crate architecture
