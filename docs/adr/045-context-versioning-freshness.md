---
adr: 45
title: Context Versioning & Freshness Tracking
status: PROPOSED
created: 
updated: 2026-02-05
related: []
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

## ADR-045: Context Versioning & Freshness Tracking

**Status**: Proposed  
**Date**: 2026-02-05  
**Deciders**: MCB Architecture Team  
**Related**: ADR-041 (Context System), ADR-035 (Context Scout)  
**Predecessor**: ADR-035 defines ContextFreshness enum

## Problem

ADR-035 defines freshness as explicit metadata: Fresh / Acceptable / Stale / StaleWithRisk.

But questions remain:

1.  **How to capture context at a point in time?** Code changes, git state changes, but we need "context as it was at 14:30:00"
2.  **How to track changes?** When code is modified, which context becomes stale?
3.  **How to version?** Store snapshots or compute on-demand?
4.  **How to scale?** 1000+ snapshots in a day = memory pressure

**This ADR specifies versioning strategy and staleness propagation.**

## Decision

### 1. ContextSnapshot Entity Spec

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub id: ContextId,
    pub timestamp: SystemTime,
    pub graph: Arc<CodeGraph>,
    pub memory_state: MemorySnapshot,  // Observations at this time
    pub vcs_state: VcsSnapshot,        // Git state at this time
    pub workflow_state: WorkflowState,
    pub freshness: ContextFreshness,
    pub version: u64,                  // Monotonic for versioning
}
```

### 2. Immutable Snapshots with TTL

**Rationale**:

-   **im::Vector**: Provides copy-on-write semantics. New snapshots don't copy old history.
-   **TTL policy**: Automatic cleanup prevents unbounded growth (keep 24h, archive older)
-   **Immutable**: No mutation, prevents consistency bugs
-   **DashMap**: Lock-free staleness tracking (high throughput)

### 2. Staleness Computation

Staleness is computed from **multiple signals** (not just time):

```rust
#[derive(Clone, Debug)]
pub enum StalenessSignal {
    None,
    TimeOnly { age_seconds: u64 },
    GitHookTriggered { commit_hash: String },  // New commit detected
    ManualEdit { reason: String },             // User modified files
    IndexInvalidated { trigger: String },      // Re-indexing happened
}

pub struct StalenessComputer {
    time_thresholds: StalenessThresholds,
    vcs_provider: Arc<dyn VcsProvider>,
}

pub struct StalenessThresholds {
    pub fresh_max_age: Duration,        // < 5s = Fresh
    pub acceptable_max_age: Duration,   // < 30s = Acceptable
    pub stale_max_age: Duration,        // > 30s = Stale
}

impl StalenessComputer {
    pub async fn compute(
        &self,
        snapshot: &ContextSnapshot,
        signals: &StalenessSignal,
    ) -> Result<ContextFreshness> {
        let age = SystemTime::now()
            .duration_since(snapshot.timestamp)
            .unwrap_or_default();
        
        // Signal-based override
        match signals {
            StalenessSignal::GitHookTriggered { .. } => {
                // Immediate stale
                return Ok(ContextFreshness::StaleWithRisk);
            },
            StalenessSignal::ManualEdit { .. } => {
                return Ok(ContextFreshness::StaleWithRisk);
            },
            _ => {},
        }
        
        // Time-based staleness
        if age < self.time_thresholds.fresh_max_age {
            Ok(ContextFreshness::Fresh)
        } else if age < self.time_thresholds.acceptable_max_age {
            Ok(ContextFreshness::Acceptable)
        } else if age < self.time_thresholds.stale_max_age {
            Ok(ContextFreshness::Stale)
        } else {
            // Very old + check if code actually changed
            let has_changes = self.vcs_provider
                .has_changes_since(snapshot.vcs_state.commit_hash.as_str())
                .await?;
            
            if has_changes {
                Ok(ContextFreshness::StaleWithRisk)
            } else {
                Ok(ContextFreshness::Stale)
            }
        }
    }
}
```

### 3. Time-Travel API: Get Context at Specific Timestamp

```rust
#[async_trait]
pub trait TimelineQuery: Send + Sync {
    // Get context snapshot created at or before given timestamp
    async fn get_at(&self, timestamp: SystemTime) -> Result<ContextSnapshot>;
    
    // Get timeline of changes between two timestamps
    async fn between(
        &self,
        start: SystemTime,
        end: SystemTime,
    ) -> Result<TimelineChange>;
    
    // Get "closest" snapshot to a timestamp (within margin)
    async fn closest(&self, timestamp: SystemTime, margin: Duration) -> Result<ContextSnapshot>;
}

pub struct TimelineChange {
    pub from_snapshot: ContextSnapshot,
    pub to_snapshot: ContextSnapshot,
    pub graph_changes: GraphDelta,
    pub memory_changes: MemoryDelta,
    pub vcs_changes: VcsDelta,
    pub duration: Duration,
}

impl TimelineQuery for VersionedContextStore {
    async fn get_at(&self, timestamp: SystemTime) -> Result<ContextSnapshot> {
        let versions = self.versions.read().await;
        
        // Binary search for closest snapshot before timestamp
        let idx = versions.iter()
            .position(|s| s.timestamp <= timestamp)
            .ok_or(Error::NoSnapshotBefore)?;
        
        Ok(versions[idx].clone())
    }
    
    async fn between(
        &self,
        start: SystemTime,
        end: SystemTime,
    ) -> Result<TimelineChange> {
        let from = self.get_at(start).await?;
        let to = self.get_at(end).await?;
        
        TimelineChange {
            from_snapshot: from.clone(),
            to_snapshot: to.clone(),
            graph_changes: GraphDelta::compute(&from.graph, &to.graph),
            memory_changes: MemoryDelta::compute(&from.memory_state, &to.memory_state),
            vcs_changes: VcsDelta::compute(&from.vcs_state, &to.vcs_state),
            duration: end - start,
        }
    }
}

// Example query:
// "Get context as it was 2 minutes ago"
let now = SystemTime::now();
let two_min_ago = now - Duration::from_secs(120);
let old_context = store.get_at(two_min_ago).await?;

// "Show me what changed in the last 5 minutes"
let timeline = store.between(
    now - Duration::from_secs(300),
    now
).await?;
println!("Code changes: {} new functions", timeline.graph_changes.new_nodes);
println!("Memory changes: {} new observations", timeline.memory_changes.new_observations);
```

### 4. Integration with ADR-034-037

**Workflow FSM (ADR-034)**:

-   Each FSM state transition tagged with context snapshot
-   On "Execute" state entry: capture snapshot (for rollback if needed)
-   Compensation handler invalidates snapshots if rolled back

**Context Scout (ADR-035)**:

-   Freshness enum embedded in every snapshot
-   Staleness signals trigger context re-discovery

**Policies (ADR-036)**:

-   Policy evaluation results tied to snapshot (reproducible)
-   Historical policy compliance queries: "Was code compliant at 14:30?"

## ADR-035 Contract Assumptions

This section documents the contract between ADR-045 (Context Versioning) and ADR-035 (Context Scout), ensuring v0.4.0 freshness tracking extends (not replaces) ADR-035's design.

### ContextFreshness Entity (from ADR-035)

ADR-035 defines the `ContextFreshness` enum as the authoritative freshness indicator:

```rust
#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub enum ContextFreshness {
    Fresh,              // < 5s old
    Acceptable,         // 5-30s old
    Stale,             // > 30s old
    StaleWithRisk,     // Uncommitted changes or git hook stale
}
```

**ADR-045 Dependency**: Every `ContextSnapshot` embeds a `ContextFreshness` value. This enum is **not redefined** in ADR-045; it is **reused directly** from ADR-035.

### CachedContextScout TTL & Invalidation (from ADR-035)

ADR-035 specifies the `CachedContextScout` provider with:

-   **Default TTL**: 30 seconds (configurable)
-   **Invalidation strategy**: Time-based (TTL expiry) + signal-based (git hooks, manual edits)
-   **Cache layers**: Separate caches for git status, tracker state, and full context

**ADR-045 Extension**:

-   Snapshots are stored immutably with timestamps
-   TTL policy determines which snapshots are kept in-memory vs archived to disk
-   Staleness signals (from ADR-035) trigger context re-validation during snapshot creation

### v0.4.0 Freshness Tracking EXTENDS ADR-035

**What ADR-035 provides**:

-   Freshness enum (Fresh/Acceptable/Stale/StaleWithRisk)
-   CachedContextScout with TTL-based caching
-   Staleness signals (time, git hooks, manual edits)

**What ADR-045 adds**:

-   Immutable snapshot versioning (capture context at point-in-time)
-   Time-travel queries (get context as it was at 14:30)
-   Snapshot-level staleness tracking (not just cache invalidation)
-   TTL-based garbage collection (keep 24h, archive older)
-   Historical policy compliance queries

**Explicit Dependency**: v0.4.0 ContextVersioning **depends on** ADR-035 ContextFreshness. The freshness enum is embedded in every snapshot and used to gate search Result ranking and policy evaluation.

### Snapshot Lifecycle with Freshness

```rust
// When creating a snapshot:
let snapshot = ContextSnapshot {
    id: context_id,
    timestamp: SystemTime::now(),
    graph: Arc::new(code_graph),
    memory_state: memory_snapshot,
    vcs_state: vcs_snapshot,
    workflow_state: workflow_state,
    freshness: freshness_computer.compute(&signals).await?,  // From ADR-035 logic
    version: monotonic_version,
};

// Freshness determines:
// 1. Search result ranking (stale results demoted)
// 2. Policy evaluation (some policies require Fresh context)
// 3. Snapshot retention (StaleWithRisk snapshots archived sooner)
```

## Testing

-   **Unit tests** (8): Snapshot creation, TTL GC, time-travel queries
-   **Immutability tests** (3): No accidental mutations, thread-safe
-   **Staleness tests** (7): Time-based, signal-based, composite
-   **Time-travel tests** (5): Correctness of historical queries
-   **Performance tests** (4): Snapshot creation <10ms, GC <100ms

**Target**: 27+ tests, 85%+ coverage

## Success Criteria

-   ✅ Snapshot creation <10ms
-   ✅ Time-travel query <20ms for 1000+ snapshots
-   ✅ Memory usage < 100MB for 24h history (auto-GC)
-   ✅ Staleness signals working (time + git + manual)
-   ✅ Historical policy compliance queryable

## Architecture Corrections

### Correction 1: ADR-035 Contract Documentation (2026-02-06)

**Issue**: ADR-045 referenced ADR-035 ContextFreshness but did not document the contract or dependency relationship.

**Resolution**:

-   **Added**: "ADR-035 Contract Assumptions" section documenting:
    -   ContextFreshness entity definition and reuse
    -   CachedContextScout TTL and invalidation strategy
    -   How v0.4.0 freshness tracking EXTENDS (not replaces) ADR-035
    -   Explicit dependency: v0.4.0 ContextVersioning depends on ADR-035 ContextFreshness
    -   Snapshot lifecycle showing freshness integration

**Rationale**: Clear documentation of cross-ADR dependencies prevents implementation bugs and ensures ADR-035 (ACCEPTED/locked) is not accidentally modified during Phase 9 implementation.

---

**Depends on**: ADR-041 (context), ADR-035 (freshness enum)  
**Feeds**: ADR-046 (compensation + rollback uses snapshots)
