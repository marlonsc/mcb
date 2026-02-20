<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Application Layer (Use Cases)

**Source**: `crates/mcb-application/src/`
**Crate**: `mcb-application`

## ↔ Code ↔ Docs cross-reference

| Direction | Link |
| --------- | ---- |
| Code → Docs | [`crates/mcb-application/src/lib.rs`](../../crates/mcb-application/src/lib.rs) links here |
| Docs → Code | [`crates/mcb-application/src/lib.rs`](../../crates/mcb-application/src/lib.rs) — crate root |
| Architecture | [`ARCHITECTURE.md`](../architecture/ARCHITECTURE.md) · [`ADR-013`](../adr/013-clean-architecture-crate-separation.md) · [`ADR-029`](../adr/029-hexagonal-architecture-dill.md) |
| Roadmap | [`ROADMAP.md`](../developer/ROADMAP.md) |

## Overview

The application layer orchestrates the flow of data between the user-facing gateways and the domain layer. It contains the business use cases, ensuring that domain entities and ports are used correctly to fulfill system requirements.

---

## Use Cases

These services implement the business logic defined in the domain ports.

- **IndexingService** ([`indexing_service.rs`](../../crates/mcb-application/src/use_cases/indexing_service.rs)): Coordinates codebase analysis, chunking, and storage into vector/lexical indexes.
- **SearchService** ([`search_service.rs`](../../crates/mcb-application/src/use_cases/search_service.rs)): Implements semantic, hybrid, and lexical search workflows.
- **ContextService** ([`context_service.rs`](../../crates/mcb-application/src/use_cases/context_service.rs)): Aggregates embeddings and vector data for query enrichment.
- **MemoryService** ([`memory_service.rs`](../../crates/mcb-application/src/use_cases/memory_service.rs)): Manages observation capture and session awareness.
- **AgentSessionService** ([`agent_session_service.rs`](../../crates/mcb-application/src/use_cases/agent_session_service.rs)): Orchestrates agent lifecycle, checkpoints, and tool call history.
- **ValidationService** ([`validation_service.rs`](../../crates/mcb-application/src/use_cases/validation_service.rs)): Runs multi-phase validation pipelines across the codebase.

---

## Decorators

The application layer uses decorators to add cross-cutting concerns (like metrics or logging) to service implementations without bloating the core logic.

- **InstrumentedEmbedding** ([`instrumented_embedding.rs`](../../crates/mcb-application/src/decorators/instrumented_embedding.rs)): Adds OpenTelemetry metrics to embedding generations.

---

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
├── decorators/          # Service decorators (Metrics, Logging)
├── use_cases/           # Implementation of domain port traits
├── constants.rs         # Shared app-layer constants
└── lib.rs               # Crate root
```

---

### Updated 2026-02-20 - Consolidated services.md into application.md for SSOT
