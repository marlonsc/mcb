# Documentation

<!-- TOC START -->

- [Deprecation Status](#deprecation-status)
- [↔ Code ↔ Docs cross-reference](#code-docs-cross-reference)
- [Overview](#overview)
- [Use Cases](#use-cases)
- [Decorators](#decorators)
- [Key Exports](#key-exports)
- [File Structure](#file-structure)
  - [Updated 2026-02-20 - Consolidated services.md into application.md for SSOT](#updated-2026-02-20-consolidated-servicesmd-into-applicationmd-for-ssot)

<!-- TOC END -->

> **Superseded (v0.3.0)**: The `mcb-application` crate was removed. Use case services
> moved to `mcb-infrastructure::di::modules::use_cases`. See
> [infrastructure module](./infrastructure.md).

---

<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->

# Application Layer (Use Cases)

## Deprecation Status

The `mcb-application` crate no longer exists in v0.3.0.

- Use case services were moved to `crates/mcb-infrastructure/src/services/`
- Historical rationale is documented in
  `docs/adr/046-integration-adr-034-037-policies.md`

**Source (historical)**: `crates/mcb-infrastructure/src/services/` **Current location**:
`crates/mcb-infrastructure/src/services/`

## ↔ Code ↔ Docs cross-reference

| Direction    | Link                                                                                                                                                                                    |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Code → Docs  | Historical `crates/mcb-infrastructure/src/services/` (crate removed in v0.3.0)                                                                                                                |
| Docs → Code  | Current equivalent: `crates/mcb-infrastructure/src/services/`                                                                                                               |
| Architecture | [`ARCHITECTURE.md`](../architecture/ARCHITECTURE.md) · [`ADR-013`](../adr/013-clean-architecture-crate-separation.md) · [`ADR-050`](../adr/050-manual-composition-root-dill-removal.md) |
| Roadmap      | [`ROADMAP.md`](../developer/ROADMAP.md)                                                                                                                                                 |

## Overview

The application layer orchestrates the flow of data between the user-facing gateways and
the domain layer. It contains the business use cases, ensuring that domain entities and
ports are used correctly to fulfill system requirements.

---

## Use Cases

These services implement the business logic defined in the domain ports.

- **IndexingService**
  (`crates/mcb-infrastructure/src/services/indexing_service/`):
  Coordinates codebase analysis, chunking, and storage into vector/lexical indexes.
- **SearchService**
  (`crates/mcb-infrastructure/src/services/search_service.rs`): Implements
  semantic, hybrid, and lexical search workflows.
- **ContextService**
  (`crates/mcb-infrastructure/src/services/context_service.rs`): Aggregates
  embeddings and vector data for query enrichment.
- **MemoryService**
  (`crates/mcb-infrastructure/src/services/memory_service/`): Manages
  observation capture and session awareness.
- **AgentSessionService**
  (`crates/mcb-infrastructure/src/services/agent_session_service.rs`):
  Orchestrates agent lifecycle, checkpoints, and tool call history.
- **Validation pipeline**: Validation concerns are now implemented in `mcb-validate` and
  wired from `mcb-server`/`mcb-infrastructure`.

---

## Decorators

The application layer uses decorators to add cross-cutting concerns (like metrics or
logging) to service implementations without bloating the core logic.

- **InstrumentedEmbedding**: Historical decorator; observability is now handled through
  the current infrastructure and server telemetry stack.

---

## Key Exports

```rust
pub use use_cases::agent_session_service::AgentSessionServiceImpl;
pub use use_cases::context_service::ContextServiceImpl;
pub use use_cases::indexing_service::IndexingServiceImpl;
pub use use_cases::memory_service::MemoryServiceImpl;
pub use use_cases::search_service::SearchServiceImpl;
```

## File Structure

```text
crates/mcb-infrastructure/src/services/
├── agent_session_service.rs  # Agent session orchestration
├── context_service.rs        # Context orchestration
├── indexing_service.rs       # Indexing orchestration
├── instrumented_embedding.rs # Metrics wrapper
├── memory_service.rs         # Memory orchestration
├── search_service.rs         # Search orchestration
└── validation_service.rs     # Validation orchestration
```

---

### Updated 2026-02-20 - Consolidated services.md into application.md for SSOT
