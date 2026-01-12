# Code Refactoring Roadmap - Large File Splitting Plan

**Status**: Planning (v0.1.0 → v0.2.0)
**Priority**: LOW
**Scope**: 13 files exceeding 500-line limit

---

## Overview

The codebase currently has 13 files exceeding 500 lines (Rust style guide recommendation).
This document provides a detailed split plan for each file, prioritized by impact and complexity.

## Summary

| File | Lines | Priority | Proposed Split | Status |
|------|-------|----------|-----------------|--------|
| `src/admin/handlers.rs` | 1394 | HIGH | 6 modules (config, providers, indexes, monitoring, maintenance, utils) | Planning |
| `src/admin/service/implementation.rs` | 1255 | HIGH | 5 modules (config ops, provider ops, index ops, monitoring, maintenance) | Planning |
| `src/server/mcp_server.rs` | 826 | HIGH | 3 modules (core, lifecycle, initialization) | Planning |
| `src/adapters/providers/vector_store/filesystem.rs` | 771 | MEDIUM | 3 modules (storage, indexing, utils) | Planning |
| `src/adapters/providers/vector_store/milvus.rs` | 649 | MEDIUM | 2 modules (client, operations) | Planning |
| `src/adapters/providers/vector_store/edgevec.rs` | 617 | MEDIUM | 2 modules (storage, operations) | Planning |
| `src/infrastructure/recovery.rs` | 582 | MEDIUM | 2 modules (manager, policy) | Planning |
| `src/adapters/providers/routing/health.rs` | 582 | MEDIUM | 2 modules (monitor, checker) | Planning |
| `src/infrastructure/rate_limit.rs` | 571 | LOW | 2 modules (limiter, config) | Deferred |
| `src/adapters/providers/routing/router.rs` | 548 | LOW | 2 modules (router, strategy) | Deferred |
| `src/server/init.rs` | 528 | MEDIUM | 2 modules (setup, providers) | Planning |
| `src/infrastructure/limits/enforcer.rs` | 511 | LOW | 2 modules (enforcer, metrics) | Deferred |

---

## Detailed Split Plans

### 1. admin/handlers.rs (1394 lines) - HIGHEST PRIORITY

**Current Structure**: 47 handler functions with diverse responsibilities

**Proposed Split** into `src/admin/handlers/`:
```
handlers/
├── mod.rs              (re-exports all handlers)
├── config.rs           (250 lines)   - Config management handlers
├── providers.rs        (180 lines)   - Provider lifecycle handlers
├── indexes.rs          (200 lines)   - Index operation handlers
├── monitoring.rs       (320 lines)   - Metrics and health monitoring
├── maintenance.rs      (280 lines)   - Cache cleanup and maintenance
└── utils.rs            (100 lines)   - Shared helper functions
```

**Handler Distribution**:

**config.rs**:
- `get_config_handler` (46 lines)
- `update_config_handler` (112 lines)
- `get_configuration_handler` (13 lines)
- `update_configuration_handler` (18 lines)
- `validate_configuration_handler` (14 lines)
- `get_configuration_history_handler` (15 lines)

**providers.rs**:
- `list_providers_handler` (25 lines)
- `add_provider_handler` (54 lines)
- `remove_provider_handler` (23 lines)
- `restart_provider_handler` (14 lines)

**indexes.rs**:
- `list_indexes_handler` (30 lines)
- `index_operation_handler` (110 lines)
- `get_status_handler` (68 lines)
- `rebuild_index_handler` (14 lines)

**monitoring.rs**:
- `get_dashboard_metrics_handler` (80 lines)
- `get_logs_handler` (14 lines)
- `export_logs_handler` (15 lines)
- `get_log_stats_handler` (14 lines)
- `get_health_status_handler` (TBD)
- `get_system_metrics_handler` (TBD)

**maintenance.rs**:
- `clear_cache_handler` (22 lines)
- `cleanup_data_handler` (28 lines)
- `optimize_database_handler` (TBD)
- `backup_operation_handler` (TBD)

**Implementation Pattern**:
```rust
// src/admin/handlers/mod.rs
pub mod config;
pub mod providers;
pub mod indexes;
pub mod monitoring;
pub mod maintenance;
pub mod utils;

pub use config::*;
pub use providers::*;
pub use indexes::*;
pub use monitoring::*;
pub use maintenance::*;
```

**Benefits**:
- Each module ~250 lines (well under 500 limit)
- Clear separation by responsibility
- Easier to test individual handler groups
- Simpler merge conflict resolution

---

### 2. admin/service/implementation.rs (1255 lines) - HIGH PRIORITY

**Current Structure**: Service implementation with 47 methods

**Proposed Split** into `src/admin/service/implementation/`:
```
implementation/
├── mod.rs              (re-exports, struct definition)
├── config.rs           (250 lines)   - Configuration operations
├── providers.rs        (220 lines)   - Provider management
├── indexes.rs          (200 lines)   - Index operations
├── monitoring.rs       (280 lines)   - Monitoring and metrics
└── maintenance.rs      (200 lines)   - Maintenance operations
```

**Method Distribution** (by category):

**Config Operations** (config.rs):
- `get_configuration` - read config
- `update_configuration` - write config with validation
- `get_configuration_history` - retrieve historical configs
- `reset_to_defaults` - restore default config
- `export_configuration` - export as JSON/TOML

**Provider Management** (providers.rs):
- `list_providers` - enumerate registered providers
- `register_embedding_provider` - add new embedding provider
- `register_vector_store_provider` - add vector store
- `update_provider_config` - modify provider settings
- `remove_provider` - unregister provider
- `get_provider_health` - check provider status

**Index Operations** (indexes.rs):
- `list_indexes` - enumerate indexes
- `create_index` - create new index
- `delete_index` - remove index
- `rebuild_index` - rebuild index structure
- `get_indexing_status` - check index state
- `estimate_index_size` - calculate storage

**Monitoring & Metrics** (monitoring.rs):
- `get_system_metrics` - CPU, memory, disk
- `get_performance_metrics` - latency, throughput
- `get_logs` - retrieve application logs
- `export_logs` - export to file
- `get_log_stats` - summarize logs
- `get_health_status` - system health

**Maintenance** (maintenance.rs):
- `clear_cache` - reset cache
- `cleanup_expired_data` - remove old entries
- `optimize_database` - defragment/optimize
- `backup_data` - create backup
- `restore_backup` - restore from backup
- `analyze_database` - gather statistics

**Implementation Pattern**:
```rust
// src/admin/service/implementation/mod.rs
pub struct AdminService {
    config_service: Arc<dyn ConfigService>,
    provider_service: Arc<dyn ProviderService>,
    // ... other services
}

mod config;
mod providers;
mod indexes;
mod monitoring;
mod maintenance;
```

**Benefits**:
- Each module handles single concern
- Easier to unit test
- Simpler code navigation
- Reduced cognitive load

---

### 3. server/mcp_server.rs (826 lines) - HIGH PRIORITY

**Current Structure**: Main MCP server implementation with initialization, lifecycle, and request handling

**Proposed Split** into `src/server/`:
```
server/
├── mcp_server/
│   ├── mod.rs          (core struct and impl)
│   ├── lifecycle.rs    (initialization, shutdown, health)
│   ├── routing.rs      (tool routing and request dispatch)
│   └── state.rs        (shared server state)
├── mcp_server.rs       (will reference mcp_server/mod.rs)
├── handlers/           (separate directory)
├── transport/          (already modular)
└── init.rs             (startup logic)
```

**Responsibility Distribution**:

**mcp_server/mod.rs** (~250 lines):
- `McpServer` struct definition
- Constructor (`new`, `from_components`)
- Public accessors for components
- Core initialization logic

**mcp_server/lifecycle.rs** (~200 lines):
- Server lifecycle (init, start, stop)
- Health checks
- Graceful shutdown
- Component lifecycle coordination

**mcp_server/routing.rs** (~200 lines):
- Tool registration and lookup
- Request routing to handlers
- Error handling
- Request/response serialization

**mcp_server/state.rs** (~80 lines):
- Shared state structures
- Component holder
- State synchronization

**Implementation Pattern**:
```rust
// src/server/mcp_server/mod.rs
pub struct McpServer {
    // ... fields
}

pub use self::lifecycle::*;
pub use self::routing::*;

mod lifecycle;
mod routing;
mod state;
```

**Benefits**:
- Clear separation of concerns
- Each module < 250 lines
- Lifecycle management isolated
- Routing logic centralized

---

### 4. adapters/providers/vector_store/filesystem.rs (771 lines)

**Current Structure**: Full filesystem vector store implementation

**Proposed Split** into `filesystem/`:
```
filesystem/
├── mod.rs              (provider struct, public API)
├── storage.rs          (read/write operations)
├── indexing.rs         (index building and traversal)
├── cache.rs            (in-memory caching)
└── utils.rs            (helpers)
```

**Responsibility Distribution**:

**mod.rs** (~200 lines):
- `FilesystemVectorStore` struct
- Configuration
- Public interface implementation
- Provider trait implementation

**storage.rs** (~250 lines):
- File I/O operations
- Vector serialization/deserialization
- Directory management
- Disk operations

**indexing.rs** (~200 lines):
- Index structure management
- Building indexes from vectors
- Index traversal
- Range queries

**cache.rs** (~100 lines):
- In-memory caching
- Cache invalidation
- Memory management

---

### 5. infrastructure/recovery.rs (582 lines)

**Proposed Split** into `recovery/`:
```
recovery/
├── mod.rs              (manager struct, public API)
├── policy.rs           (recovery policies and strategies)
├── state.rs            (recovery state tracking)
└── utils.rs            (helpers)
```

**Responsibility Distribution**:

**mod.rs**:
- `RecoveryManager` struct
- Main recovery orchestration
- Component restart logic

**policy.rs**:
- Backoff strategies
- Retry policies
- Recovery decision logic

**state.rs**:
- Component health tracking
- Recovery history
- State transitions

---

## Implementation Timeline

### Phase 1 (v0.2.0 Sprint 1)
**Estimated effort**: 2-3 weeks

1. **admin/handlers.rs** split (600 lines of work)
2. **admin/service/implementation.rs** split (700 lines of work)

### Phase 2 (v0.2.0 Sprint 2)
**Estimated effort**: 1-2 weeks

1. **server/mcp_server.rs** split (500 lines of work)
2. **infrastructure/recovery.rs** split (200 lines of work)

### Phase 3 (v0.2.0 Sprint 3+)
**Estimated effort**: 2-3 weeks

1. Vector store implementations (filesystem, milvus, edgevec)
2. Remaining infrastructure files

---

## Implementation Checklist

For each split:

- [ ] Create new module directory
- [ ] Create `mod.rs` with re-exports
- [ ] Move code to submodules
- [ ] Update imports throughout codebase
- [ ] Update relative paths in doc comments
- [ ] Run `cargo check` to verify compilation
- [ ] Run `cargo test --lib` to verify all tests pass
- [ ] Run `cargo clippy --lib` for linting
- [ ] Create commit with clear message
- [ ] Update CHANGELOG with refactoring note
- [ ] Update architecture documentation if needed

## Best Practices During Refactoring

1. **One split per commit** - Makes history easier to review
2. **Preserve public API** - Re-export from mod.rs to avoid import changes
3. **Test after each split** - Ensure no regressions
4. **Keep logical grouping** - Don't split by line count alone
5. **Document dependencies** - Comment on inter-module dependencies
6. **Minimize visibility changes** - Keep public/private as before

## References

- [Rust API Guidelines - Module Organization](https://rust-lang.github.io/api-guidelines/organization.html)
- [Module Examples in std Library](https://github.com/rust-lang/rust/tree/master/library)
- Project Architecture: See `docs/architecture/ARCHITECTURE.md`

---

**Last Updated**: 2026-01-12
**Next Review**: v0.2.0 planning cycle
