# application Module

**Source**: `crates/mcb-application/src/`
**Crate**: `mcb-application`
**Files**: 11 files
**Lines of Code**: ~2,000

**Project links**: See `docs/architecture/ARCHITECTURE.md`, `docs/modules/domain.md`, `docs/developer/ROADMAP.md`.

## Overview

The application module implements use-case services and decorators following Clean Architecture. The crate uses `use_cases/` and `decorators/` as its service organization.

## Key Components

### Use Cases (`use_cases/`)

- `agent_session_service.rs` - Agent session lifecycle use cases
- `context_service.rs` - Embedding and vector operations
- `indexing_service.rs` - Codebase indexing and processing
- `memory_service.rs` - Observation/memory use cases
- `search_service.rs` - Query processing and ranking
- `validation_service.rs` - Validation orchestration

### Decorators (`decorators/`)

- `instrumented_embedding.rs` - Embedding provider metrics instrumentation

### Root Files

- `constants.rs` - Shared application constants
- `lib.rs` - Crate root and exports

## File Structure

```text
crates/mcb-application/src/
├── constants.rs
├── decorators/
│   ├── instrumented_embedding.rs
│   └── mod.rs
├── use_cases/
│   ├── agent_session_service.rs
│   ├── context_service.rs
│   ├── indexing_service.rs
│   ├── memory_service.rs
│   ├── search_service.rs
│   ├── validation_service.rs
│   └── mod.rs
└── lib.rs
```

## Key Exports

```rust
pub use use_cases::agent_session_service::AgentSessionServiceImpl;
pub use use_cases::context_service::ContextServiceImpl;
pub use use_cases::indexing_service::IndexingServiceImpl;
pub use use_cases::memory_service::MemoryServiceImpl;
pub use use_cases::search_service::SearchServiceImpl;
pub use use_cases::validation_service::ValidationServiceImpl;
```

## Testing

Application tests are located in `crates/mcb-application/tests/`.

---

*Updated 2026-02-12 - Reflects modular crate architecture (v0.2.1)*
