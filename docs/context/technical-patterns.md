# Technical Patterns Context

**Last updated:** 2026-02-11
**Source:** Codebase analysis across 9 crates

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

1. **linkme** — Compile-time discovery via `#[distributed_slice]`
2. **dill** — Runtime IoC `Catalog` wiring (`mcb-infrastructure/src/di/catalog.rs`)
3. **Handle<T>** — `RwLock<Arc<dyn T>>` for runtime provider switching (`di/handle.rs`)

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

## Trait Patterns

- All async traits: `#[async_trait]` + `Send + Sync` for `Arc<dyn Trait>` usage
- Composition: `VectorStoreProvider` extends `VectorStoreAdmin + VectorStoreBrowser`
- Port hierarchy: `ports/{providers, services, repositories, infrastructure, admin, browse}`

## Error Handling

- **Single enum**: `mcb-domain/src/error/mod.rs` with `#[derive(thiserror::Error)]`
- **Factory methods**: `Error::io("msg")`, `Error::embedding("msg")` — never construct variants directly
- **Context trait**: `ErrorContext<T>` adds `.context("msg")` (`mcb-infrastructure/src/error_ext.rs`)
- **Rule**: No `unwrap()`/`expect()` outside tests. Always `?` propagation.

## Macros (`mcb-domain/src/macros.rs`)

| Macro | Purpose |
|-------|---------|
| `impl_from_str!` | Case-insensitive enum parsing from config strings |
| `impl_registry!` | Full registry boilerplate (entry, slice, resolve, list) |
| `table!`, `col!`, `index!` | Schema DDL with less boilerplate |
| `define_id!` | Strong-typed ID newtype with From/Into/Display/AsRef |

## Module Organization

- **lib.rs as hub**: Declares modules, re-exports key types
- **File-per-trait**: Each major trait in own file (`embedding.rs`, `vector_store.rs`)
- **Functional grouping**: `ports/providers/`, `ports/services/`, `ports/repositories/`

## Configuration

- **Hierarchical**: `AppConfig → {ProvidersConfig, ServerConfig, AuthConfig}`
- **Loader**: TOML/env vars with validation (`config/loader.rs`)
- **Hot-reload**: `ConfigWatcher` monitors changes (`config/watcher.rs`)

## Testing

- **Mocks**: `Arc<Mutex<Vec<T>>>` state tracking in `test_utils/mock_services/`
- **Real providers**: `extern crate mcb_providers` forces linkme in integration tests
- **Fixtures**: Shared data in `test_utils` modules per crate

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

## Related Context

- `docs/context/domain-concepts.md` — entity model
- `docs/context/conventions.md` — coding style
- `docs/architecture/ARCHITECTURE.md` — system architecture

## Mirror Context

- `context/project-intelligence/technical-patterns.md` — compact operational mirror

## Change Notes

- 2026-02-11T23:26:00-03:00 - Reconciled with `context/project-intelligence/technical-patterns.md` and added mirror reference.
