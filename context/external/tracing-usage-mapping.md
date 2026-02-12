# Tracing Library - Internal Usage Mapping

**Library**: `tracing` + `tracing-subscriber` (v0.1.x)  
**Status**: IMPLEMENTED (v0.1.0+)  
**ADR Reference**: Implicit in [ADR-019: Error Handling Strategy](../../docs/adr/019-error-handling-strategy.md)  
**Purpose**: Structured logging, distributed tracing instrumentation, and observability

## Architecture Overview

Tracing is integrated as the **primary observability framework** across all MCB crates. It provides structured logging with JSON output, span instrumentation via macros, and integration with Prometheus metrics.

### Design Pattern
- **Pattern**: Macro-based instrumentation + Subscriber layers
- **Scope**: Application-wide logging, handler instrumentation, error tracking
- **Integration**: Prometheus metrics, file rotation, JSON formatting

---

## Core Logging Infrastructure

### 1. Logging Module
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/logging.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 1-4 | Module documentation | Structured logging with tracing |
| 9-10 | Imports | `tracing::{Level, debug, error, info, warn}` + `tracing_subscriber` |
| 16-29 | `init_logging()` | Main initialization function |
| 17-18 | `parse_log_level()` | Parse level string to tracing Level |
| 19 | `create_log_filter()` | Create EnvFilter from config |
| 20 | `create_file_appender()` | Create rolling file appender |
| 21-25 | JSON vs text format selection | Conditional initialization |
| 27 | `info!("Logging initialized...")` | Initialization confirmation |
| 34-36 | `create_log_filter()` | EnvFilter with MCP_LOG env var priority |
| 39-49 | `create_file_appender()` | Daily rolling file appender |
| 43 | `tracing_appender::rolling::daily()` | Rolling file creation |
| 52-76 | `init_json_logging()` | JSON format initialization |
| 56-62 | JSON layer configuration | Target, thread IDs, file, line numbers |
| 64 | `Registry::default().with(filter)` | Subscriber composition |
| 65-71 | File appender integration | Optional file output |
| 78-100 | `init_text_logging_terminal()` | Terminal-friendly colored output |
| 83-89 | Console layer configuration | ANSI colors, targets, thread info |

### 2. Logging Configuration
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/config/types/system.rs`

| Component | Purpose |
|-----------|---------|
| `LoggingConfig` struct | Configuration for logging level, format, file output |

### 3. Initialization in Server
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/init.rs:73`

| Line | Component | Purpose |
|------|-----------|---------|
| 73 | `mcb_infrastructure::logging::init_logging(config.logging.clone())?;` | Logging initialization |

---

## Handler Instrumentation

### 4. Macro-Based Instrumentation
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/handlers/`

| File | Line | Component | Purpose |
|------|------|-----------|---------|
| `search.rs` | 39 | `#[tracing::instrument(skip_all)]` | Search handler instrumentation |
| `index.rs` | 57 | `#[tracing::instrument(skip_all)]` | Index handler instrumentation |
| `validate.rs` | 30 | `#[tracing::instrument(skip_all)]` | Validation handler instrumentation |
| `project.rs` | 30 | `#[tracing::instrument(skip_all)]` | Project handler instrumentation |
| `agent.rs` | 32 | `#[tracing::instrument(skip_all)]` | Agent handler instrumentation |
| `plan_entity.rs` | 26 | `#[tracing::instrument(skip_all)]` | Plan entity handler instrumentation |
| `issue_entity.rs` | 27 | `#[tracing::instrument(skip_all)]` | Issue entity handler instrumentation |

### 5. VCS Handler Instrumentation
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/handlers/vcs/`

| File | Line | Component | Purpose |
|------|------|-----------|---------|
| `handler.rs` | 29 | `#[tracing::instrument(skip_all)]` | VCS handler instrumentation |
| `search_branch.rs` | 14 | `#[tracing::instrument(skip_all)]` | Branch search instrumentation |
| `list_repos.rs` | 13 | `#[tracing::instrument(skip_all)]` | Repository listing instrumentation |
| `index_repo.rs` | 15 | `#[tracing::instrument(skip_all)]` | Repository indexing instrumentation |
| `compare_branches.rs` | 14 | `#[tracing::instrument(skip_all)]` | Branch comparison instrumentation |
| `analyze_impact.rs` | 14 | `#[tracing::instrument(skip_all)]` | Impact analysis instrumentation |

### 6. Session Handler Instrumentation
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/handlers/session/`

| File | Line | Component | Purpose |
|------|------|-----------|---------|
| `handler.rs` | 42 | `#[tracing::instrument(skip_all)]` | Session handler instrumentation |
| `create.rs` | 17 | `#[tracing::instrument(skip_all)]` | Session creation instrumentation |
| `get.rs` | 13 | `#[tracing::instrument(skip_all)]` | Session retrieval instrumentation |
| `list.rs` | 13 | `#[tracing::instrument(skip_all)]` | Session listing instrumentation |
| `update.rs` | 14 | `#[tracing::instrument(skip_all)]` | Session update instrumentation |
| `summarize.rs` | 13 | `#[tracing::instrument(skip_all)]` | Session summarization instrumentation |

### 7. Memory Handler Instrumentation
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/handlers/memory/`

| File | Line | Component | Purpose |
|------|------|-----------|---------|
| `handler.rs` | 42 | `#[tracing::instrument(skip_all)]` | Memory handler instrumentation |
| `session.rs` | 14, 57 | `#[tracing::instrument(skip_all)]` | Memory session instrumentation (2 methods) |
| `observation.rs` | 17, 73 | `#[tracing::instrument(skip_all)]` | Observation instrumentation (2 methods) |
| `inject.rs` | 14 | `#[tracing::instrument(skip_all)]` | Injection instrumentation |
| `execution.rs` | 71, 158 | `#[tracing::instrument(skip_all)]` | Execution instrumentation (2 methods) |
| `quality_gate.rs` | 18, 98 | `#[tracing::instrument(skip_all)]` | Quality gate instrumentation (2 methods) |
| `list_timeline.rs` | 14, 69 | `#[tracing::instrument(skip_all)]` | Timeline listing instrumentation (2 methods) |

---

## Error Logging

### 8. Error Mapping with Tracing
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/error_mapping.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 10 | `tracing::error!(error = %other, "operation failed");` | Generic error logging |
| 39 | `tracing::error!(error = %error, "database operation failed");` | Database error logging |
| 43 | `tracing::error!(error = %error, "vector database operation failed");` | Vector DB error logging |
| 47 | `tracing::error!(error = %error, "embedding operation failed");` | Embedding error logging |
| 51 | `tracing::error!(error = %error, "network operation failed");` | Network error logging |
| 55 | `tracing::error!(error = %error, "observation storage failed");` | Observation storage error |
| 59 | `tracing::error!(error = %error, "VCS operation failed");` | VCS error logging |
| 74 | `tracing::error!(error = %error, "cache operation failed");` | Cache error logging |
| 78 | `tracing::error!(error = %error, "infrastructure error");` | Infrastructure error logging |
| 82 | `tracing::error!(error = %error, "internal error");` | Internal error logging |
| 88 | `tracing::error!(error = %error, "JSON processing failed");` | JSON error logging |
| 92 | `tracing::error!(error = %error, "encoding error");` | Encoding error logging |
| 102 | `tracing::error!(error = %e, "I/O operation failed");` | I/O error logging |
| 116 | `tracing::error!(error = %e, "browse operation failed");` | Browse operation error |
| 120 | `tracing::error!(error = %e, "highlight operation failed");` | Highlight operation error |

### 9. Structured Logging in Handlers
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/handlers/index.rs:83`

| Line | Component | Purpose |
|------|-----------|---------|
| 83 | `tracing::warn!(error = %e, path = ?path, "indexing failed");` | Structured warning with context |

### 10. Admin Handler Logging
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/handlers.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 83 | `tracing::info!("health_check called");` | Health check logging |
| 97 | `tracing::info!("get_metrics called");` | Metrics retrieval logging |
| 118 | `tracing::info!("get_jobs_status called");` | Job status logging |
| 170 | `tracing::info!("list_browse_projects called");` | Project listing logging |
| 181 | `tracing::warn!("Using default org context...");` | Multi-tenant warning |
| 189 | `tracing::error!(error = %e, "failed to list projects");` | Error logging |
| 243 | `tracing::info!("list_browse_repositories called");` | Repository listing logging |
| 270 | `tracing::error!(error = %e, "failed to list repositories");` | Error logging |
| 288 | `tracing::info!("list_browse_plans called");` | Plan listing logging |
| 308 | `tracing::error!(error = %e, "failed to list plans");` | Error logging |
| 326 | `tracing::info!("list_browse_issues called");` | Issue listing logging |
| 345 | `tracing::error!(error = %e, "failed to list issues");` | Error logging |
| 380 | `tracing::error!(error = %e, "failed to list organizations");` | Error logging |
| 412 | `tracing::info!("readiness_check called");` | Readiness check logging |
| 438 | `tracing::info!("liveness_check called");` | Liveness check logging |
| 512 | `tracing::info!("shutdown called");` | Shutdown logging |
| 579 | `tracing::info!("extended_health_check called");` | Extended health check logging |
| 721 | `tracing::info!("get_cache_stats called");` | Cache stats logging |
| 734 | `tracing::error!(error = %e, "failed to get cache stats");` | Cache stats error logging |

---

## Application Layer Observability

### 11. VCS Indexing Service
**File**: `/home/marlonsc/mcb/crates/mcb-application/src/use_cases/vcs_indexing.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 274 | `tracing::warn!(...)` | Warning for VCS indexing issues |
| 386 | `tracing::warn!(path = %relative, error = %e, "Failed to hash file");` | File hashing error |
| 397 | `tracing::debug!(path = %relative, "File changed, re-indexing");` | Change detection logging |
| 410 | `tracing::debug!(path = %old_file, "File deleted, creating tombstone");` | Deletion logging |

### 12. Memory Service
**File**: `/home/marlonsc/mcb/crates/mcb-application/src/use_cases/memory_service.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 241 | `tracing::warn!(...)` | Warning for memory service issues |
| 250 | `tracing::debug!(vector_search_failed, "Hybrid search degraded to FTS-only");` | Fallback logging |

### 13. Indexing Service
**File**: `/home/marlonsc/mcb/crates/mcb-application/src/use_cases/indexing_service.rs:21`

| Line | Component | Purpose |
|------|-----------|---------|
| 21 | `use tracing::{error, info, warn};` | Tracing imports |

---

## Provider-Level Observability

### 14. Git Submodule Logging
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/git/submodule.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 81 | `tracing::debug!(...)` | Submodule discovery logging |
| 92 | `tracing::warn!(error = %e, "Failed to list submodules");` | Submodule listing error |
| 106 | `tracing::warn!(...)` | Submodule initialization warning |
| 118 | `tracing::warn!(...)` | Submodule update warning |
| 145 | `tracing::debug!(...)` | Recursive traversal logging |
| 175 | `tracing::warn!(...)` | Submodule path warning |
| 187 | `tracing::info!(...)` | Successful operation logging |

### 15. Language Engine Logging
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/engine.rs:113`

| Line | Component | Purpose |
|------|-----------|---------|
| 113 | `tracing::warn!(...)` | Language engine warning |

### 16. Project Detection Logging
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/git/project_detection/`

| File | Line | Component | Purpose |
|------|------|-----------|---------|
| `npm.rs` | 48 | `tracing::debug!(path = ?package_path, error = %e, ...)` | NPM package.json error |
| `npm.rs` | 56 | `tracing::debug!(path = ?package_path, error = %e, ...)` | NPM parse error |
| `maven.rs` | 137 | `tracing::debug!(path = ?pom_path, error = %e, ...)` | Maven pom.xml error |
| `go.rs` | 50 | `tracing::debug!(path = ?gomod_path, error = %e, ...)` | Go go.mod error |
| `cargo.rs` | 38 | `tracing::debug!(path = ?manifest_path, error = %e, ...)` | Cargo.toml parse error |
| `cargo.rs` | 45 | `tracing::debug!(path = ?manifest_path, ...)` | Workspace root logging |
| `detector.rs` | 28 | `tracing::debug!(...)` | Detection logging |
| `detector.rs` | 37 | `tracing::warn!(...)` | Detection warning |
| `detector.rs` | 45 | `tracing::warn!(...)` | Detection warning |

### 17. Database Logging
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/database/sqlite/provider.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 90 | `tracing::info!("Connecting to SQLite database at: {}", path.display());` | Connection logging |
| 103 | `tracing::info!("Memory database initialized at {}", path.display());` | Memory DB logging |
| 110 | `tracing::warn!(...)` | Warning for database issues |
| 122 | `tracing::info!(...)` | Info logging |
| 128 | `tracing::error!(...)` | Error logging |
| 154 | `tracing::info!(backup = %backup.display(), "Old database backed up");` | Backup logging |

### 18. Vector Store Logging
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/vector_store/milvus.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 130 | `tracing::debug!("Flush attempt {} rate limited, retrying...", attempt + 1);` | Rate limit logging |
| 190 | `tracing::debug!(...)` | Debug logging |
| 379 | `tracing::debug!(...)` | Debug logging |
| 722 | `tracing::warn!(...)` | Warning logging |
| 878 | `tracing::warn!("Failed to query file paths: {}", e);` | Query error logging |
| 945 | `tracing::warn!("Failed to query chunks by file: {}", e);` | Chunk query error logging |

### 19. Event Bus Logging
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/events/nats.rs:38`

| Line | Component | Purpose |
|------|-----------|---------|
| 38 | `use tracing::{debug, info, warn};` | Tracing imports |

---

## Infrastructure Logging

### 20. Lifecycle Management
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/infrastructure/lifecycle.rs:61`

| Line | Component | Purpose |
|------|-----------|---------|
| 61 | `use tracing::{error, info, warn};` | Tracing imports |

### 21. Metrics Initialization
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/infrastructure/prometheus_metrics.rs:130`

| Line | Component | Purpose |
|------|-----------|---------|
| 130 | `tracing::error!("Failed to initialize Prometheus metrics: {}", e);` | Metrics error logging |

### 22. Configuration Watching
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/config/watcher.rs:14`

| Line | Component | Purpose |
|------|-----------|---------|
| 14 | `use tracing::warn;` | Tracing import |

### 23. DI Bootstrap
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/di/bootstrap.rs:26`

| Line | Component | Purpose |
|------|-----------|---------|
| 26 | `use tracing::info;` | Tracing import |

### 24. DI Catalog
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/di/catalog.rs:16`

| Line | Component | Purpose |
|------|-----------|---------|
| 16 | `use tracing::info;` | Tracing import |

---

## Testing & Validation

### 25. Logging Tests
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/tests/unit/logging_tests.rs:5`

| Line | Component | Purpose |
|------|-----------|---------|
| 5 | `use tracing::Level;` | Tracing Level import |

---

## Cargo.toml Dependencies

### 26. Dependency Declarations
**File**: `/home/marlonsc/crates/mcb-infrastructure/Cargo.toml:37-38,150`

```toml
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
tracing-appender = { workspace = true }
```

**File**: `/home/marlonsc/crates/mcb-server/Cargo.toml:56-57`

```toml
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

**File**: `/home/marlonsc/crates/mcb-providers/Cargo.toml:40`

```toml
tracing = { workspace = true }
```

---

## ADR Alignment

### ADR-019: Error Handling Strategy
- **Alignment**: Tracing used for structured error logging
- **Pattern**: `tracing::error!()` with context fields
- **Integration**: Error mapping with observability

### ADR-031: Documentation Excellence
- **Alignment**: Tracing provides runtime observability for documentation
- **Pattern**: Instrumentation macros for handler documentation

---

## Summary Table

| Aspect | Details |
|--------|---------|
| **Core Init** | `/home/marlonsc/mcb/crates/mcb-infrastructure/src/logging.rs:16-100` |
| **Handler Instrumentation** | 30+ `#[tracing::instrument]` macros across handlers |
| **Error Logging** | 20+ error logging sites in error_mapping.rs and handlers |
| **Provider Logging** | 15+ logging sites in git, language, database, vector store |
| **Admin Logging** | 20+ logging sites in admin handlers |
| **Bootstrap** | `/home/marlonsc/mcb/crates/mcb-server/src/init.rs:73` |
| **Tests** | Logging tests in infrastructure |
| **ADR** | ADR-019, ADR-031 |
| **Status** | IMPLEMENTED, production-ready |

