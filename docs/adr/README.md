<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Architecture Decision Records

This directory contains all Architecture Decision Records (ADRs) for the Memory Context Browser project.

## Current ADRs

### Core Architecture (v0.1.x+)

- [ADR 001: Modular Crates Architecture](001-modular-crates-architecture.md) — Workspace crate organization
- [ADR 002: Async-First Architecture](002-async-first-architecture.md) — Tokio async patterns
- [ADR 003: Unified Provider Architecture & Routing](003-unified-provider-architecture.md) — Provider abstraction, multi-provider routing, failover
- [ADR 005: Context Cache Support (Moka and Redis)](005-context-cache-support.md) — Hybrid caching layer
- [ADR 006: Code Audit and Architecture Improvements](006-code-audit-and-improvements.md) — Quality improvements

### v0.2.0 Features

- [ADR 008: Git-Aware Semantic Indexing](008-git-aware-semantic-indexing-v0.2.0.md) — Repository-level context
- [ADR 009: Persistent Session Memory](009-persistent-session-memory-v0.2.0.md) — Cross-session observation storage
- [ADR 010: Hooks Subsystem with Agent-Backed Processing](010-hooks-subsystem-agent-backed.md) — Agent hooks

### Infrastructure (v0.1.2)

- [ADR 011: HTTP Transport Request/Response Pattern](011-http-transport-request-response-pattern.md)
- [ADR 012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) — Historical; see ADR-050 for current composition root
- [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) — Seven-crate workspace

### Multi-Domain & Integration (v0.2.0+)

- [ADR 014: Multi-Domain Architecture Strategy](014-multi-domain-architecture.md) — Future domain expansion
- [ADR 015: Workspace Shared Libraries](015-workspace-shared-libraries.md) — Shared code in libs/
- [ADR 016: Integration Points and Adapter Pattern](016-integration-points-adapter-pattern.md) — PMAT integration
- [ADR 017: Phased Feature Integration](017-phased-feature-integration.md) — Release roadmap
- [ADR 018: Hybrid Caching Strategy](018-hybrid-caching-strategy.md) — Moka + SHA256 caching
- [ADR 019: Error Handling Strategy](019-error-handling-strategy.md) — thiserror + anyhow integration
- [ADR 020: Testing Strategy Integration](020-testing-strategy-integration.md) — Test migration plan
- [ADR 021: Dependency Management](021-dependency-management.md) — Workspace dependencies
- [ADR 022: CI Integration Strategy](022-ci-integration-strategy.md) — Quality gates and benchmarks

### v0.1.2 Refactoring & Simplification

- [ADR 023: Inventory to Linkme Migration](023-inventory-to-linkme-migration.md) — Plugin registration simplification
- [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md) — Historical; superseded by ADR-050

### v0.1.3 / v0.2.0 Architecture Evolution

- [ADR 027: Architecture Evolution v0.1.3](027-architecture-evolution-v013.md) — Onion/Clean enhancement — **Proposed**
- [ADR 028: Advanced Code Browser v0.2.0](028-advanced-code-browser-v020.md)
- [ADR 030: Multi-Provider Strategy](030-multi-provider-strategy.md) — Merged into ADR-003
- [ADR 031: Documentation Excellence](031-documentation-excellence.md) — Documentation standards

### v0.2.1 Additions

- [ADR 032: Agent & Quality Domain Extension](032-agent-quality-domain-extension.md) — **Superseded** by ADR-034
- [ADR 033: MCP Handler Consolidation](033-mcp-handler-consolidation.md) — Handler registration and routing
- [ADR 039: Context Persistence Boundary](039-context-persistence-boundary.md) — **Proposed**
- [ADR 040: Unified Tool Execution Gate](040-unified-tool-execution-gate.md) — **Proposed**

### v0.2.2 Observability

- [ADR 048: Gap-Free Observability Strategy](048-observability-strategy.md) — OpenTelemetry metrics, logs, traces — **Accepted**

### v0.3.0 — SeaQL + Loco Platform Rebuild (CURRENT)

The v0.3.0 release is a full platform rebuild on SeaQL (SeaORM, SeaQuery, SeaSchema, SeaStreamer) and Loco.rs. See [ADR 051](051-seaql-loco-platform-rebuild.md) for the master plan.

- [ADR 049: Axum Return for rmcp Tower Compatibility](049-axum-return-rmcp-tower-compatibility.md) — Supersedes ADR-026 — **Accepted**
- [ADR 050: Manual Composition Root — dill Removal](050-manual-composition-root-dill-removal.md) — Supersedes ADR-029 — **Implemented**
- [ADR 051: SeaQL + Loco.rs Platform Rebuild](051-seaql-loco-platform-rebuild.md) — Supersedes ADR-004, 007, 025, 026 — **Accepted**
- [ADR 052: Schema Resolution with SeaORM 2.x](052-schema-resolution-seaorm.md) — Domain-driven DDL generation — **Accepted**

### Workflow FSM & Policies (v0.4.0, previously v0.3.0)

- [ADR 034: Workflow Core FSM](034-workflow-core-fsm.md) — Finite state machine and persistence
- [ADR 035: Context Scout](035-context-scout.md) — Project state discovery
- [ADR 036: Enforcement Policies](036-enforcement-policies.md) — Policy enforcement framework
- [ADR 037: Workflow Orchestrator](037-workflow-orchestrator.md) — Coordination and MCP integration
- [ADR 038: Multi-Tier Execution Model](038-multi-tier-execution-model.md) — Integration of ADR-034–037

### Integrated Context System (v0.5.0, previously v0.4.0)

- [ADR 041: Context Architecture](phase-9/README.md#adr-041-context-architecture) — 5-layer context system design
- [ADR 042: Knowledge Graph](phase-9/README.md#adr-042-knowledge-graph) — Graph structure and relationships
- [ADR 043: Hybrid Search Engine](phase-9/README.md#adr-043-hybrid-search-engine) — RRF fusion algorithm
- [ADR 044: Model Selection](phase-9/README.md#adr-044-model-selection) — Embedding and search model choices
- [ADR 045: Context Versioning](phase-9/README.md#adr-045-context-versioning) — Snapshot and temporal queries
- [ADR 046: Integration Patterns](phase-9/README.md#adr-046-integration-patterns) — MCP tool integration
- [ADR 047: Project Architecture](047-project-architecture.md) — Central Hub and Multi-Dimensional Coordination

## Archived (Superseded)

These ADRs have been superseded by newer decisions and moved to [`archive/`](archive/):

| Archived ADR | Superseded By | Reason |
|---|---|---|
| 004 — Event Bus (Local and Distributed) | [ADR 051](051-seaql-loco-platform-rebuild.md) | Replaced by SeaStreamer event architecture |
| 007 — Integrated Web Administration Interface | [ADR 051](051-seaql-loco-platform-rebuild.md) | Replaced by Loco.rs + SeaORM Pro admin |
| 012 — DI Strategy (Shaku) | ADR-024 | Original Shaku-based DI approach |
| 024 — Simplified DI | [ADR 050](050-manual-composition-root-dill-removal.md) | Replaced by manual composition root |
| 025 — Figment Configuration | [ADR 051](051-seaql-loco-platform-rebuild.md) | Replaced by Loco YAML config system |
| 026 — Routing Refactor (Rocket vs Poem) | [ADR 049](049-axum-return-rmcp-tower-compatibility.md) | Replaced by Axum return for rmcp Tower |
| 029 — Hexagonal Architecture (dill) | [ADR 050](050-manual-composition-root-dill-removal.md) | Replaced by linkme + Handle pattern |
| 032 — Agent Quality Domain | [ADR 034](034-workflow-core-fsm.md) | Replaced by Workflow Core FSM |

## Version Roadmap (ADR alignment)

| Version | Theme | Key ADRs |
|---|---|---|
| v0.1.x | Core architecture, Clean Architecture layers | 001–006, 011–013 |
| v0.1.2 | Refactoring: linkme, simplified DI, Figment | 023–024, 027–031 |
| v0.2.0 | Git-aware indexing, persistent memory, hooks | 008–010, 014–022 |
| v0.2.1 | Handler consolidation, context boundaries | 033, 039–040 |
| v0.2.2 | Observability (OpenTelemetry) | 048 |
| **v0.3.0** | **SeaQL + Loco.rs platform rebuild** | **049–052** |
| v0.4.0 | Workflow FSM & enforcement policies | 034–038 |
| v0.5.0 | Integrated context system, knowledge graph | 041–047 |

## ADR Status Legend

| Status | Meaning |
|---|---|
| Proposed | Under discussion |
| Accepted | Approved and to be implemented |
| Implemented | Completed in codebase |
| Deprecated | No longer relevant |
| Superseded | Replaced by another ADR |

## ADR Count

**Total ADRs**: 52 (ADR-001 through ADR-052)

- **Active**: 47 ADRs in this directory
- **Archived**: 8 superseded ADRs in [`archive/`](archive/)
- **Core Architecture**: ADR-001–006 (5 active)
- **v0.2.0 Features**: ADR-008–010 (3 ADRs)
- **Infrastructure**: ADR-011–022 (12 ADRs)
- **v0.1.2 Refactoring**: ADR-023–031 (5 active, 3 archived)
- **v0.2.1 Additions**: ADR-032–033, 039–040 (4 ADRs)
- **v0.2.2 Observability**: ADR-048 (1 ADR)
- **v0.3.0 Platform Rebuild**: ADR-049–052 (4 ADRs)
- **v0.4.0 Workflow**: ADR-034–038 (5 ADRs)
- **v0.5.0 Context System**: ADR-041–047 (7 ADRs)

## Creating New ADRs

Use the sequential numbering format: `XXX-descriptive-name.md`

See [ADR Template](../templates/adr-template.md) and [standard format](../architecture/ARCHITECTURE.md#adr-template).
