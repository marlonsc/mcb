# Development Roadmap

## Overview

This roadmap outlines the development of MCP Context Browser, a drop-in replacement for Claude-context with enhanced capabilities for semantic code search.

---

## Current Status

### v0.2.0 - Documentation Refactoring 🎉 RELEASED

**Status**: Released
**Release Date**: February 5, 2026
**Key Architecture**: ADR-003 (Provider), ADR-029 (Hexagonal DI)

MCP Context Browser v0.2.0 completes documentation refactoring with ADR consolidation, YAML metadata standardization, and cross-reference updates.

#### Achievements

**Documentation Consolidation:**

-   ✅ 44 ADRs with standardized YAML frontmatter (ADR, title, status, created, updated, related, supersedes, superseded_by, implementation_status)
-   ✅ 3 deprecated ADRs archived to `docs/adr/archive/` (ADR-012, ADR-024, ADR-032)
-   ✅ ADR-003 + ADR-030 into unified "Provider Architecture & Routing"
-   ✅ 5-value status set standardized (IMPLEMENTED, ACCEPTED, PROPOSED, SUPERSEDED, ARCHIVED)

**Cross-Reference Updates:**

-   ✅ All Shaku references removed (→ ADR-029 dill-based DI)
-   ✅ ADR-024 → ADR-029 migration documented
-   ✅ ADR-030 → ADR-003 consolidation verified
-   ✅ ADR-032 → ADR-034 supersession mapped

**Metrics:**

-   ADRs: 44 with metadata, 3 archived, 1 consolidation
-   Tests: 2040+ passing
-   Code: 7 commits, 801+ lines added, 874+ lines removed (net cleanup)
-   Violations: 0 architecture, 0 lint errors

---

### v0.3.0 - Workflow System Implementation 🚀 IN DEVELOPMENT

**Status**: Planning/Spec Finalization
**Target Date**: Q1 2026 (4-8 weeks)
**Key Architecture**: ADR-034 (FSM), ADR-035 (Scout), ADR-036 (Policies), ADR-037 (Orchestrator), ADR-038 (Tiers)

MCP Context Browser v0.3.0 implements complete workflow system with FSM-based task orchestration, context scouting, and policy enforcement.

#### Planned Achievements

**Workflow Core (ADR-034-038):**

-   WorkflowFSM state machine for task orchestration
-   ContextScout for context gathering and search
-   PolicyEngine for workflow validation and enforcement
-   TaskOrchestrator for multi-layer task coordination
-   ExecutionTiers for hierarchical execution management

**Infrastructure:**

-   5 new workflow system modules/crates
-   Unit tests (target: 95%+ coverage)
-   Integration tests with existing providers
-   Complete rustdoc API documentation

**Quality & Release:**

-   `make quality` passes (0 errors)
-   `make validate` passes (0 violations)
-   Performance benchmarks
-   Migration guide from v0.2.0

**Estimated Effort**: 4-8 weeks (parallelizable features)

#### Unblocks

-   ✅ v0.4.0 Integrated Context System (depends on workflow APIs)
-   ✅ Multi-agent collaboration infrastructure
-   ✅ Session lifecycle management

---

### v0.4.0 - Integrated Context System 🎨 PLANNED

**Status**: Design Phase (Parallel to v0.3.0)
**Target Date**: Q2 2026 (after v0.3.0)
**Key Architecture**: ADR-041-046 (Context System), depends on v0.3.0 APIs

MCP Context Browser v0.4.0 implements integrated context system with multi-agent collaboration, context merging, and session lifecycle management.

#### Planned Achievements

**Context System:**

-   Multi-agent context aggregation
-   Context merging and conflict resolution
-   Session-based lifecycle management
-   Global memory patterns
-   Hierarchical planning support

**Integration:**

-   Workflow FSM extensions (v0.3.0 compatibility)
-   Context Scout multi-agent support
-   Policy-aware context merging
-   Task orchestration with context awareness

**Quality & Release:**

-   Full integration test suite
-   Performance benchmarks (context merging)
-   Documentation + examples
-   Migration guide from v0.3.0

**Estimated Effort**: 6-10 weeks (dependent on v0.3.0)

#### Coordination

-   **Sync**: Weekly with v0.3.0 agent
-   **API Lock-in**: Week 6 (v0.3.0 finalizes interfaces)
-   **Implementation Start**: Week 8 (post v0.3.0 release)
-   **Timeline**: Q2 2026 release

---

### v0.1.4 - RCA Integration + Security Fixes ✅ PREVIOUS

**Status**: Released
**Release Date**: January 28, 2026
**Key Architecture**: ADR-001 (Modular Crates), ADR-002 (Async-First), ADR-029 (Hexagonal DI)

MCP Context Browser v0.1.4 completes Rust-code-analysis (RCA) integration, fixes security vulnerabilities, and updates dependencies.

#### Achievements

**RCA Integration:**

-   ✅ Migrated unwrap_detector.rs to RCA Callback pattern
-   ✅ Deleted legacy AST executor code (240 lines removed)
-   ✅ Removed TOML fallback from rete_engine.rs
-   ✅ Added INTERNAL_DEP_PREFIX constant

**Security Fixes:**

-   ✅ Removed atty dependency (GHSA-g98v-hv3f-hcfr vulnerability)
-   ✅ Replaced with std::io::IsTerminal (stable since Rust 1.70)

**Dependency Updates:**

<!-- markdownlint-disable MD044 -->
-   ✅ uuid 1.20.0, clap 4.5.55, rust-rule-engine 1.18.26
-   ✅ jsonwebtoken 10.3.0, dirs 6.0.0, moka 0.12.13
-   ✅ chrono 0.4.43, thiserror 2.0.18, proc-macro2 1.0.106
<!-- markdownlint-enable MD044 -->

**Metrics:**

-   Tests: 950+ passing (up from 790+)
-   Code reduction: ~607 lines net reduction
-   Architecture violations: 0

**Known transitive RUSTSEC (no fix available from upstream):**

-   atomic-polyfill@1.0.3 (RUSTSEC-2023-0089), number_prefix@0.4.0 (RUSTSEC-2025-0119), paste@1.0.15 (RUSTSEC-2024-0436), rustls-pemfile@2.2.0 (RUSTSEC-2025-0134) — pulled in by fastembed, indicatif, tokenizers, tonic; crates are unmaintained or deprecated. Resolve when upstream deps update or via workspace patch.

---

### v0.1.2 - Provider Modernization + Validation Tooling

**Status**: Released
**Release Date**: January 18, 2026
**Key Architecture**: ADR-023 (Linkme Migration), ADR-027 (Architecture Evolution)

MCP Context Browser v0.1.2 modernizes provider registration using compile-time linkme distributed slices and introduces the mcb-validate crate scaffolding.

#### Achievements

**Provider Modernization:**

-   ✅ All 15 providers migrated to linkme distributed slices (compile-time registration)
-   ✅ 4 pure linkme registries (embedding, vector store, cache, language)
-   ✅ Zero runtime overhead (provider discovery at compile time)
-   ✅ Eliminated inventory dependency (removed from Cargo.toml)

**Architecture Validation Scaffolding (mcb-validate):**

-   ✅ New mcb-validate crate (8th crate in workspace)
-   ✅ Phase 1: Linters verified (17/17 tests pass)
-   ✅ Phase 2: AST verified (26/26 tests pass)
-   ✅ Phase 3: Rule Engines verified (30/30 tests pass)
-   ✅ Phases 4–7: Metrics, Duplication, Architecture (CA001–CA009), Integration verified
-   ✅ 12 migration validation rules (YAML files in rules/migration/)
-   ✅ 750+ mcb-validate tests; 2982+ tests project-wide

**Admin UI Code Browser:**

-   ✅ VectorStoreBrowser trait in mcb-domain (ports layer)
-   ✅ CollectionInfo value object for browse metadata
-   ✅ 6 provider implementations (Milvus, InMemory, Filesystem, EdgeVec, Null, Encrypted)
-   ✅ REST API handlers (list collections, files, chunks)
-   ✅ 3 UI pages (collections grid, files list, code chunks)
-   ✅ Prism.js syntax highlighting in code viewer
-   ✅ Nav links added to all admin pages

**Verification Date**: 2026-01-28 via `make test`. See `docs/developer/IMPLEMENTATION_STATUS.md`.

**Maintained from v0.1.1:**

-   ✅ 2982+ tests with comprehensive coverage (100% pass rate)
-   ✅ 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null)
-   ✅ 6 vector stores (Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null)
-   ✅ 14 languages with AST parsing support
-   ✅ Clean architecture with dill-based DI (ADR-029)

#### Technical Metrics

-   **Source Files**: 340+ Rust files
-   **Test Suite**: 2982+ tests passing
-   **Crates**: 8 (7 + mcb-validate)
-   **Validation Rules**: 12 YAML migration rules; CA001–CA009 architecture rules
-   **Provider Registration**: Compile-time via linkme (inventory removed)
-   **mcb-validate Status**: Phases 1–7 verified (2982+ tests)

---

## Recent Releases

### v0.1.0 - First Stable Release ✅ RELEASED

**Status**: Production-Ready
**Release Date**: January 11, 2026
**Key Architecture**: ADR-001 (Modular Crates), ADR-002 (Async-First), ADR-013 (Clean Architecture)

MCP Context Browser v0.1.0 is the first stable release, providing a complete drop-in replacement for Claude-context with superior performance and expanded capabilities.

#### Achievements

-   ✅ Full MCP protocol implementation (4 tools)
-   ✅ 14 languages with AST parsing (Rust, Python, JS/TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin)
-   ✅ 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null)
-   ✅ 6 vector stores (In-Memory, Encrypted, Null)
-   ✅ Claude-context environment variable compatibility
-   ✅ 2982+ tests with comprehensive coverage (100% pass rate)
-   ✅ JWT authentication and rate limiting
-   ✅ Clean architecture with trait-based dependency injection
-   ✅ HTTP transport foundation for future enhancements
-   ✅ Systemd service integration
-   ✅ Binary auto-respawn mechanism

---

## Upcoming Releases

### v0.3.0 - Workflow System + Feature Implementation 🚀 IN DEVELOPMENT

**Status**: Planning/Spec Finalization
**Target Date**: Q1 2026 (4-8 weeks)
**Priority**: High
**Key Architecture**: ADR-008 (Git-Aware Indexing), ADR-009 (Session Memory), ADR-028 (Code Browser), ADR-034 (FSM), ADR-035 (Scout), ADR-036 (Policies), ADR-037 (Orchestrator), ADR-038 (Tiers)

#### Vision

Transform MCP Context Browser into a comprehensive development platform combining FSM-based task orchestration with git-aware semantic search, persistent cross-session memory, and IDE-like code browsing.

#### Objectives

**Workflow Core (ADR-034-038):**

-   WorkflowFSM state machine for task orchestration
-   ContextScout for context gathering and search
-   PolicyEngine for workflow validation and enforcement
-   TaskOrchestrator for multi-layer task coordination
-   ExecutionTiers for hierarchical execution management

**Git-Aware Semantic Indexing (ADR-008):**

-   Project-relative indexing (portable)
-   Multi-branch support with commit history
-   Change impact analysis
-   Monorepo and submodule support

**Persistent Session Memory (ADR-009):**

-   Cross-session observation storage
-   Semantic search over past decisions and work
-   Token-efficient progressive disclosure (3-layer workflow)
-   Context injection for session continuity

**Advanced Code Browser UI (ADR-028):**

-   Tree view navigation with collapsible directories
-   Full syntax highlighting with chunk boundary markers
-   Inline search Result highlighting
-   Keyboard shortcuts and dark mode
-   Real-time SSE updates during indexing

#### Infrastructure

-   5+ new workflow system modules
-   Unit tests (target: 95%+ coverage)
-   Integration tests with existing providers
-   Complete rustdoc API documentation

#### Quality & Release

-   `make quality` passes (0 errors)
-   `make validate` passes (0 violations)
-   Performance benchmarks
-   Migration guide from v0.2.0

**Estimated Effort**: 4-8 weeks (parallelizable features)

#### Unblocks

-   ✅ v0.4.0 Integrated Context System (depends on workflow APIs)
-   ✅ Multi-agent collaboration infrastructure
-   ✅ Session lifecycle management

---

### Future - Advanced Code Intelligence 📋 CONCEPTUAL

**Status**: Conceptual
**Priority**: Medium
**Dependencies**: v0.3.0 completion
**Key Architecture**: ADR-039 (Symbol Extraction), ADR-040 (Call Graph Analysis)

#### Vision

Enhance semantic code search with deep code intelligence features, enabling advanced analysis beyond keyword and semantic matching. This version focuses on understanding code relationships and providing actionable insights for refactoring and optimization.

#### Objectives

| Feature | Description | Benefit |
|---------|-------------|---------|
|**Symbol Extraction**| Extract and index all symbols (functions, classes, variables) | Navigate code by symbols, not just files |
|**Cross-Referencing**| Build symbol usage graph across codebase | "Find all usages" with precision |
|**Call Graph Analysis**| Map function call relationships | Understand execution paths |
|**Dependency Mapping**| Visualize module and package dependencies | Identify refactoring opportunities |
|**Code Similarity**| Detect duplicate or similar code patterns | Reduce code duplication |
|**Refactoring Suggestions**| AI-powered refactoring recommendations | Improve code quality |

#### Technical Approach

-   **AST Enhancement**: Extend existing tree-sitter integration with symbol extraction
-   **Graph Database**: Consider Neo4j or in-memory graph for relationships
-   **Incremental Updates**: Update graph on file changes (not full reindex)
-   **MCP Tools**: New tools for symbol search, call graph queries, similarity detection

#### Success Metrics

-   Symbol extraction: <1s for 10,000 LOC
-   Cross-reference lookup: <100ms
-   Call graph generation: <5s for large projects
-   Similarity detection: >90% accuracy

---

### v0.4.0 - Integrated Context System 📋 PLANNED

**Status**: Planning Complete (ADR-041-046)
**Priority**: High
**Dependencies**: v0.3.0 completion
**Timeline**: Feb 17 - Mar 16, 2026 (4 weeks)
**Key Architecture**: ADR-041-046 (Context Architecture, Knowledge Graph, Hybrid Search, Versioning, Integration)

#### Vision

Implement an integrated context system with knowledge graphs, freshness tracking, and time-travel queries. This system enables intelligent, adaptive code search with temporal awareness and policy-driven context discovery.

#### Objectives

| Feature | Description | Benefit |
|---------|-------------|---------|
|**Knowledge Graph**| Code relationships (calls, imports, extends) | Understand code structure and dependencies |
|**Freshness Tracking**| Temporal metadata and staleness signals | Find current, relevant code patterns |
|**Time-Travel Queries**| Query code as it existed at specific commits | Understand code evolution and regressions |
|**Hybrid Search**| RRF fusion of semantic + keyword search | Better relevance through dual ranking |
|**Context Snapshots**| Immutable captures of code state | Enable temporal queries and versioning |
|**Policy-Driven Discovery**| Freshness and validation policies | Enforce context quality standards |

#### Technical Approach

-   **5-Layer Architecture**: Code indexing → Graph → Hybrid search → Versioning → Integration
-   **Knowledge Graph**: petgraph-based with relationship types (calls, imports, extends, implements)
-   **Hybrid Search**: RRF (Reciprocal Rank Fusion) combining semantic embeddings + BM25 keyword search
-   **Snapshot Versioning**: Immutable captures at commits/tags with temporal query support
-   **Policy Enforcement**: Freshness gates and validation at FSM state boundaries
-   **MCP Integration**: Enhanced search, index, memory, session tools with new parameters

#### New Capabilities

**Freshness-Aware Search**:

```bash
mcb search --query "authenticate" --freshness-max-age 7
# Returns only patterns < 7 days old with staleness warnings
```

**Time-Travel Queries**:

```bash
mcb search --query "auth" --snapshot v0.2.0
# Show authentication patterns as they existed in v0.2.0
```

**Policy-Driven Context**:

```bash
mcb search --query "API docs" --policy api_docs
# Apply "API docs must be < 7 days old" policy
```

#### Success Metrics

-   2982+ tests passing (unit, integration, end-to-end)
-   Knowledge graph: <1s for 10,000 nodes
-   Hybrid search: <500ms average query time
-   Snapshot creation: <5s for large codebases
-   Zero architecture violations
-   Complete documentation and migration guide

---

### v1.0.0 - Production Enterprise 📋 FUTURE

**Status**: Conceptual
**Priority**: High
**Dependencies**: v0.4.0 completion
**Key Architecture**: ADR-044 (HA Architecture), ADR-045 (Disaster Recovery), ADR-046 (Compliance Framework)

#### Vision

Deliver a fully production-ready enterprise platform with SLA guarantees, professional support, compliance certifications, and high-availability deployment options suitable for mission-critical use cases in large enterprises.

#### Objectives

| Feature | Description | Benefit |
|---------|-------------|---------|
|**Full Enterprise Feature Set**| All v0.2.0-v0.4.0 features polished and hardened | Production-grade reliability |
|**SLA Guarantees**| 99.9% uptime commitment with monitoring | Business continuity |
|**Professional Support**| 24/7 support with response time SLAs | Enterprise peace of mind |
|**Compliance Certifications**| SOC 2 Type II, ISO 27001, GDPR | Regulatory compliance |
|**High Availability**| Multi-region deployment with automatic failover | Zero downtime |
|**Disaster Recovery**| Backup/restore with point-in-time recovery | Data protection |

#### Technical Approach

-   **HA Architecture**: Active-active deployment across multiple regions
-   **Automated Backup**: Continuous backup with 99.999% durability
-   **Monitoring & Alerting**: Comprehensive observability with PagerDuty integration
-   **Compliance Framework**: Automated compliance checking and reporting
-   **Documentation**: Professional documentation with support portal
-   **Certification Process**: Third-party security audits and certifications

#### Success Metrics

-   Uptime: 99.9% (measured monthly)
-   Response time: P95 <200ms for search queries
-   Support SLA: <15 min for critical issues
-   Compliance: 100% audit pass rate
-   Recovery Time Objective (RTO): <1 hour
-   Recovery Point Objective (RPO): <15 minutes

#### Certification Timeline

| Certification | Timeline | Estimated Cost |
|---------------|----------|----------------|
| SOC 2 Type I | Months 1-3 | $25k-$50k |
| SOC 2 Type II | Months 4-9 | $50k-$100k |
| ISO 27001 | Months 6-12 | $30k-$75k |
| GDPR Compliance | Months 1-6 | $10k-$25k |

---

## Version History

| Version | Status | Key Features |
|---------|--------|--------------|
| v0.0.1 | Released | Initial prototype |
| v0.0.2 | Released | Core architecture |
| v0.0.3 | Released | Production foundation |
| v0.1.0 | Released | Documentation excellence, clean architecture, first stable release |
| v0.1.1 | Released | Modular crate architecture (7 crates), DI foundation |
| v0.1.2 | Released | Linkme provider registration, mcb-validate Phases 1-3, Admin UI Browse |
| v0.1.3 | Released | RCA integration (unwrap_detector), executor deletion, 497 lines removed |
| v0.1.4 | Released | Complete RCA integration, atty security fix, dependency updates, 2982+ tests |
| v0.2.0 | **Released** | Documentation refactoring, ADR consolidation, architecture audit |
| v0.3.0 | **In Development** | Workflow System (ADR-034-038), Git indexing, Session memory, Code browser |
| v0.4.0 | Planned | Integrated Context System (Phase 9: Knowledge Graph, Hybrid Search, Versioning) |
| v1.0.0 | Future | Production enterprise |

---

## Implementation Principles

### Development Practices

1.**ADR-Driven Development**: Architectural decisions documented before implementation
2.**Test-First**: Core functionality developed with comprehensive tests
3.**Clean Architecture**: Separation of concerns with trait-based DI
4.**Documentation First**: Documentation updated with each code change
5.**Security by Design**: Security considerations in every component

### Quality Gates

All releases must pass:

-   [ ] All tests pass (unit, integration, e2e)
-   [ ] Code coverage meets targets (>85%)
-   [ ] Clippy lint clean
-   [ ] Security audit clean
-   [ ] Performance benchmarks maintained
-   [ ] Documentation complete and accurate

---

## Cross-References

-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
-   **Contributing**: [CONTRIBUTING.md](./CONTRIBUTING.md)
-   **ADR Index**: [docs/ADR/README.md](../adr/README.md)
-   **Version History**: [VERSION_HISTORY.md](../VERSION_HISTORY.md)
-   **Deployment**: [DEPLOYMENT.md](../operations/DEPLOYMENT.md)
