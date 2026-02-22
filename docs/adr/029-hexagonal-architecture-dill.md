<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 29
title: Hexagonal Architecture with dill IoC
status: SUPERSEDED
created:
updated: 2026-02-22
related: []
supersedes: []
superseded_by: [50]
implementation_status: Superseded
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 029: Hexagonal Architecture with dill IoC

## Status

**Superseded** by [ADR 050: Manual Composition Root — dill Removal](050-manual-composition-root-dill-removal.md) (v0.2.1)

> Originally: Evolution of [ADR 024: Simplified Dependency Injection]
> (024-simplified-dependency-injection.md), adding dill Catalog as IoC
> container. dill was removed in v0.2.1 because `build_catalog()` was
> never called in production and dill was the sole nightly Rust dependency.

## Context

The previous architecture (ADR-024) used a handle-based DI pattern with linkme
registry for compile-time provider discovery. While effective, this approach had
coupling issues:

1. **Infrastructure imported concrete types from Application**
    - `domain_services.rs` imported `ContextServiceImpl`, `SearchServiceImpl`

2. **Application ports were duplicated**
    - `mcb-domain/src/ports/providers/` (correct location)
    - `mcb-application/src/ports/providers/` (duplication)

3. **No IoC container for service lifecycle management**
    - Manual wiring in bootstrap.rs
    - No dependency graph validation

## Decision

We implement a proper hexagonal architecture with dill IoC container:

### 1. Ports in mcb-domain (Single Source of Truth)

All provider ports are defined in `mcb-domain/src/ports/providers/`:

```rust
// mcb-domain/src/ports/providers/embedding.rs
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Embedding>;
}

// mcb-domain/src/ports/providers/vector_store.rs
pub trait VectorStoreProvider: Send + Sync {
    fn store(&self, embedding: &Embedding) -> Result<()>;
    fn search(&self, query: &Embedding) -> Result<Vec<SearchResult>>;
}
```

Application layer does not own provider ports. Import provider traits directly from
`mcb-domain/src/ports/providers/` to avoid duplicate declarations and compatibility shims.

### 2. dill Catalog as IoC Container

The dill `Catalog` manages service registration and resolution:

```rust
// mcb-infrastructure/src/di/catalog.rs
pub async fn build_catalog(config: AppConfig) -> Result<Catalog> {
    CatalogBuilder::new()
        // Configuration
        .add_value(config)
        // Providers (from linkme registry)
        .add_value(embedding_provider)
        .add_value(vector_store_provider)
        // Handles (for runtime switching)
        .add_value(embedding_handle)
        .add_value(vector_store_handle)
        // Admin services
        .add_value(embedding_admin)
        .add_value(vector_store_admin)
        .build()
}

// Service retrieval via AppContext (bootstrap.rs)
// AppContext holds all resolved providers as typed fields:
//   app_context.embedding_handle()    → Arc<EmbeddingProviderHandle>
//   app_context.vector_store_handle() → Arc<VectorStoreProviderHandle>
//   app_context.cache_handle()        → Arc<CacheProviderHandle>
```

### 3. Architecture Layers

```text
┌──────────────────────────────────────────────────────────────┐
│                        mcb-domain                             │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ PORTS (trait definitions)                              │  │
│  │   - EmbeddingProvider                                  │  │
│  │   - VectorStoreProvider                                │  │
│  │   - CacheProvider                                      │  │
│  │   - LanguageChunkingProvider                           │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              ↑
┌──────────────────────────────────────────────────────────────┐
│                      mcb-application                          │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ USE CASES (import ports from mcb-domain)               │  │
│  │   - ContextServiceImpl                                 │  │
│  │   - SearchServiceImpl                                  │  │
│  │   - IndexingServiceImpl                                │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ REGISTRY (linkme distributed slices)                   │  │
│  │   - EMBEDDING_PROVIDERS                                │  │
│  │   - VECTOR_STORE_PROVIDERS                             │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              ↑
┌──────────────────────────────────────────────────────────────┐
│                    mcb-infrastructure                         │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ COMPOSITION ROOT (dill Catalog)                        │  │
│  │   - build_catalog() creates IoC container              │  │
│  │   - Provider Handles for runtime switching             │  │
│  │   - Admin Services for API-based management            │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              ↑
┌──────────────────────────────────────────────────────────────┐
│                      mcb-providers                            │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ ADAPTERS (implement ports from mcb-domain)             │  │
│  │   - OllamaEmbeddingProvider                            │  │
│  │   - MilvusVectorStore                                  │  │
│  │   - MokaCacheProvider                                  │  │
│  │   - Register via linkme distributed slices             │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### 4. Validation Rules

New mcb-validate rules enforce the architecture:

| Rule ID | Description |
| --------- | ------------- |
| CA007 | Infrastructure cannot import concrete types from Application |
| CA008 | Application must import ports from mcb-domain |

## Consequences

### Positive

1. **Clear layer separation**: Ports in domain, implementations in providers
2. **IoC container benefits**: dill Catalog manages service lifecycle
3. **Gradual migration**: `add_value()` allows mixing with existing pattern
4. **Compile-time validation**: mcb-validate enforces architecture
5. **Runtime switching**: Provider handles still support admin API

### Negative

1. **Additional dependency**: dill crate added to workspace
2. **Learning curve**: Developers must understand dill API
3. **Migration effort**: Existing code updated to new import paths

### Neutral

1. **Bootstrap still exists**: `init_app()` wraps `build_catalog()`
2. **AppContext unchanged**: Same public interface for consumers

## Canonical References

> **Note**: This ADR is a historical decision record. For current architecture
> details, consult the normative documents below. The code paths in this ADR
> reflect the state at the time of writing; the current single source of truth
> for port trait locations is `mcb-domain/src/ports/providers/` (not
> `mcb-application/src/ports/providers/`, which was removed as duplicated).

- [ARCHITECTURE_BOUNDARIES.md](../architecture/ARCHITECTURE_BOUNDARIES.md) — Layer rules and module ownership (normative)
- [PATTERNS.md](../architecture/PATTERNS.md) — Technical patterns reference (normative)
- [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) — Full system architecture (normative)

## References

- [dill-rs Documentation](https://docs.rs/dill/latest/dill/)
- [ADR 023: Inventory to linkme Migration](023-inventory-to-linkme-migration.md)
- [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md)
- [Clean Architecture](<https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html>)
