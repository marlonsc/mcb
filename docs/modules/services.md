<!-- markdownlint-disable MD013 -->
# Services Module

**Source**: `crates/mcb-application/src/use_cases/`
**Traits**: `crates/mcb-domain/src/ports/services/`
**Crate**: `mcb-application`

Orchestrates semantic indexing, search, memory, validation, and session workflows.

## Overview

The services module contains the application-layer use cases. Implementations are in `mcb-application`, and service interfaces are defined in domain ports under `mcb-domain/src/ports/services/`.

## Service Interface Traits

Service interfaces are defined in domain ports and resolved through DI.

```rust
pub trait ContextServiceInterface: Send + Sync { /* ... */ }
pub trait SearchServiceInterface: Send + Sync { /* ... */ }
pub trait IndexingServiceInterface: Send + Sync {
    fn index(&self, path: &Path, collection: &str) -> impl Future<Output = Result<IndexingResult>> + Send;
}
pub trait MemoryServiceInterface: Send + Sync { /* ... */ }
pub trait AgentSessionServiceInterface: Send + Sync { /* ... */ }
pub trait ValidationServiceInterface: Send + Sync { /* ... */ }
```

## Services

### ContextService

Coordinates embedding generation and vector storage operations.

**Location**: `crates/mcb-application/src/use_cases/context_service.rs`

### IndexingService

Processes codebases and creates searchable indexes.

**Location**: `crates/mcb-application/src/use_cases/indexing_service.rs`

### SearchService

Executes semantic and hybrid searches.

**Location**: `crates/mcb-application/src/use_cases/search_service.rs`

### MemoryService

Handles memory/observation workflows.

**Location**: `crates/mcb-application/src/use_cases/memory_service.rs`

### AgentSessionService

Manages agent session lifecycle operations.

**Location**: `crates/mcb-application/src/use_cases/agent_session_service.rs`

### ValidationService

Runs validation workflows and quality checks.

**Location**: `crates/mcb-application/src/use_cases/validation_service.rs`

## Integration Points

- **Providers**: [providers.md](./providers.md)
- **Domain ports**: [domain.md](./domain.md)
- **Server handlers**: [server.md](./server.md)

## Key Exports

```rust
pub use use_cases::agent_session_service::AgentSessionServiceImpl;
pub use use_cases::context_service::ContextServiceImpl;
pub use use_cases::indexing_service::IndexingServiceImpl;
pub use use_cases::memory_service::MemoryServiceImpl;
pub use use_cases::search_service::SearchServiceImpl;
pub use use_cases::validation_service::ValidationServiceImpl;
```

## File Structure

```text
crates/mcb-application/src/
├── constants.rs
├── decorators/
│   └── instrumented_embedding.rs
├── use_cases/
│   ├── agent_session_service.rs
│   ├── context_service.rs
│   ├── indexing_service.rs
│   ├── memory_service.rs
│   ├── search_service.rs
│   ├── validation_service.rs
│   └── mod.rs
└── lib.rs

crates/mcb-domain/src/ports/services/
├── agent.rs
├── chunking.rs
├── context.rs
├── hash.rs
├── indexing.rs
├── memory.rs
├── project.rs
├── search.rs
├── validation.rs
└── mod.rs
```

## Testing

See `crates/mcb-application/tests/` for service tests.

---

### Updated 2026-02-12 - Reflects modular crate architecture (v0.2.1)
