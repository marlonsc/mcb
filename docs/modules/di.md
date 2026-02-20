<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# di Module

**Source**: `crates/mcb-infrastructure/src/di/`
**Crate**: `mcb-infrastructure`
**Files**: 8+
**Lines of Code**: ~800

## Overview

Dependency injection system using dill IoC Container with handle-based runtime switching and linkme registry for provider discovery.

## Architecture (ADR-024 → ADR-029)

```text
linkme (compile-time)     dill Catalog (runtime)     Handle-based
─────────────────────     ─────────────────────      ────────────
EMBEDDING_PROVIDERS  →    Resolver → add_value() →   Handle (RwLock)
(list of factories)                                       ↓
                                                    AdminService
                                                   (switch via API)
```

### Key Components

### Catalog (`catalog.rs`)

dill IoC Container configuration and service resolution.

```rust
pub async fn build_catalog(config: AppConfig) -> Result<Catalog> {
    CatalogBuilder::new()
        .add_value(config)
        .add_value(embedding_provider)    // From linkme registry
        .add_value(embedding_handle)      // RwLock wrapper
        .add_value(embedding_admin)       // Runtime switching
        .build()
}

// Service retrieval via AppContext (bootstrap.rs):
//   app_context.embedding_handle()    → Arc<EmbeddingProviderHandle>
//   app_context.vector_store_handle() → Arc<VectorStoreProviderHandle>
//   app_context.cache_handle()        → Arc<CacheProviderHandle>
```

### Bootstrap (`bootstrap.rs`)

Application initialization and AppContext creation.

### Handles (`handles.rs`)

RwLock wrappers for runtime provider switching:

- `EmbeddingProviderHandle`
- `VectorStoreProviderHandle`
- `CacheProviderHandle`
- `LanguageProviderHandle`

### Provider Resolvers (`provider_resolvers.rs`)

Components that access the linkme registry to resolve providers by name.

### Admin Services (`admin.rs`)

Runtime provider switching via API:

- `EmbeddingAdminService` (implements `EmbeddingAdminInterface`)
- `VectorStoreAdminService` (implements `VectorStoreAdminInterface`)
- `CacheAdminService` (implements `CacheAdminInterface`)
- `LanguageAdminService` (implements `LanguageAdminInterface`)

## File Structure

```text
crates/mcb-infrastructure/src/di/
├── admin.rs              # Admin services (runtime switching)
├── bootstrap.rs          # Application initialization
├── catalog.rs            # dill Catalog configuration
├── dispatch.rs           # Dispatch utilities
├── handles.rs            # RwLock provider handles
├── mod.rs                # Module exports
├── modules/              # Domain services factory
├── provider_resolvers.rs # linkme registry access
└── resolver.rs           # Provider resolution utilities
```

## DI Pattern

```rust
// 1. Ports defined in mcb-domain (single source of truth)
use mcb_domain::ports::providers::EmbeddingProvider;

// 2. Providers resolve via linkme registry
let resolver = EmbeddingProviderResolver::new(config);
let provider = resolver.resolve_from_config()?;

// 3. Handles wrap providers for runtime switching
let handle = EmbeddingProviderHandle::new(provider);

// 4. Admin services expose switching API
let admin = EmbeddingAdminService::new(resolver, handle);
admin.switch_provider(new_config)?;

// 5. dill Catalog stores all services
let catalog = build_catalog(config).await?;
let embedding: Arc<dyn EmbeddingProvider> = get_service(&catalog)?;
```

## Key Exports

```rust
pub use bootstrap::*;
pub use catalog::{
    build_catalog, get_cache_provider, get_embedding_provider,
    get_language_provider, get_service, get_vector_store_provider,
};
pub use handles::{
    CacheProviderHandle, EmbeddingProviderHandle,
    LanguageProviderHandle, VectorStoreProviderHandle,
};
pub use admin::{
    CacheAdminService, EmbeddingAdminService,
    LanguageAdminService, VectorStoreAdminService,
};
```

## Architecture Rules (mcb-validate)

| Rule ID | Description |
| --------- | ------------- |
| CA007 | Infrastructure cannot import concrete types from Application |
| CA008 | Application must import ports from mcb-domain |

## Cross-References

- **ADR-024**: [Simplified Dependency Injection](../adr/024-simplified-dependency-injection.md) (superseded)
- **ADR-029**: [Hexagonal Architecture with dill](../adr/029-hexagonal-architecture-dill.md) (current)
- **Domain Ports**: [mcb-domain/src/ports/providers/](../../crates/mcb-domain/src/ports/providers/)
- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

### Updated 2026-01-20 - Reflects dill IoC + handle-based DI (v0.2.1)
