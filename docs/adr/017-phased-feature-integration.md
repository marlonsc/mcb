<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 17
title: Phased Feature Integration Roadmap
status: ACCEPTED
created:
updated: 2026-02-05
related: [12, 13, 16, 20]
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 017: Phased Feature Integration Roadmap

## Status

**Accepted** (v0.1.1+)
**Date**: 2026-01-14
**Version**: v0.1.1 Update

## Context

MCB and PMAT have distinct feature sets. Integration must be incremental to maintain stability.

**MCB v0.1.1** (Semantic Search - RELEASED):

- 4 MCP tools (index, search, clear, status)
- 308+ tests (100% pass rate)
- Seven-crate Clean Architecture
- Two-layer DI strategy (linkme + handles; ADR-050, ADR-029 superseded)

**PMAT** (Code Analysis):

- Complexity analysis (cyclomatic, cognitive)
- TDG scoring (Technical Debt Gradient)
- SATD detection (Self-Admitted Technical Debt)
- Refactoring suggestions
- 4600+ tests

## Decision

**6-phase integration roadmap** with backward compatibility:

### Phase 1: v0.1.1 - Foundation (RELEASED)

**Focus**: Seven-Crate Clean Architecture

Deliverables:

- [x] mcb-domain: Core entities, repositories, events, value objects
- [x] mcb-application: Port traits (~20+) + Context, Search, Indexing services
- [x] mcb-providers: 7 embedding + 4 vector store providers
- [x] mcb-infrastructure: linkme + Handle DI + config management
- [x] mcb-server: MCP protocol + Admin API
- [x] mcb-validate: Architecture enforcement
- [x] mcb: Facade crate with re-exports

**Tools**: 4 (index, search, clear, status)
**Tests**: 308+

### Phase 2: v0.2.0 - Infrastructure (NEXT)

**Focus**: Git-aware indexing + Session memory

Deliverables:

- Git integration (project-relative indexing)
- Session memory storage
- Hybrid search (BM25 + vector)
- Rayon integration for parallelism

**Tools**: 6 (+git_index, +session_recall)
**Tests**: 500+ (target)

### Phase 3: v0.3.0 - Analysis Core

**Focus**: Complexity, TDG, SATD

Deliverables:

- Port PMAT complexity analyzer
- Port TDG scorer
- Port SATD detector
- Analysis adapters in mcb-providers

**Tools**: 9 (+validate (action=analyze), +tdg_score, +satd_detect)
**Tests**: 1500+ (includes PMAT tests)

### Phase 4: v0.4.0 - Extended Analysis

**Focus**: Advanced code analysis

Deliverables:

- Dead code detection
- Dependency analysis (DAG)
- Big-O complexity estimation
- Code smell detection

**Tools**: 14 (+dead_code, +dependency_graph, +big_o, +code_smells, +debt_hotspots)
**Tests**: 3000+

### Phase 5: v0.5.0 - Quality + Git

**Focus**: Quality metrics and Git integration

Deliverables:

- Quality scoring
- File maintainability index
- Commit history analysis
- Impact analysis

**Tools**: 21 (+quality_score, +maintainability, +commit_analysis, +impact_analysis, +blame_analysis, +diff_analysis, +branch_compare)
**Tests**: 4500+

### Phase 6: v0.6.0+ - Advanced Features

**Focus**: Refactoring and mutations

Deliverables:

- Refactoring suggestions
- Mutation testing integration
- TUI dashboard
- Codebase scaffolding

**Tools**: 25+
**Tests**: 5390+

## Migration Principles

1. **No Breaking Changes**: Each version is backward compatible
2. **Feature Flags**: New features behind Cargo features
3. **Incremental Testing**: Port PMAT tests alongside code
4. **Parallel Development**: MCB features continue during integration

### Consequences

Positive:

- Controlled risk
- Continuous delivery
- Clear milestones
- Testable increments

Negative:

- Longer timeline
- Multiple integration points

Mitigation:

- Automate integration testing
- Feature flags for partial adoption
- Clear documentation per version

## Related ADRs

- [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - DI foundation
- [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Crate structure
- [ADR-016: Integration Points Adapter Pattern](016-integration-points-adapter-pattern.md) - PMAT integration
- [ADR-020: Testing Strategy Integration](020-testing-strategy-integration.md) - Test migration

---

Updated 2026-01-17 - Reflects v0.1.2 release status
