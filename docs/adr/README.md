# Architecture Decision Records

This directory contains all Architecture Decision Records (ADRs) for the Memory Context Browser project.

## Current ADRs

### Core Architecture (v0.1.2+)

- [ADR 001: Modular Crates Architecture](001-modular-crates-architecture.md)
- [ADR 002: Async-First Architecture](002-async-first-architecture.md) - Tokio async patterns
- [ADR 003: Unified Provider Architecture & Routing](003-unified-provider-architecture.md) - Provider abstraction, multi-provider routing, failover
- [ADR 004: Event Bus (Local and Distributed)](004-event-bus-local-distributed.md)
- [ADR 005: Context Cache Support (Moka and Redis)](005-context-cache-support.md)

### Documentation & Quality

- [ADR 006: Code Audit and Architecture Improvements](006-code-audit-and-improvements.md)
- [ADR 007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md)

### v0.3.0 Features (Planned)

- [ADR 008: Git-Aware Semantic Indexing v0.2.0](008-git-aware-semantic-indexing-v0.2.0.md)
- [ADR 009: Persistent Session Memory v0.2.0](009-persistent-session-memory-v0.2.0.md)
- [ADR 010: Hooks Subsystem with Agent-Backed Processing](010-hooks-subsystem-agent-backed.md)

### Infrastructure ADRs (v0.1.2)

- [ADR 011: HTTP Transport Request/Response Pattern](011-http-transport-request-response-pattern.md)
- [ADR 012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - DI strategy; see ADR-029 for current dill-based implementation
- [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Eight-crate workspace organization

### v0.1.2 Refatoracao & Simplification

- [ADR 023: Inventory to Linkme Migration](023-inventory-to-linkme-migration.md) - Plugin registration simplification
- [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md) - Supersedes former Shaku-based DI; see [ADR 029](029-hexagonal-architecture-dill.md)
- [ADR 025: Figment Configuration Migration](025-figment-configuration.md) - Unified configuration loading
- [ADR 026: API Routing Refactor (Rocket vs Poem)](026-routing-refactor-rocket-poem.md) - HTTP framework evaluation and migration

### v0.1.3 / v0.2.0 Architecture Evolution

- [ADR 027: Architecture Evolution v0.1.3](027-architecture-evolution-v013.md) - Onion/Clean enhancement with bounded contexts, engine contracts, incremental indexing - **Proposed**
- [ADR 028: Advanced Code Browser v0.2.0](028-advanced-code-browser-v020.md)
- [ADR 029: Hexagonal Architecture with dill](029-hexagonal-architecture-dill.md) - Current DI IoC container; handle-based pattern
- [ADR 030: Multi-Provider Strategy](030-multi-provider-strategy.md) - **into ADR-003** (see [ADR 003: Unified Provider Architecture & Routing](003-unified-provider-architecture.md))
- [ADR 031: Documentation Excellence](031-documentation-excellence.md) - Documentation standards and automation

### Phase 8-9: v0.3→v0.4.0 (Workflow + Integrated Context System)

#### Phase 8: Workflow FSM & Policies (v0.3.0) — ACCEPTED

- [ADR 033: MCP Handler Consolidation](033-mcp-handler-consolidation.md) - Handler registration and routing patterns
- [ADR 034: Workflow Core FSM](034-workflow-core-fsm.md) - Finite state machine and persistence
- [ADR 035: Context Scout](035-context-scout.md) - Project state discovery
- [ADR 036: Enforcement Policies](036-enforcement-policies.md) - Policy enforcement framework
- [ADR 037: Workflow Orchestrator](037-workflow-orchestrator.md) - Coordination and MCP integration
- [ADR 038: Multi-Tier Execution Model](038-multi-tier-execution-model.md) - Integration of ADR-034–037

#### Phase 9: Integrated Context System (v0.4.0)

- [ADR 041: Context Architecture](phase-9/README.md#adr-041-context-architecture) - 5-layer context system design
- [ADR 042: Knowledge Graph](phase-9/README.md#adr-042-knowledge-graph) - Graph structure and relationships
- [ADR 043: Hybrid Search Engine](phase-9/README.md#adr-043-hybrid-search-engine) - RRF fusion algorithm
- [ADR 044: Model Selection](phase-9/README.md#adr-044-model-selection) - Embedding and search model choices
- [ADR 045: Context Versioning](phase-9/README.md#adr-045-context-versioning) - Snapshot and temporal queries
- [ADR 046: Integration Patterns](phase-9/README.md#adr-046-integration-patterns) - MCP tool integration
- [ADR 047: Project Architecture](047-project-architecture.md) - Central Hub and Multi-Dimensional Coordination

### Implementation Timeline v0.1.2

#### Phase 1: Foundation (Week 1-2)

- [ADR 023] Inventory to Linkme Migration
- [ADR 024] Simplified Dependency Injection

#### Phase 2: Configuration & Routing (Week 3-4)

- [ADR 025] Figment Configuration Migration
- [ADR 026] API Routing Refactor (Rocket vs Poem)

#### Phase 3: Integration & Testing (Week 5-6)

- Integration testing across all changes
- Performance validation
- Documentation updates

**Target Release**: v0.1.2 (6 weeks from planning completion)

### Multi-Domain & Integration (v0.2.0+)

- [ADR 014: Multi-Domain Architecture Strategy](014-multi-domain-architecture.md) - Future domain expansion
- [ADR 015: Workspace Shared Libraries](015-workspace-shared-libraries.md) - Shared code in libs/
- [ADR 016: Integration Points and Adapter Pattern](016-integration-points-adapter-pattern.md) - PMAT integration pattern
- [ADR 017: Phased Feature Integration](017-phased-feature-integration.md) - Release roadmap
- [ADR 018: Hybrid Caching Strategy](018-hybrid-caching-strategy.md) - Moka + SHA256 caching
- [ADR 019: Error Handling Strategy](019-error-handling-strategy.md) - thiserror + anyhow integration
- [ADR 020: Testing Strategy Integration](020-testing-strategy-integration.md) - Test migration plan
- [ADR 021: Dependency Management](021-dependency-management.md) - Workspace dependencies
- [ADR 022: CI Integration Strategy](022-ci-integration-strategy.md) - Quality gates and benchmarks

## ADR Status Legend

| Status | Meaning |
|--------|---------|
| Proposed | Under discussion |
| Accepted | Approved and to be implemented |
| Implemented | Completed in codebase |
| Deprecated | No longer relevant |
| Superseded | Replaced by another ADR |

## ADR Count

**Total ADRs**: 47 (ADR-001 through ADR-047)

- **Core Architecture**: ADR-001-007 (7 ADRs)
- **v0.2.0 Features**: ADR-008-010 (3 ADRs)
- **Infrastructure**: ADR-011-022 (12 ADRs)
- **v0.1.2 Refactoring**: ADR-023-031 (9 ADRs)
- **Phase 8 (Workflow)**: ADR-034-037 (4 ADRs)
- **Phase 9 (Context)**: ADR-041-047 (7 ADRs)

## Creating New ADRs

Use the sequential numbering format: `XXX-descriptive-name.md`

See [ADR Template](../templates/adr-template.md) and [standard format](../architecture/ARCHITECTURE.md#adr-template).
