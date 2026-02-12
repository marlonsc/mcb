# Phase 9 Roadmap: Integrated Context System (v0.4.0)

## Overview

**Phase 9** implements the Integrated Context System with knowledge graphs, freshness tracking, and time-travel queries. This is a 4-week execution plan (Feb 17 - Mar 16, 2026) with 70+ tests and 6 ADRs (ADR-041-046).

**Builds on**: Phase 8 (Workflow FSM, Freshness Policies, Compensation)

## Timeline

| Week | Dates | Focus | Deliverables |
| ------ | ------- | ------- | -------------- |
| 1 | Feb 17-23 | Context Architecture & Graph | ADR-041, ADR-042, CodeGraph implementation |
| 2 | Feb 24-Mar 2 | Hybrid Search & Versioning | ADR-043, ADR-044, HybridSearchEngine, ContextSnapshot |
| 3 | Mar 3-9 | Integration & Policies | ADR-045, ADR-046, MCP tools, policy enforcement |
| 4 | Mar 10-16 | Testing & Documentation | 70+ tests, docs, migration guide, release |

## Week 1: Context Architecture & Graph (Feb 17-23)

### Goals

- Define context system architecture (5 layers)
- Implement knowledge graph (petgraph-based)
- Create CodeGraph with relationships
- 15+ tests

### Tasks

**ADR-041: Context Architecture**

- [ ] Define 5-layer context system
- [ ] Document layer responsibilities
- [ ] Define component interfaces
- [ ] Create architecture diagrams
- [ ] Beads issue: ADR-041 implementation

**ADR-042: Knowledge Graph**

- [ ] Design graph structure (nodes, edges, metadata)
- [ ] Define relationship types (calls, imports, extends, implements)
- [ ] Design freshness metadata storage
- [ ] Create graph traversal algorithms
- [ ] Beads issue: ADR-042 implementation

**CodeGraph Implementation** (mcb-application)

- [ ] Create CodeGraph struct (petgraph-based)
- [ ] Implement node insertion (code entities)
- [ ] Implement edge insertion (relationships)
- [ ] Implement graph traversal (BFS, DFS)
- [ ] Add freshness metadata tracking
- [ ] 15+ unit tests

**Deliverables**:

- ADR-041 document
- ADR-042 document
- CodeGraph implementation (mcb-application/src/graph/)
- 15+ tests passing

### Beads Issues

```bash
bd create "ADR-041: Context Architecture" -t feature -p 1
bd create "ADR-042: Knowledge Graph" -t feature -p 1
bd create "Implement CodeGraph (petgraph)" -t task -p 1
bd create "Graph traversal algorithms" -t task -p 2
```

## Week 2: Hybrid Search & Versioning (Feb 24-Mar 2)

### Goals

- Implement hybrid search engine (semantic + keyword)
- Implement context snapshots and versioning
- Create temporal query support
- 20+ tests

### Tasks

**ADR-043: Hybrid Search**

- [ ] Design RRF (Reciprocal Rank Fusion) algorithm
- [ ] Define search modes (semantic, keyword, hybrid)
- [ ] Design freshness filtering
- [ ] Create search Result ranking
- [ ] Beads issue: ADR-043 implementation

**ADR-044: Model Selection**

- [ ] Evaluate embedding models (OpenAI, VoyageAI, Ollama)
- [ ] Evaluate search algorithms (BM25, TF-IDF, RRF)
- [ ] Document model choices and trade-offs
- [ ] Create model configuration
- [ ] Beads issue: ADR-044 implementation

**HybridSearchEngine Implementation** (mcb-application)

- [ ] Create HybridSearchEngine struct
- [ ] Implement semantic search (embedding-based)
- [ ] Implement keyword search (full-text index)
- [ ] Implement RRF fusion algorithm
- [ ] Implement freshness filtering
- [ ] 15+ unit tests

**ContextSnapshot Implementation** (mcb-application)

- [ ] Create ContextSnapshot struct
- [ ] Implement snapshot creation (at commit/date)
- [ ] Implement snapshot storage
- [ ] Implement snapshot retrieval
- [ ] Implement temporal queries
- [ ] 10+ unit tests

**Deliverables**:

- ADR-043 document
- ADR-044 document
- HybridSearchEngine implementation (mcb-application/src/search/)
- ContextSnapshot implementation (mcb-application/src/snapshot/)
- 25+ tests passing

### Beads Issues

```bash
bd create "ADR-043: Hybrid Search Engine" -t feature -p 1
bd create "ADR-044: Model Selection" -t feature -p 1
bd create "Implement HybridSearchEngine" -t task -p 1
bd create "Implement ContextSnapshot" -t task -p 1
bd create "RRF fusion algorithm" -t task -p 2
```

## Week 3: Integration & Policies (Mar 3-9)

### Goals

- Integrate context system with workflow FSM
- Implement policy enforcement
- Create MCP tools for context operations
- 20+ tests

### Tasks

**ADR-045: Context Versioning**

- [ ] Design snapshot versioning scheme
- [ ] Design temporal query language
- [ ] Design snapshot retention policies
- [ ] Create version comparison algorithms
- [ ] Beads issue: ADR-045 implementation

**ADR-046: Integration Patterns**

- [ ] Design MCP tool integration
- [ ] Design policy enforcement hooks
- [ ] Design compensation triggers
- [ ] Design event system
- [ ] Beads issue: ADR-046 implementation

**MCP Tool Implementation** (mcb-server)

- [ ] Implement `search` tool (with freshness, snapshots, policies)
- [ ] Implement `index` tool (with snapshot creation)
- [ ] Implement `memory` tool (context storage)
- [ ] Implement `session` tool (workflow sessions)
- [ ] 15+ integration tests

**Policy Enforcement** (mcb-application)

- [ ] Create PolicyEngine struct
- [ ] Implement freshness policy enforcement
- [ ] Implement validation policies
- [ ] Implement compensation triggers
- [ ] Integrate with workflow FSM
- [ ] 10+ unit tests

**Deliverables**:

- ADR-045 document
- ADR-046 document
- MCP tool implementations (mcb-server/src/handlers/)
- PolicyEngine implementation (mcb-application/src/policy/)
- 25+ tests passing

### Beads Issues

```bash
bd create "ADR-045: Context Versioning" -t feature -p 1
bd create "ADR-046: Integration Patterns" -t feature -p 1
bd create "Implement MCP tools (search, index, memory)" -t task -p 1
bd create "Implement PolicyEngine" -t task -p 1
bd create "FSM + context system integration" -t task -p 1
```

## Week 4: Testing & Documentation (Mar 10-16)

### Goals

- Achieve 70+ tests across all components
- Complete documentation and migration guide
- Release v0.4.0
- 15+ tests

### Tasks

**Testing**

- [ ] Unit tests for CodeGraph (15+)
- [ ] Unit tests for HybridSearchEngine (15+)
- [ ] Unit tests for ContextSnapshot (10+)
- [ ] Unit tests for PolicyEngine (10+)
- [ ] Integration tests for MCP tools (15+)
- [ ] End-to-end tests (5+)
- [ ] Total: 70+ tests

**Documentation**

- [ ] Complete ADR-041-046 documents
- [ ] Create migration guide (v0.3 → v0.4.0)
- [ ] Create feature guide (integrated-context.md)
- [ ] Create architecture documentation
- [ ] Update ROADMAP.md
- [ ] Update CHANGELOG.md
- [ ] Update README.md

**Release**

- [ ] Run full test suite (`make test`)
- [ ] Run quality gates (`make check`)
- [ ] Run architecture validation (`make validate`)
- [ ] Create release notes
- [ ] Tag v0.4.0
- [ ] Push to remote

**Deliverables**:

- 70+ tests passing
- Complete documentation
- v0.4.0 release
- Migration guide
- Feature guides

### Beads Issues

```bash
bd create "Write 70+ tests for Phase 9" -t task -p 1
bd create "Complete documentation (ADRs, guides, migration)" -t task -p 1
bd create "Release v0.4.0" -t task -p 1
```

## Test Coverage

### Unit Tests (50+)

**CodeGraph** (15+):

- Node insertion and retrieval
- Edge insertion and retrieval
- Relationship type handling
- Freshness metadata tracking
- Graph traversal (BFS, DFS)
- Cycle detection
- Path finding

**HybridSearchEngine** (15+):

- Semantic search
- Keyword search
- RRF fusion
- Freshness filtering
- Result ranking
- Edge cases (empty results, single Result)

**ContextSnapshot** (10+):

- Snapshot creation
- Snapshot storage and retrieval
- Temporal queries
- Version comparison
- Snapshot retention

**PolicyEngine** (10+):

- Freshness policy enforcement
- Validation policy enforcement
- Compensation triggers
- Policy composition

### Integration Tests (15+)

**MCP Tools**:

- `search` with freshness filtering
- `search` with snapshots
- `search` with policies
- `index` with snapshot creation
- `memory` context storage
- `session` workflow integration

**FSM Integration**:

- Context gates in workflow transitions
- Policy enforcement at state boundaries
- Compensation triggers

### End-to-End Tests (5+)

**Workflows**:

- Freshness-aware search workflow
- Time-travel query workflow
- Policy-driven context discovery
- Compensation and rollback workflow

## Success Criteria

- [ ] All 70+ tests passing
- [ ] Zero architecture violations (`make validate`)
- [ ] Clean lint (`make lint`)
- [ ] Clean markdown lint (`make docs-lint`)
- [ ] ADR-041-046 complete and linked
- [ ] Migration guide complete
- [ ] Feature guides complete
- [ ] v0.4.0 released and tagged

## Related Documentation

- **ADR-034**: Workflow FSM – Foundation for context workflows
- **ADR-035**: Freshness Tracking – Temporal metadata
- **ADR-036**: Policies & Validation – Policy framework
- **ADR-037**: Compensation & Orchestration – Rollback patterns
- **ADR-041-046**: Phase 9 implementation details
- [`docs/guides/features/INTEGRATED_CONTEXT.md`](../guides/features/INTEGRATED_CONTEXT.md) – Feature overview
- [`docs/migration/v0.3-to-v0.4.md`](../migration/v0.3-to-v0.4.md) – Migration guide
- [`docs/architecture/CLEAN_ARCHITECTURE.md`](../architecture/CLEAN_ARCHITECTURE.md) – Architecture patterns

## Next Steps

1. Review ADR-034-037 (Phase 8 foundation)
2. Review ADR-041-046 (Phase 9 design)
3. Create Beads issues for each task
4. Start Week 1 (Feb 17)
5. Track progress weekly
