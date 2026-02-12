# Technical Patterns Reference

**Last updated:** 2026-02-12
**Source:** Codebase analysis across 9 crates (v0.2.1)

This document captures the recurring implementation patterns used throughout MCB. For the full architecture overview, see [ARCHITECTURE.md](./ARCHITECTURE.md). For boundary rules, see [ARCHITECTURE_BOUNDARIES.md](./ARCHITECTURE_BOUNDARIES.md).

---

## Architecture: Clean Architecture + Hexagonal

**Dependency direction:** `mcb-server → mcb-infrastructure → mcb-application → mcb-domain`
Providers (`mcb-providers`) implement domain ports — never depend upstream.

```
mcb-domain         → Entities, ports (traits), errors, value objects, macros, registry
mcb-application    → Use cases, decorators, services (orchestration)
mcb-infrastructure → DI (dill+Handle), config, crypto, logging, health, routing
mcb-providers      → Embedding, vector store, cache, database, git, language, events
mcb-server         → MCP handlers, admin UI (Handlebars), transport (stdio/HTTP), hooks
mcb-validate       → Architecture rules, AST analysis, linters, metrics
```

## Three-Layer DI: linkme → dill → Handle

MCB uses a three-layer dependency injection pattern that combines compile-time discovery with runtime flexibility:

1. **linkme** — Compile-time discovery via `#[distributed_slice]`. Each provider registers itself at compile time, so the runtime can discover all available implementations without manual wiring.

2. **dill** — Runtime IoC `Catalog` wiring (`mcb-infrastructure/src/di/catalog.rs`). The catalog assembles the full dependency graph at startup, resolving traits to concrete implementations based on configuration.

3. **Handle<T>** — `RwLock<Arc<dyn T>>` for runtime provider switching (`di/handle.rs`). Handles allow hot-swapping provider implementations without restarting the server — useful for failover and configuration changes.

**Why three layers?** linkme discovers *what's available*, dill wires *what's active*, and Handle enables *runtime switching*. This separation means adding a new provider requires only a `#[distributed_slice]` annotation — zero changes to wiring code.

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

**Adding a new provider:**

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
|-------|---------|---------|
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

Configuration follows a hierarchical override pattern using figment (see [ADR-025](../adr/025-figment-configuration.md)):

- **Hierarchical**: `AppConfig → {ProvidersConfig, ServerConfig, AuthConfig}`
- **Loader**: Default TOML → Override TOML → `MCP__*` env vars (highest priority)
- **Hot-reload**: `ConfigWatcher` monitors changes (`config/watcher.rs`)
- **Validation**: Config is validated at startup before providers are initialized

```
MCP__PROVIDERS__EMBEDDING__PROVIDER=openai
MCP__PROVIDERS__VECTOR_STORE__PROVIDER=edgevec
MCP__SERVER__NETWORK__PORT=8080
MCP__INFRASTRUCTURE__CACHE__PROVIDER=moka
```

## Testing Patterns

MCB uses a layered testing approach:

- **Mocks**: `Arc<Mutex<Vec<T>>>` state tracking in `test_utils/mock_services/`. Mocks record all calls for assertion. Used when you need to verify interaction patterns.
- **Real providers**: `extern crate mcb_providers` forces linkme registration in integration tests. This ensures the full provider discovery chain works end-to-end.
- **Fixtures**: Shared data in `test_utils` modules per crate. Fixtures provide consistent test data across unit and integration tests.
- **Test layout**: Integration tests in `tests/` directory (not inline `#[cfg(test)]`). Test files follow `tests/unit/*_tests.rs`, `tests/integration/*_tests.rs` pattern.
- **Tools**: `rstest` (parameterized), `mockall` (auto-mocks), `insta` (snapshots), `tempfile` (temp dirs)

## Key Patterns Summary

| Pattern | Key Location | When to Use |
|---------|-------------|-------------|
| Clean Architecture | `*/lib.rs` | All new features |
| linkme registration | `mcb-domain/src/registry/` | New provider types |
| Three-layer DI | `mcb-infrastructure/src/di/` | Service wiring |
| Error factories | `mcb-domain/src/error/mod.rs` | All error handling |
| Handle<T> | `di/handle.rs` | Runtime-switchable providers |
| define_id! | `value_objects/ids.rs` | New domain IDs |
| EntityCrudAdapter | `mcb-server/src/admin/crud_adapter.rs` | Admin CRUD entities |
| Decorator | `mcb-application/src/decorators/` | Cross-cutting concerns |
| impl_registry! | `mcb-domain/src/macros.rs` | New provider registries |

## Related Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) — Full system architecture
- [ARCHITECTURE_BOUNDARIES.md](./ARCHITECTURE_BOUNDARIES.md) — Boundary enforcement rules
- [CLEAN_ARCHITECTURE.md](./CLEAN_ARCHITECTURE.md) — Clean architecture principles
- [../modules/domain.md](../modules/domain.md) — Domain entity model
- [../modules/providers.md](../modules/providers.md) — Provider implementations
- [../developer/CONTRIBUTING.md](../developer/CONTRIBUTING.md) — Coding conventions

---

*Updated 2026-02-12 — Reflects v0.2.1 crate architecture*
