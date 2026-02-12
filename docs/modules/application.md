# application Module

**Source**: `crates/mcb-application/src/`
**Crate**: `mcb-application`
**Files**: 10+
**Lines of Code**: ~2,000

**Project links**: See `docs/architecture/ARCHITECTURE.md`, `docs/modules/domain.md`, `docs/developer/ROADMAP.md` for service architecture and roadmap alignment.

## Overview

The application module implements business logic services following Clean Architecture principles. It contains use cases (service implementations) and domain services (chunking orchestration, search logic).

## Key Components

### Use Cases (`use_cases/`)

Service implementations that orchestrate domain logic:

- `context_service.rs` - ContextServiceImpl: Embedding and vector operations
- `indexing_service.rs` - IndexingServiceImpl: Codebase indexing and processing
- `search_service.rs` - SearchServiceImpl: Query processing and ranking

### Domain Services (`domain_services/`)

Business logic components:

- `chunking.rs` - ChunkingOrchestrator: Batch file chunking coordination
- `search.rs` - Search domain logic and Result ranking

### Ports (`ports/`)

Service interface definitions:

- `infrastructure/sync.rs` - SyncProvider interface
- `providers/cache.rs` - CacheProvider interface

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

- **Architecture guidance**: `docs/architecture/ARCHITECTURE.md` explains the Clean Architecture layering and provider patterns used by this module.
- **Roadmap signals**: `docs/developer/ROADMAP.md` outlines v0.2.1 goals (git-aware indexing, session memory, advanced browser) that require resilient application services.
- **Operational metrics**: Coordinate behavior with `docs/operations/CHANGELOG.md`/`docs/operations/CI_OPTIMIZATION_VALIDATION.md` for test and validation metrics whenever you touch service tests.

---

*Updated 2026-01-18 - Reflects modular crate architecture (v0.2.1)*
