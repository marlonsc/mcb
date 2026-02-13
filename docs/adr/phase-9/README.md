# Phase 9: Integrated Context System (ADR-041-046)

## Overview

Phase 9 implements the**Integrated Context System** for v0.4.0, building on Phase 8's workflow FSM and policy framework. This phase introduces knowledge graphs, freshness tracking, time-travel queries, and hybrid search capabilities.

**Timeline**: Feb 17 - Mar 16, 2026 (4 weeks)
**ADRs**: ADR-041 through ADR-046 (6 decisions)
**Tests**: 70+ (unit, integration, end-to-end)
**Deliverables**: Context architecture, graph implementation, hybrid search, versioning, MCP integration

## ADRs

### ADR-041: Context Architecture

**Status**: Proposed
**Date**: Feb 2026
**Scope**: System design for integrated context system

**Summary**:
Defines the 5-layer context system architecture:

1. Code Indexing & Embeddings (AST parsing, vector embeddings)
2. Knowledge Graph (code relationships, freshness metadata)
3. Hybrid Search Engine (semantic + keyword search, RRF fusion)
4. Versioning & Snapshots (temporal queries, snapshot management)
5. Integration & Policies (policy enforcement, compensation triggers)

Key Decisions:

- 5-layer architecture with clear separation of concerns
- Knowledge graph as central component (petgraph-based)
- Hybrid search combining semantic and keyword approaches
- Snapshot-based versioning for time-travel queries
- Policy-driven context discovery

**Related**: ADR-042, ADR-043, ADR-044, ADR-045, ADR-046

**See**: [`docs/architecture/ARCHITECTURE.md`](../../architecture/ARCHITECTURE.md) for detailed architecture

---

### ADR-042: Knowledge Graph

**Status**: Proposed
**Date**: Feb 2026
**Scope**: Graph structure and relationship modeling

**Summary**:
Defines the knowledge graph structure for representing code relationships:

Graph Components:

- **Nodes**: Code entities (functions, classes, modules, types)
- **Edges**: Relationships (calls, imports, extends, implements, uses)
- **Metadata**: Freshness (last modified, staleness signals), version, source

Relationship Types:

- `calls`: Function A calls Function B
- `imports`: Module A imports Module B
- `extends`: Class A extends Class B
- `implements`: Class A implements Interface B
- `uses`: Code A uses Type B
- `defines`: Module A defines Function B

Freshness Metadata:

- Last modified timestamp
- Commit hash
- Branch information
- Staleness signals (deprecated, outdated, legacy)

Graph Operations:

- Node insertion/retrieval
- Edge insertion/retrieval
- Traversal (BFS, DFS, shortest path)
- Cycle detection
- Relationship queries

**Related**: ADR-041, ADR-035 (Freshness Tracking)

**See**: [`docs/guides/features/INTEGRATED_CONTEXT.md`](../../guides/features/INTEGRATED_CONTEXT.md) for usage examples

---

### ADR-043: Hybrid Search Engine

**Status**: Proposed
**Date**: Feb 2026
**Scope**: Search algorithm design and implementation

**Summary**:
Defines the hybrid search engine combining semantic and keyword search:

Search Modes:

- **Semantic**: Embedding-based similarity search
- **Keyword**: Full-text index (BM25, TF-IDF)
- **Hybrid**: RRF (Reciprocal Rank Fusion) combining both

RRF Algorithm:

```text
score(d) = Σ 1 / (k + rank(d))
```

Where k=60 (constant), rank is position in each ranking

Freshness Filtering:

- Filter results by max age (days)
- Apply staleness signals (deprecated, outdated, legacy)
- Rank by freshness score

Result Ranking:

1. RRF score (semantic + keyword)
2. Freshness score (recency + staleness)
3. Relevance score (combined)

**Related**: ADR-041, ADR-044 (Model Selection)

**See**: [`docs/migration/v0.3-to-v0.4.md`](../../migration/v0.3-to-v0.4.md) for usage examples

---

### ADR-044: Model Selection

**Status**: Proposed
**Date**: Feb 2026
**Scope**: Embedding and search model choices

**Summary**:
Evaluates and selects embedding and search models for v0.4.0:

Embedding Models:

- **OpenAI** (text-embedding-3-small): High quality, cost-effective
- **VoyageAI** (voyage-2): Specialized for code, high quality
- **Ollama** (nomic-embed-text): Local, privacy-preserving
- **Gemini** (embedding-001): Google's model, good quality
- **FastEmbed** (BAAI/bge-small-en-v1.5): Fast, local, good quality

Search Algorithms:

- **BM25**: Keyword search baseline
- **TF-IDF**: Term frequency-inverse document frequency
- **RRF**: Reciprocal Rank Fusion for hybrid search

Trade-offs:

- Quality vs. Cost (OpenAI vs. Ollama)
- Speed vs. Accuracy (FastEmbed vs. VoyageAI)
- Privacy vs. Features (Ollama vs. OpenAI)

Recommendations:

- Default: OpenAI (best quality/cost balance)
- Privacy-first: Ollama (local, no API calls)
- Code-specialized: VoyageAI (best for code)

**Related**: ADR-041, ADR-043 (Hybrid Search)

---

### ADR-045: Context Versioning

**Status**: Proposed
**Date**: Feb 2026
**Scope**: Snapshot and temporal query design

**Summary**:
Defines snapshot-based versioning for time-travel queries:

Snapshot Structure:

- Immutable capture of code state at specific commit/date
- Includes: graph, embeddings, metadata, freshness info
- Identified by: commit hash, tag, date

Snapshot Operations:

- Create: Capture current state
- Retrieve: Load snapshot by commit/tag/date
- Compare: Diff between snapshots
- Cleanup: Retention policies (N commits, N days)

Temporal Queries:

- "Show code as it was at v0.2.0"
- "How did this function evolve?"
- "When was this pattern introduced?"

Retention Policies:

- Keep last N commits (e.g., 100)
- Keep last N days (e.g., 90)
- Snapshot frequency (e.g., every 10 commits)

**Related**: ADR-041, ADR-042 (Knowledge Graph)

**See**: [`docs/guides/features/INTEGRATED_CONTEXT.md`](../../guides/features/INTEGRATED_CONTEXT.md) for time-travel examples

---

### ADR-046: Integration with ADR-034-037 & Policies

**Status**: Proposed
**Date**: Feb 2026
**Scope**: MCP tool integration and policy enforcement

**Summary**:
Defines integration patterns for context system with MCP tools and workflow FSM:

MCP Tools:

- `search`: Semantic search with freshness, snapshots, policies
- `index`: Indexing operations with snapshot creation
- `memory`: Context storage and retrieval
- `session`: Workflow session management
- `agent`: Agent activity logging
- `project`: Project workflow operations
- `vcs`: Repository operations

Tool Parameters:

```text
search:
  --query: Search query
  --freshness-max-age: Max age in days
  --snapshot: Snapshot version (commit/tag/date)
  --policy: Policy name
  --mode: Search mode (semantic, keyword, hybrid)

index:
  --start: Start indexing
  --status: Get status
  --clear: Clear index
  --snapshot-create: Create snapshot
```

Policy Enforcement:

- Freshness gates: Block if context too old
- Validation gates: Block if policy violated
- Compensation hooks: Trigger refresh/rollback

FSM Integration:

- Context gates at state transitions
- Policy enforcement at boundaries
- Compensation triggers on violations

Event System:

- Context updated event
- Policy violation event
- Compensation triggered event
- Snapshot created event

**Related**: ADR-041, ADR-036 (Policies), ADR-037 (Compensation)

---

## Cross-References

### Phase 8 Foundation (ADR-034-037)

Phase 9 builds on Phase 8's workflow system:

- **ADR-034** (Workflow FSM): Provides state machine for context workflows
- **ADR-035** (Freshness Tracking): Provides temporal metadata foundation
- **ADR-036** (Policies & Validation): Provides policy framework
- **ADR-037** (Compensation & Orchestration): Provides rollback patterns

### Related ADRs

- **ADR-001**: Modular Crates Architecture
- **ADR-002**: Async-First Architecture
- **ADR-003**: Unified Provider Architecture
- **ADR-013**: Clean Architecture Crate Separation
- **ADR-023**: Inventory to Linkme Migration
- **ADR-029**: Hexagonal Architecture with dill

## Implementation Roadmap

See [`docs/implementation/phase-9-roadmap.md`](../../implementation/phase-9-roadmap.md) for detailed 4-week execution plan:

- **Week 1** (Feb 17-23): Context Architecture & Graph
- **Week 2** (Feb 24-Mar 2): Hybrid Search & Versioning
- **Week 3** (Mar 3-9): Integration & Policies
- **Week 4** (Mar 10-16): Testing & Documentation

## Feature Guides

- [`docs/guides/features/INTEGRATED_CONTEXT.md`](../../guides/features/INTEGRATED_CONTEXT.md) – Feature overview and workflows
- [`docs/migration/v0.3-to-v0.4.md`](../../migration/v0.3-to-v0.4.md) – Migration guide from v0.3
- [`docs/architecture/CLEAN_ARCHITECTURE.md`](../../architecture/CLEAN_ARCHITECTURE.md) – Architecture patterns

## Testing

**Target**: 70+ tests across all components

- CodeGraph: 15+ tests
- HybridSearchEngine: 15+ tests
- ContextSnapshot: 10+ tests
- PolicyEngine: 10+ tests
- MCP Tools: 15+ integration tests
- End-to-end: 5+ tests

### Success Criteria

- [ ] All ADR-041-046 complete
- [ ] 70+ tests passing
- [ ] Zero architecture violations
- [ ] Clean lint and docs-lint
- [ ] Migration guide complete
- [ ] Feature guides complete
- [ ] v0.4.0 released

## Next Steps

1. Review ADR-034-037 (Phase 8 foundation)
2. Review ADR-041-046 (Phase 9 design)
3. Create Beads issues for each ADR
4. Start Week 1 implementation (Feb 17)
5. Track progress weekly
