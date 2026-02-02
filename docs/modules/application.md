# application Module

**Source**: `crates/mcb-application/src/`
**Crate**: `mcb-application`
**Files**: 10+
**Lines of Code**: ~2,000

**Project links**: `docs/context/technical-patterns.md` (provider patterns), `docs/context/domain-concepts.md`, `docs/context/project-state.md`, `.planning/STATE.md` (Phase 6 progress), and `docs/developer/ROADMAP.md` (v0.2.0 plan) so service work aligns with Hybrid Search, git-aware requirements, and current validated goals.

## Overview

The application module implements business logic services following Clean Architecture principles. It contains use cases (service implementations) and domain services (chunking orchestration, search logic).

## Key Components

### Use Cases (`use_cases/`)

Service implementations that orchestrate domain logic:

-   `context_service.rs` - ContextServiceImpl: Embedding and vector operations
-   `indexing_service.rs` - IndexingServiceImpl: Codebase indexing and processing
-   `search_service.rs` - SearchServiceImpl: Query processing and ranking

### Domain Services (`domain_services/`)

Business logic components:

-   `chunking.rs` - ChunkingOrchestrator: Batch file chunking coordination
-   `search.rs` - Search domain logic and Result ranking

### Ports (`ports/`)

Service interface definitions:

-   `infrastructure/sync.rs` - SyncProvider interface
-   `providers/cache.rs` - CacheProvider interface

## File Structure

```text
crates/mcb-application/src/
├── use_cases/
│   ├── context_service.rs    # ContextServiceImpl
│   ├── indexing_service.rs   # IndexingServiceImpl
│   ├── search_service.rs     # SearchServiceImpl
│   └── mod.rs
├── domain_services/
│   ├── chunking.rs           # ChunkingOrchestrator
│   ├── search.rs             # Search logic
│   └── mod.rs
├── ports/
│   ├── infrastructure/       # Infrastructure port traits
│   └── providers/            # Provider port traits
└── lib.rs                    # Crate root
```

## Key Exports

```rust
// Service implementations
pub use use_cases::context_service::ContextServiceImpl;
pub use use_cases::indexing_service::IndexingServiceImpl;
pub use use_cases::search_service::SearchServiceImpl;

// Domain services
pub use domain_services::chunking::{ChunkingOrchestrator, ChunkingResult};
```

## Testing

Application tests are located in `crates/mcb-application/tests/`.

## Project Alignment

- **Phase context**: Follow `docs/context/project-state.md` and `.planning/STATE.md` while advancing Phase 6 Hybrid Search (06-02 plan) so changes to use cases and chunking services deliver on the roadmap.
- **Architecture guidance**: `docs/architecture/ARCHITECTURE.md` explains the Clean Architecture layering; `docs/context/technical-patterns.md` documents provider patterns used by this module.
- **Roadmap signals**: `docs/developer/ROADMAP.md` outlines v0.2.0 goals (git-aware indexing, session memory, advanced browser) that require resilient application services.
- **Operational metrics**: Coordinate behavior with `docs/operations/CHANGELOG.md`/`docs/operations/CI_OPTIMIZATION_VALIDATION.md` for test and validation metrics whenever you touch service tests.

---

*Updated 2026-01-18 - Reflects modular crate architecture (v0.1.2)*
