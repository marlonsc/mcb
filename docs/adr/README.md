# Architecture Decision Records

This directory contains all Architecture Decision Records (ADRs) for the MCP Context Browser project.

## Current ADRs

### Core Architecture (v0.1.2)

-   [ADR 001: Modular Crates Architecture](001-modular-crates-architecture.md)
-   [ADR 002: Dependency Injection with Shaku](002-dependency-injection-shaku.md)
-   [ADR 003: Unified Provider Architecture](003-unified-provider-architecture.md)
-   [ADR 004: Event Bus (Local and Distributed)](004-event-bus-local-distributed.md)
-   [ADR 005: Context Cache Support (Moka and Redis)](005-context-cache-support.md)

### Documentation & Quality

-   [ADR 006: Code Audit and Architecture Improvements](006-code-audit-and-improvements.md)
-   [ADR 007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md)

### v0.2.0 Features (Planned)

-   [ADR 008: Git-Aware Semantic Indexing v0.2.0](008-git-aware-semantic-indexing-v0.2.0.md)
-   [ADR 009: Persistent Session Memory v0.2.0](009-persistent-session-memory-v0.2.0.md)
-   [ADR 010: Hooks Subsystem with Agent-Backed Processing](010-hooks-subsystem-agent-backed.md)

### Foundation Architecture (Complementary to Core)

> Note: These ADRs use numbers 002-005 for historical reasons but cover DIFFERENT topics than the Core Architecture ADRs above. Both sets are valid and complementary.

-   [ADR 002: Async-First Architecture](002-async-first-architecture.md) - Tokio async patterns
-   [ADR 003: C4 Model Documentation](003-c4-model-documentation.md) - Architecture visualization
-   [ADR 004: Multi-Provider Strategy](004-multi-provider-strategy.md) - Provider routing and failover
-   [ADR 005: Documentation Excellence](005-documentation-excellence.md) - Documentation standards

### Infrastructure ADRs (v0.1.2)

-   [ADR 011: HTTP Transport Request/Response Pattern](011-http-transport-request-response-pattern.md)
-   [ADR 012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - Shaku modules + runtime factories
-   [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Eight-crate workspace organization

### v0.1.2 Refatoracao & Simplification

-   [ADR 023: Inventory to Linkme Migration](023-inventory-to-linkme-migration.md) - Plugin registration simplification
-   [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md) - **SUPERSEDES** [ADR 002](002-dependency-injection-shaku.md)
-   [ADR 025: Figment Configuration Migration](025-figment-configuration.md) - Unified configuration loading
-   [ADR 026: API Routing Refactor (Rocket vs Poem)](026-routing-refactor-rocket-poem.md) - HTTP framework evaluation and migration

### v0.1.3 Architecture Evolution

-   [ADR 027: Architecture Evolution v0.1.3](027-architecture-evolution-v013.md) - Onion/Clean enhancement with bounded contexts, engine contracts, and incremental indexing - **Proposed**

### Implementation Timeline v0.1.2

#### Phase 1: Foundation (Week 1-2)

-   [ADR 023] Inventory to Linkme Migration
-   [ADR 024] Simplified Dependency Injection

#### Phase 2: Configuration & Routing (Week 3-4)

-   [ADR 025] Figment Configuration Migration
-   [ADR 026] API Routing Refactor (Rocket vs Poem)

#### Phase 3: Integration & Testing (Week 5-6)

-   Integration testing across all changes
-   Performance validation
-   Documentation updates

**Target Release**: v0.1.2 (6 weeks from planning completion)

### Multi-Domain & Integration (v0.2.0+)

-   [ADR 014: Multi-Domain Architecture Strategy](014-multi-domain-architecture.md) - Future domain expansion
-   [ADR 015: Workspace Shared Libraries](015-workspace-shared-libraries.md) - Shared code in libs/
-   [ADR 016: Integration Points and Adapter Pattern](016-integration-points-adapter-pattern.md) - PMAT integration pattern
-   [ADR 017: Phased Feature Integration](017-phased-feature-integration.md) - Release roadmap
-   [ADR 018: Hybrid Caching Strategy](018-hybrid-caching-strategy.md) - Moka + SHA256 caching
-   [ADR 019: Error Handling Strategy](019-error-handling-strategy.md) - thiserror + anyhow integration
-   [ADR 020: Testing Strategy Integration](020-testing-strategy-integration.md) - Test migration plan
-   [ADR 021: Dependency Management](021-dependency-management.md) - Workspace dependencies
-   [ADR 022: CI Integration Strategy](022-ci-integration-strategy.md) - Quality gates and benchmarks

## ADR Status Legend

| Status | Meaning |
|--------|---------|
| Proposed | Under discussion |
| Accepted | Approved and to be implemented |
| Implemented | Completed in codebase |
| Deprecated | No longer relevant |
| Superseded | Replaced by another ADR |

## Creating New ADRs

Use the sequential numbering format: `XXX-descriptive-name.md`

See [ADR Template](../architecture/ARCHITECTURE.md#adr-template) for the standard format
