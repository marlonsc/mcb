<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 53
title: Shared Provider Resolution via ServiceResolutionContext
status: ACCEPTED
created: 2026-03-01
updated: 2026-03-01
related: [50, 23, 24]
extends: [50]
supersedes: []
superseded_by: []
implementation_status: In Progress
---

# ADR 053: Shared Provider Resolution via ServiceResolutionContext

## Status

**Accepted** (v0.3.0)

> Extends [ADR 050: Manual Composition Root — dill Removal](050-manual-composition-root-dill-removal.md).

## Context

ADR-050 established a manual composition root (`composition.rs`) where providers
are resolved once and injected into services. However, the linkme service registry
builders (`context_service.rs`, `memory_service/registry.rs`) each independently
resolve `EmbeddingProvider` and `VectorStoreProvider` from `AppConfig`, creating
three problems:

1. **Duplicate resolution**: The same config-to-provider propagation logic is
   duplicated across 3 files (~100 lines of identical code in `composition.rs`,
   `context_service.rs`, and `memory_service/registry.rs`).

2. **DI bypass in composition root**: Because registry builders resolve their own
   providers, `composition.rs` cannot rely on the registry for services that need
   shared providers. It resorts to direct `::new()` calls (4 violations:
   `ContextServiceImpl`, `SearchServiceImpl`, `IndexingServiceImpl`,
   `MemoryServiceImpl`), bypassing the linkme DI infrastructure.

3. **No provider sharing**: Each registry builder creates its own provider
   instance. While `ContextServiceImpl` is stateless (only holds Arc wrappers),
   the pattern prevents shared state (health checks, connection pools,
   operation tracking) from being consistent across services.

## Decision

Extend `ServiceResolutionContext` with pre-resolved shared provider fields:

```rust
pub struct ServiceResolutionContext {
    pub db: DatabaseConnection,
    pub config: Arc<AppConfig>,
    pub event_bus: Arc<dyn EventBusProvider>,
    // NEW: shared providers resolved once at startup
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    pub vector_store_provider: Arc<dyn VectorStoreProvider>,
}
```

### Design Choices

1. **Eager resolution at startup** — Providers are resolved before
   `ServiceResolutionContext` is constructed. Fail-fast: if a provider cannot
   be resolved, the application fails to start rather than failing on first use.

2. **Centralized helpers** — Two public functions in `resolution_context.rs`
   (`resolve_embedding_from_config`, `resolve_vector_store_from_config`)
   centralize the config-to-provider propagation, eliminating the DRY violation.

3. **Non-optional fields** — Provider fields are required (`Arc<dyn T>`), not
   `Option<Arc<dyn T>>`. The null provider pattern handles "not configured"
   cases via the existing `"null"` fallback in config resolution.

4. **Extend existing struct** — No new types, traits, or abstraction layers.
   YAGNI-compliant: only two shared providers exist today.

### Resolution Flow

```text
Loco Initializer (mcp_server.rs)
  ├→ Load AppConfig
  ├→ resolve_embedding_from_config(config)      → Arc<dyn EmbeddingProvider>
  ├→ resolve_vector_store_from_config(config)    → Arc<dyn VectorStoreProvider>
  └→ ServiceResolutionContext { db, config, event_bus, embedding_provider, vector_store_provider }
       └→ Registry builders clone from context (zero re-resolution)
```

## Consequences

### Positive

1. **DRY compliance** — ~100 lines of duplicated config propagation replaced by
   2 centralized helpers
2. **Pure registry composition** — `composition.rs` uses only `resolve_*()` calls,
   zero direct `::new()` construction of infrastructure services
3. **Shared state** — All services share the same provider instances, ensuring
   consistent health checks and operation tracking
4. **Fail-fast** — Provider resolution errors surface at startup, not at first
   request

### Negative

1. **Larger struct** — `ServiceResolutionContext` grows from 3 to 5 fields
2. **Startup coupling** — Provider resolution must succeed before any service
   can be built (intentional — fail-fast design)

### Neutral

1. **linkme unchanged** — Provider registration pattern is identical
2. **ServiceBuilder enum unchanged** — No signature changes to registry builders
3. **Multiple ContextServiceImpl instances acceptable** — The struct is stateless
   (only holds Arc wrappers), so pure-registry resolution creating separate
   instances per `resolve_context_service()` call is safe

## Alternatives Considered

### Alternative 1: Loco SharedStore at infrastructure layer

- **What**: Use Loco's `SharedStore` (`DashMap<TypeId, Box<dyn Any + Send + Sync>>`)
  in `mcb-infrastructure`
- **Pros**: Already implemented in Loco, typed container with runtime insertion
- **Cons**: `SharedStore` lives at the web framework layer (`loco_rs::app`).
  Using it in `mcb-infrastructure` would violate the Clean Architecture boundary
  (infrastructure depends on framework). Also adds runtime type erasure overhead.
- **Rejection**: Architecture boundary violation. The pattern is good but the
  layer is wrong.

### Alternative 2: New `ExecutionContainer` type

- **What**: Create a dedicated `ExecutionContainer` struct alongside
  `ServiceResolutionContext`
- **Pros**: Separation of concerns, extensible
- **Cons**: Adds a new type with its own lifecycle. `ServiceResolutionContext`
  already serves this exact purpose. Two overlapping contexts creates confusion
  about which to use where.
- **Rejection**: YAGNI — `ServiceResolutionContext` IS the execution container.

### Alternative 3: OnceLock / lazy resolution

- **What**: Use `OnceLock<Arc<dyn EmbeddingProvider>>` for lazy initialization
- **Pros**: Defers resolution cost, doesn't require all providers at startup
- **Cons**: Hides initialization failures (first-use errors instead of startup
  errors). Adds `OnceLock` complexity for no real benefit — providers are always
  needed immediately by service builders.
- **Rejection**: Eager resolution is simpler and fails faster.

### Alternative 4: Generic typemap container

- **What**: Implement a generic `TypeMap<dyn Any>` container for arbitrary
  provider types
- **Pros**: Extensible to any number of provider types
- **Cons**: Runtime type erasure, no compile-time safety for required providers,
  over-engineered for 2 providers.
- **Rejection**: YAGNI — explicit fields are simpler and type-safe.

## Implementation Notes

### Changes

1. Added `resolve_embedding_from_config()` and `resolve_vector_store_from_config()`
   to `crates/mcb-infrastructure/src/resolution_context.rs`
2. Extended `ServiceResolutionContext` with `embedding_provider` and
   `vector_store_provider` fields
3. Updated all 3 construction sites (Loco initializer, 2 test fixtures)
4. Simplified `context_service.rs` and `memory_service/registry.rs` builders
   to clone from context instead of resolving independently
5. Converted `composition.rs` to pure registry resolution (zero `::new()` calls)

### Migration Impact

- **Zero API changes** — `ServiceResolutionContext` is internal to the
  infrastructure layer
- **3 construction sites updated** — Loco initializer + 2 test fixtures
- **~100 lines removed** — duplicated config propagation across 3 files
- **4 DI violations eliminated** — `composition.rs` becomes pure registry

## Canonical References

- [ADR 050: Manual Composition Root](050-manual-composition-root-dill-removal.md) — baseline this extends
- [ADR 023: Inventory to linkme Migration](023-inventory-to-linkme-migration.md) — registry pattern

## References

- [linkme Documentation](https://docs.rs/linkme)
- [Loco.rs SharedStore](https://docs.rs/loco-rs/latest/loco_rs/app/struct.SharedStore.html) — pattern reference (not used at this layer)
