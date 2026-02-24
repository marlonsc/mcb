<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 41
title: Integrated Context System Architecture v0.4.0
status: PROPOSED
created:
updated: 2026-02-05
related: []
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR-041: Integrated Context System Architecture v0.4.0

> **v0.3.0 Note**: `mcb-application` crate was removed. Use cases moved to `mcb-infrastructure::di::modules::use_cases`.


**Status**: Proposed
**Date**: 2026-02-05
**Deciders**: MCB Architecture Team
**Related**: ADR-034, ADR-035, ADR-036, ADR-037 (Workflow series)
**Series**: ADR-041 → ADR-042 → ADR-043 → ADR-044 → ADR-045 → ADR-046

## Context

MCB v0.2.0 implements semantic code search with git awareness and persistent memory. v0.3.0 rebuilds the platform on SeaQL + Loco.rs (ADR-049-052). v0.4.0 adds workflow orchestration (ADR-034-037: FSM, context discovery, policies, compensation). v0.5.0 must unify these into an**integrated context system** that combines:

- VCS data (git history, branches, commits)
- Code indexing (AST chunks, relationships)
- Session history (workflow state + transitions)
- Chat memories (observations + tags + session context)
- Project hierarchy (plans, tasks, scopes from Beads)
- Policies (context boundaries, access control)

into a**single queryable knowledge base** with explicit freshness tracking, versioning, and search.

Problem Statement:

1. **No unified context**: Code search, git history, memory, and workflow state are separate systems. Queries cannot reason across all information sources.
2. **No freshness guarantees**: Caches expire, git state changes, but consumers don't know context age or staleness.
3. **No versioning**: Context snapshots lost between sessions. Cannot time-travel to "how did code look at 14:30?"
4. **No semantic relationships**: Code chunks are independent; no graph of dependencies, calls, data flows.
5. **Scope boundary enforcement**: No way to isolate context by task/scope/policy without manual filtering.

## Decision

### 1. Five-Layer Architecture

```text
┌─────────────────────────────────────────┐
│  Layer 5: Policies & FSM Gating        │  (ADR-034-036)
│  (FSM state gates freshness requirements)
├─────────────────────────────────────────┤
│  Layer 4: Hybrid Search & Discovery    │  (ADR-043)
│  (FTS + vectors + graph traversal + RRF fusion)
├─────────────────────────────────────────┤
│  Layer 3: Knowledge Graph              │  (ADR-042)
│  (Relationships, dependencies, call graphs)
├─────────────────────────────────────────┤
│  Layer 2: Versioned Context            │  (ADR-045)
│  (Immutable snapshots, temporal queries)
├─────────────────────────────────────────┤
│  Layer 1: Data Sources                 │  (VCS + Memory + Indexing)
│  (Git, observations, code chunks)
└─────────────────────────────────────────┘
```

Rationale:

- Clear separation of concerns (data → graph → search → policies)
- Each layer has well-defined ports/interfaces
- Can be implemented, tested, deployed independently
- Follows existing MCB Clean Architecture (ADR-013)

### 2. Core Data Model

```rust
// mcb-domain/src/entities/context.rs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub id: ContextId,
    pub project_id: ProjectId,
    pub worktree_id: WorktreeId,
    pub timestamp: SystemTime,
    pub workflow_state: WorkflowState,  // From ADR-034
    pub freshness: ContextFreshness,    // From ADR-035
    pub graph: Arc<CodeGraph>,          // From ADR-042
    pub memory_state: MemorySnapshot,   // Observations at this time
    pub vcs_state: VcsSnapshot,         // Git state at this time
    pub scope: ScopeFilter,             // Task/project/crate level
    pub version: u64,                   // Monotonic for CAS operations
}

#[derive(Clone, Debug)]
pub enum ContextFreshness {
    Fresh,              // < 5s old
    Acceptable,         // 5-30s old
    Stale,             // > 30s old
    StaleWithRisk,     // Uncommitted changes or git hook stale
}

#[derive(Clone, Debug)]
pub struct ScopeFilter {
    pub project_id: String,
    pub crate_path: Option<String>,
    pub module_path: Option<String>,
    pub file_path: Option<String>,
}
```

### 3. Port Interfaces (mcb-domain/ports/)

```rust
// ContextRepository trait (source of truth for snapshots)
#[async_trait]
pub trait ContextRepository: Send + Sync {
    async fn snapshot(&self, id: ContextId) -> Result<ContextSnapshot>;
    async fn create(&self, snapshot: ContextSnapshot) -> Result<ContextId>;
    async fn list_snapshots(&self, limit: u32, offset: u32) -> Result<Vec<ContextSnapshot>>;
    async fn timeline(
        &self,
        session_id: SessionId,
        start: SystemTime,
        end: SystemTime,
    ) -> Result<Vec<ContextSnapshot>>;
    async fn invalidate(&self, id: ContextId, reason: &str) -> Result<()>;
    async fn prune(&self, older_than: SystemTime) -> Result<u32>;  // Cleanup old snapshots
}

// ContextGraphTraversal trait (graph navigation and relationship queries)
#[async_trait]
pub trait ContextGraphTraversal: Send + Sync {
    async fn find_dependencies(&self, node_id: &str) -> Result<Vec<GraphNode>>;
    async fn find_dependents(&self, node_id: &str) -> Result<Vec<GraphNode>>;
    async fn traverse_path(&self, from: &str, to: &str) -> Result<Vec<GraphEdge>>;
    async fn find_cycles(&self) -> Result<Vec<Vec<GraphNode>>>;
}
```

**Note**: `ContextService` is NOT a port trait. It is a**concrete use case service** defined in `mcb-application/src/use_cases/context_service.rs` that composes all layers (repository, graph, search, policies). Per Clean Architecture, services are application-layer concerns, not domain ports.

### 4. Integration with ADR-034-037

| ADR | Integration Point | Interaction |
| ----- | ------------------- | ------------- |
| **ADR-034 (FSM)** | ContextSnapshot.workflow_state | State determines freshness requirements: "Executing" requires Fresh, "Planning" allows Stale |
| **ADR-035 (Context Scout)** | ContextSnapshot.freshness | Explicit freshness enum embedded in every snapshot; gates search results |
| **ADR-036 (Policies)** | ContextValidationResult | Policies define scope boundaries and access control for context |
| **ADR-037 (Orchestrator)** | Event broadcasting | Every snapshot creation emits WorkflowEvent; compensation triggers re-validation |

## Implementation

### Layer Breakdown

Layer 1: Data Sources

- Existing: Memory system, VCS provider, code indexing
- Changes: Add session_id FK, freshness tracking, last_modified timestamps

Layer 2: Versioning (ADR-045)

- New: ContextRepository storing immutable snapshots
- Technology: SQLite (primary) + im::Vector (in-memory cache) + serde for serialization

Layer 3: Knowledge Graph (ADR-042)

- New: CodeGraph built via tree-sitter-graph + petgraph
- Technology: petgraph DAG + daggy + slotmap

Layer 4: Search (ADR-043)

- New: HybridSearcher composing tantivy + vecstore + graph
- Technology: tantivy (FTS) + vecstore (HNSW vectors) + RRF fusion

Layer 5: Policies (ADR-046)

- Integration: ContextValidationResult checks policies
- Technology: Policy trait (existing from ADR-036)

### Crate Structure

```text
mcb-domain/
├── ports/
│   ├── context_repository.rs        [NEW] (ContextRepository trait)
│   └── context_graph_traversal.rs   [NEW] (ContextGraphTraversal trait)
└── entities/
    └── context.rs                   [NEW] (ContextSnapshot, ContextFreshness, etc.)

mcb-application/
├── use_cases/
│   └── context_service.rs           [NEW] (ContextService concrete service - composition of all layers)
└── ports/
    └── context_service_registry.rs  [NEW] (linkme slice for ContextService providers)

mcb-providers/
├── context/
│   ├── sqlite_context_repository.rs  [NEW]
│   ├── graph_builder.rs              [NEW] (tree-sitter-graph integration)
│   ├── hybrid_searcher.rs            [NEW] (tantivy + vecstore + graph)
│   └── freshness_tracker.rs          [NEW]
└── lib.rs
    └── linkme registration: CONTEXT_SERVICE_PROVIDERS

mcb-infrastructure/
├── context_handles.rs           [NEW] (ContextRepositoryHandle + ContextServiceHandle)
└── di/
    └── bootstrap.rs             [MODIFY] (add context providers)

mcb-server/
└── handlers/
    └── context/   [NEW] (MCP tools: context_search, context_snapshot, etc.)
```

## Alternatives Considered

| Alternative | Pros | Cons | Decision |
| ------------- | ------ | ------ | ---------- |
| **External Graph DB (Neo4j)** | Powerful, mature, scalable | Operational overhead, licensing, network latency | ❌ Rejected: Embedded-first (v0.5.0 optional) |
| **Separate snapshot storage (S3/DuckDB)** | Scalable to 1M+ snapshots | Added complexity, latency | ❌ Rejected: SQLite sufficient for MVP |
| **ML-based context ranking (Candle)** | High-quality relevance | Training overhead, slow inference | ❌ Rejected for v0.4.0 (v0.5.0) |
| **Embedded vector DB (vecstore)** | Fast, no external services | New crate (needs validation) | ✅ Selected: Lightweight, supports hybrid search |
| **RRF fusion (vs learning-to-rank)** | Simple, no training, reproducible | May not reach max relevance | ✅ Selected for MVP: Good baseline |

## Testing Strategy

- **Unit tests** (30): ContextSnapshot creation, versioning invariants, scope filtering
- **Integration tests** (15): FSM + context flow, policy gating, compensation
- **Graph tests** (10): Semantic extraction, traversal, cycle detection
- **Search tests** (10): RRF fusion, freshness ranking, hybrid queries
- **Temporal tests** (5): Time-travel queries, TTL invalidation

**Target**: 70+ tests, 85%+ coverage on domain layer

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
| ------ | ------------- | -------- | ----------- |
| ADR-034-037 changes mid-Phase-9 | Low | Critical | Lock ADR-034-037 before Phase 9 Week 1 |
| Snapshot memory overhead (1000+ snapshots) | Low | Medium | Add TTL-based GC (keep 24h, archive older) |
| Cross-layer dependency bugs | Medium | High | Comprehensive integration tests; phase-based validation |
| Freshness staleness detection failure | Medium | High | Multiple staleness signals (time + git hook + tracker) |

### Success Criteria

- ✅ 5-layer architecture fully integrated
- ✅ 70+ tests with 85%+ coverage
- ✅ Time-travel queries working (get context at specific timestamp)
- ✅ Freshness propagating through search results
- ✅ Policies enforcing scope boundaries
- ✅ Compensation triggering context re-validation on failure

## Architecture Corrections

### Correction 1: ContextService Layer Placement (2026-02-06)

**Issue**: ContextService was initially shown as a port trait in `mcb-domain/ports/`, violating Clean Architecture principles. Services are application-layer concerns, not domain ports.

Resolution:

- **Removed**: `ContextService` trait from domain ports
- **Added**: `ContextGraphTraversal` trait to domain ports (graph navigation is a port concern)
- **Clarified**: `ContextService` is a concrete use case service in `mcb-application/src/use_cases/context_service.rs`
- **Kept**: `ContextRepository` and `ContextGraphTraversal` as domain port traits (correct per Clean Architecture)

**Rationale**: Per Clean Architecture, domain defines port traits (interfaces to external systems). Application layer contains concrete services that orchestrate use cases. ContextService composes multiple layers (repository, graph, search, policies) and belongs in application, not domain.

### Correction 2: ADR-035 Dependency Acknowledgment (2026-02-06)

**Issue**: ADR-041 did not explicitly acknowledge its dependency on ADR-035 (Context Scout) for freshness tracking.

Resolution:

- **Clarified**: Layer 2 (Versioned Context) embeds `ContextFreshness` enum from ADR-035
- **Documented**: ADR-045 (Context Versioning) extends ADR-035's freshness contract
- **Cross-reference**: See ADR-045 "ADR-035 Contract Assumptions" section for full dependency details

**Rationale**: Explicit dependency documentation prevents accidental modification of ADR-035 (ACCEPTED/locked) during Phase 9 implementation. ADR-041-046 build on top of ADR-035's freshness model; they do not replace it.

---

**Next**: ADR-042 (Knowledge Graph), ADR-043 (Hybrid Search), ADR-044 (Lightweight Models), ADR-045 (Versioning), ADR-046 (Policy Integration)
