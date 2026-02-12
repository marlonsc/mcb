# Moka Cache Library - Internal Usage Mapping

**Library**: `moka` (v0.12.x)  
**Status**: IMPLEMENTED (v0.1.1)  
**ADR Reference**: [ADR-005: Context Cache Support (Moka and Redis)](../../docs/adr/005-context-cache-support.md)  
**Purpose**: High-performance, concurrent in-memory cache for embedding and search result caching

## Architecture Overview

Moka is integrated as the **default cache provider** in MCB's unified provider architecture. It implements the `CacheProvider` trait from the domain layer and is registered via `linkme` distributed slices for runtime provider discovery.

### Design Pattern
- **Pattern**: Factory + Trait-based Provider + Distributed Slice Registration
- **Scope**: Local in-memory caching (single-instance deployments)
- **Fallback**: Redis for distributed scenarios (ADR-005)

---

## Core Implementation

### 1. Provider Implementation
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/cache/moka.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 25 | `use moka::future::Cache;` | Core async cache import |
| 29-40 | `MokaCacheProvider` struct | Wrapper around `moka::future::Cache<String, Vec<u8>>` |
| 42-46 | `Default` impl | Creates cache with default capacity (CACHE_DEFAULT_SIZE_LIMIT) |
| 48-75 | Factory methods | `new()`, `with_capacity()`, `with_config()` for instantiation |
| 77-149 | `CacheProvider` trait impl | Async methods: `get_json()`, `set_json()`, `delete()`, `exists()`, `clear()`, `stats()`, `size()` |
| 80 | `cache.get(key).await` | Async retrieval with UTF-8 validation |
| 106 | `cache.insert(key, bytes).await` | Async insertion with size validation |
| 112 | `cache.invalidate(key).await` | Async deletion |
| 121-122 | `invalidate_all()` + `run_pending_tasks()` | Atomic clear operation |
| 128-129 | `entry_count()` + `run_pending_tasks()` | Stats collection |
| 146-148 | `provider_name()` | Returns "moka" identifier |

### 2. Distributed Slice Registration (linkme)
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/cache/moka.rs:160-186`

| Line | Component | Purpose |
|------|-----------|---------|
| 166 | `use mcb_domain::registry::cache::{CACHE_PROVIDERS, ...}` | Import registry and types |
| 169-178 | `moka_cache_factory()` | Factory function creating `Arc<dyn CacheProvider>` |
| 172-176 | Config-aware instantiation | Respects `max_size` from `CacheProviderConfig` |
| 180-185 | `#[linkme::distributed_slice(CACHE_PROVIDERS)]` | Auto-registration in global provider registry |
| 181-185 | `MOKA_PROVIDER` static | Entry with name, description, factory function |

### 3. Module Re-exports
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/cache/mod.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 18 | `pub mod moka;` | Module declaration |
| 24 | `pub use moka::MokaCacheProvider;` | Public re-export for convenience |

---

## Dependency Injection & Registration

### 4. DI Bootstrap
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/di/bootstrap.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 57-60 | `cache_handle: Arc<CacheProviderHandle>` | Runtime-swappable cache provider handle |
| 67 | `cache_resolver: Arc<CacheProviderResolver>` | Access to linkme registry |
| 74 | `cache_admin: Arc<dyn CacheAdminInterface>` | Admin service for provider switching |

### 5. Provider Resolver
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/di/provider_resolvers.rs:207`

| Line | Component | Purpose |
|------|-----------|---------|
| 207 | `CacheProvider::Moka => "moka"` | Config-to-provider-name mapping |

### 6. Configuration Types
**File**: `/home/marlonsc/mcb/crates/mcb-domain/src/value_objects/config.rs:59`

| Line | Component | Purpose |
|------|-----------|---------|
| 59 | `/// Provider name (moka, redis, null)` | Documentation of supported providers |

**File**: `/home/marlonsc/mcb/crates/mcb-domain/src/registry/cache.rs:15`

| Line | Component | Purpose |
|------|-----------|---------|
| 15 | `/// Provider name (e.g., "moka", "redis", "null")` | Registry entry documentation |

---

## Runtime Usage & Observability

### 7. Cache Operations in Application Layer
**File**: `/home/marlonsc/mcb/crates/mcb-application/src/use_cases/memory_service.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 241 | `tracing::warn!(...)` | Observability hook for cache degradation |
| 250 | `tracing::debug!(vector_search_failed, ...)` | Fallback to FTS-only when cache fails |

### 8. Metrics Recording
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/tests/unit/prometheus_metrics_tests.rs:54-55`

| Line | Component | Purpose |
|------|-----------|---------|
| 54 | `metrics.record_cache_hit("moka");` | Hit metric recording |
| 55 | `metrics.record_cache_miss("moka");` | Miss metric recording |

### 9. Admin Interface
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/di/admin.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| (See CacheAdminService) | `get_cache_stats()` | Retrieve cache statistics via HTTP |
| (See CacheAdminService) | `clear_cache()` | Clear cache via admin API |

---

## Testing & Validation

### 10. Unit Tests - DI Resolution
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/tests/unit/di_tests.rs:25,35`

| Line | Component | Purpose |
|------|-----------|---------|
| 25 | `cache: vec![("moka", "Moka cache")]` | DI test fixture |
| 35 | `assert!(display.contains("moka"));` | Validation that moka is registered |

### 11. Integration Tests - Provider Resolution
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/tests/di/resolver_tests.rs:14`

| Line | Component | Purpose |
|------|-----------|---------|
| 14 | `cache: vec![("moka", "Moka cache")]` | Resolver test fixture |

### 12. Architecture Validation
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/tests/di/architecture_validation_tests.rs:81,189-191`

| Line | Component | Purpose |
|------|-----------|---------|
| 81 | `let expected = ["moka"];` | Expected provider list |
| 189 | `CacheProviderConfig::new("moka")` | Config creation for testing |
| 191 | `assert_eq!(cache.provider_name(), "moka", ...)` | Provider identity validation |

### 13. Application Layer Tests
**File**: `/home/marlonsc/mcb/crates/mcb-application/tests/unit/registry_tests.rs:278-307`

| Line | Component | Purpose |
|------|-----------|---------|
| 278-282 | `test_resolve_moka_cache_provider()` | Moka provider resolution test |
| 288-307 | Full provider resolution test | Validates moka as default cache |

### 14. Dispatch Tests
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/tests/di/dispatch_tests.rs:90`

| Line | Component | Purpose |
|------|-----------|---------|
| 90 | `assert_eq!(app_context.cache_handle().get().provider_name(), "moka");` | Default cache validation |

---

## Configuration & Constants

### 15. Default Size Limit
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/constants.rs`

| Component | Purpose |
|-----------|---------|
| `CACHE_DEFAULT_SIZE_LIMIT` | Default capacity for moka cache (typically 10,000 entries) |

---

## Cargo.toml Dependencies

### 16. Dependency Declaration
**File**: `/home/marlonsc/mcb/crates/mcb-providers/Cargo.toml:110`

```toml
moka = { workspace = true }
```

**Workspace Definition**: `/home/marlonsc/mcb/Cargo.toml`
- Version: Latest stable (0.12.x)
- Features: `future` (async support)

---

## ADR Alignment

### ADR-005: Context Cache Support (Moka and Redis)
- **Status**: IMPLEMENTED
- **Rationale**: 
  - Moka chosen for local, high-performance in-memory caching
  - Thread-safe concurrent access without external dependencies
  - Configurable TTL and capacity for flexible deployment
  - Redis fallback for distributed scenarios (not yet implemented)
- **Trade-offs**:
  - Single-instance only (no cross-process sharing)
  - Memory-bound (no persistence)
  - Requires restart to clear (unless admin API used)

### ADR-003: Unified Provider Architecture
- **Alignment**: Moka implements `CacheProvider` trait
- **Registration**: Via `linkme` distributed slices (ADR-023)
- **Lifecycle**: Managed by `AppContext` in bootstrap

### ADR-024: Simplified Dependency Injection
- **Pattern**: Factory function + trait object
- **Scope**: Runtime-swappable via `CacheProviderHandle`

---

## Summary Table

| Aspect | Details |
|--------|---------|
| **Core Impl** | `/home/marlonsc/mcb/crates/mcb-providers/src/cache/moka.rs:25-149` |
| **DI Registration** | `/home/marlonsc/mcb/crates/mcb-providers/src/cache/moka.rs:180-185` |
| **Bootstrap** | `/home/marlonsc/mcb/crates/mcb-infrastructure/src/di/bootstrap.rs:57-60` |
| **Observability** | Metrics via `prometheus_metrics.rs`, tracing via `memory_service.rs` |
| **Tests** | 5+ test files validating resolution, metrics, and provider identity |
| **ADR** | ADR-005 (primary), ADR-003, ADR-024 |
| **Status** | IMPLEMENTED, production-ready |

