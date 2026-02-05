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

# ADR-045: Context Versioning & Freshness Tracking

**Status**: Proposed  
**Date**: 2026-02-05  
**Deciders**: MCB Architecture Team  
**Related**: ADR-041 (Context System), ADR-035 (Context Scout)  
**Predecessor**: ADR-035 defines ContextFreshness enum

## Problem

ADR-035 defines freshness as explicit metadata: Fresh / Acceptable / Stale / StaleWithRisk.

But questions remain:

1. **How to capture context at a point in time?** Code changes, git state changes, but we need "context as it was at 14:30:00"
2. **How to track changes?** When code is modified, which context becomes stale?
3. **How to version?** Store snapshots or compute on-demand?
4. **How to scale?** 1000+ snapshots in a day = memory pressure

**This ADR specifies versioning strategy and staleness propagation.**

## Decision

### 1. Immutable Snapshots with TTL

```rust
use im::Vector;

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

pub struct VersionedContextStore {
    // Immutable history (using im::Vector for COW semantics)
    versions: Arc<RwLock<im::Vector<ContextSnapshot>>>,
    
    // Staleness tracking
    staleness_map: Arc<DashMap<ContextId, StalenessInfo>>,
    
    // TTL-based garbage collection
    ttl_policy: TtlPolicy,
}

pub struct TtlPolicy {
    pub keep_recent: Duration,      // Keep snapshots from last 24h
    pub keep_count: u32,             // Keep at least 10 snapshots
    pub archive_older: bool,         // Move 24h+ old snapshots to disk
}

impl VersionedContextStore {
    pub async fn snapshot(&self, id: ContextId) -> Result<ContextSnapshot> {
        let versions = self.versions.read().await;
        versions.iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or(Error::SnapshotNotFound)
    }
    
    pub async fn create(&self, snapshot: ContextSnapshot) -> Result<ContextId> {
        let mut versions = self.versions.write().await;
        let id = snapshot.id.clone();
        
        // Immutable append (im::Vector handles COW)
        versions.push_back(snapshot.clone());
        
        // Track staleness info
        self.staleness_map.insert(id.clone(), StalenessInfo {
            created_at: SystemTime::now(),
            last_validated: SystemTime::now(),
            stale_signal: StalenessSignal::None,
        });
        
        // Trigger GC if needed
        self.garbage_collect().await?;
        
        Ok(id)
    }
    
    pub async fn timeline(
        &self,
        session_id: &str,
        start: SystemTime,
        end: SystemTime,
    ) -> Result<Vec<ContextSnapshot>> {
        let versions = self.versions.read().await;
        Ok(versions.iter()
            .filter(|s| s.timestamp >= start && s.timestamp <= end)
            .cloned()
            .collect())
    }
    
    async fn garbage_collect(&self) -> Result<()> {
        let mut versions = self.versions.write().await;
        let now = SystemTime::now();
        
        // Keep snapshots newer than TTL policy
        let keep_cutoff = now - self.ttl_policy.keep_recent;
        let to_archive: Vec<_> = versions.iter()
            .filter(|s| s.timestamp < keep_cutoff)
            .cloned()
            .collect();
        
        // Archive or delete based on policy
        for snapshot in to_archive {
            if self.ttl_policy.archive_older {
                self.archive_snapshot(&snapshot).await?;
            }
            versions.retain(|s| s.id != snapshot.id);
        }
        
        Ok(())
    }
}
```

**Rationale**:
- **im::Vector**: Provides copy-on-write semantics. New snapshots don't copy old history.
- **TTL policy**: Automatic cleanup prevents unbounded growth (keep 24h, archive older)
- **Immutable**: No mutation, prevents consistency bugs
- **DashMap**: Lock-free staleness tracking (high throughput)

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
- Each FSM state transition tagged with context snapshot
- On "Execute" state entry: capture snapshot (for rollback if needed)
- Compensation handler invalidates snapshots if rolled back

**Context Scout (ADR-035)**:
- Freshness enum embedded in every snapshot
- Staleness signals trigger context re-discovery

**Policies (ADR-036)**:
- Policy evaluation results tied to snapshot (reproducible)
- Historical policy compliance queries: "Was code compliant at 14:30?"

## Testing

- **Unit tests** (8): Snapshot creation, TTL GC, time-travel queries
- **Immutability tests** (3): No accidental mutations, thread-safe
- **Staleness tests** (7): Time-based, signal-based, composite
- **Time-travel tests** (5): Correctness of historical queries
- **Performance tests** (4): Snapshot creation <10ms, GC <100ms

**Target**: 27+ tests, 85%+ coverage

## Success Criteria

- ✅ Snapshot creation <10ms
- ✅ Time-travel query <20ms for 1000+ snapshots
- ✅ Memory usage < 100MB for 24h history (auto-GC)
- ✅ Staleness signals working (time + git + manual)
- ✅ Historical policy compliance queryable

---

**Depends on**: ADR-041 (context), ADR-035 (freshness enum)  
**Feeds**: ADR-046 (compensation + rollback uses snapshots)
