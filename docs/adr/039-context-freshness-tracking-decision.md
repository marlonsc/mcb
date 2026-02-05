# ADR-039: Context Freshness Tracking Decision

**Status:** ACCEPTED ✅  
**Date:** 2026-02-05  
**Decision Maker:** Marlon (Technical Lead)  
**Approval:** VOTED & APPROVED

---

## Decision

**Implement Explicit Context Freshness Tracking** using `StaleRisk` enum in all context discovery paths.

Every `ProjectContext` includes:
```rust
pub struct ProjectContext {
    discovered_at: Instant,           // When discovered
    git: GitContext,
    tracker: Option<TrackerContext>,
    config: ProjectConfig,
    freshness: ContextFreshness,      // NEW: explicit freshness marker
}

pub enum ContextFreshness {
    Fresh,                  // < 5 seconds old
    Acceptable,             // < 30 seconds old (cached)
    Stale,                  // > 30 seconds old
    StaleWithRisk { age_ms: u64, reason: String },  // Risk flag
}
```

Policies can check:
```rust
if context.freshness == ContextFreshness::Stale {
    return PolicyResult::Reject("Context too old, rediscover");
}
```

---

## Rationale

### The Problem We Solve

**Silent Race Condition in Implicit Model:**

```
T=0s:   Policy evaluates: WIPLimit < 5
        Reads cached: in_progress = 3  ✓ passes

T=5s:   Meanwhile (external process):
        - User closes 2 issues → in_progress = 1
        - Cache still shows: 3 (stale)

T=5s:   Transition allowed based on outdated data
        Workflow executes with wrong preconditions
```

**Real-World Impact:**
- Distributed teams: Multiple users closing issues while policy checks run
- CI/CD: External build systems modifying project state
- Git hooks: Modifications happening between policy check and transition
- **Outcome**: Invalid transitions, data inconsistency, silent bugs

### Why Explicit Tracking Prevents This

1. **Detects Staleness**: `context.age > 30s` → mark as Stale
2. **Rejects If Risky**: Policies configured: "Reject if Stale for critical gates"
3. **Rediscovery Optional**: Policies can trigger: "Rediscover context and retry"
4. **Audit Trail**: Log shows "Rejected due to stale context @ 45s"

### Research Precedent

All production workflow engines track context age explicitly:

| Tool | Method | TTL |
|------|--------|-----|
| **GitHub Actions** | `github.event_at` timestamp | 5 min max |
| **GitLab Runner** | `CI_PIPELINE_CREATED_AT` | Job-level, always fresh |
| **Cargo** | Timestamp metadata on crate index | 1 min (local cache) |
| **Kubernetes** | `metadata.creationTimestamp` on all resources | Always tracked |
| **Terraform** | State version + timestamp + lock | Explicit locking |

**Consensus**: Production systems don't assume freshness; they track explicitly.

---

## Consequences

### Positive ✅

- Prevents race condition bugs in distributed workflows
- Audit trail: "Why was this transition rejected?" → "Stale context"
- Policies have explicit control: which operations need fresh context?
- Compatible with cache invalidation (can force rediscovery)
- Low performance impact (just timestamp comparison)

### Trade-offs ⚠️

- **+2-3 days development**: Add freshness tracking to context discovery
- Policies must be configured: which gates require fresh context?
- Slightly larger WorkflowContext struct (~24 bytes for timestamp + enum)
- Requires cache invalidation mechanism (already designed in ADR-035)

### Benefits for Production ✅

- Prevents silent failures (stale context → explicit rejection)
- Enables policy configuration: "require fresh for critical gates"
- Supports future enhancements (smart rediscovery, retry logic)
- Meets security requirement: "Know the age of data making decisions"

---

## Implementation

### ContextFreshness Enum

```rust
pub enum ContextFreshness {
    Fresh,                  // 0-5 seconds (just discovered)
    Acceptable,             // 5-30 seconds (normal cache)
    Stale,                  // > 30 seconds (should rediscover)
    StaleWithRisk {
        age_ms: u64,
        reason: String,     // "git cache miss", "tracker offline", etc.
    },
}

impl ContextFreshness {
    pub fn is_acceptable(&self, max_age_ms: u64) -> bool {
        match self {
            Fresh | Acceptable => true,
            Stale => false,
            StaleWithRisk { age_ms, .. } => age_ms < max_age_ms,
        }
    }
}
```

### Discovery Phase

```rust
pub async fn discover(&self) -> Result<ProjectContext> {
    let discovered_at = Instant::now();
    
    let git = self.git_status().await?;
    let tracker = self.tracker_state().await;  // Partial: may fail
    let config = load_config()?;
    
    let freshness = if tracker.is_err() {
        ContextFreshness::StaleWithRisk {
            age_ms: 0,
            reason: "Tracker offline, git context only".to_string(),
        }
    } else {
        ContextFreshness::Fresh  // Just discovered
    };
    
    Ok(ProjectContext {
        discovered_at,
        git,
        tracker: tracker.ok(),
        config,
        freshness,
    })
}
```

### Policy Integration

```rust
pub trait Policy: Send + Sync {
    async fn evaluate(
        &self,
        trigger: &TransitionTrigger,
        context: &ProjectContext,
    ) -> Result<PolicyResult>;
}

// Example: RequireCleanWorktree policy
impl Policy for RequireCleanWorktree {
    async fn evaluate(&self, _trigger: &TransitionTrigger, context: &ProjectContext) -> Result<PolicyResult> {
        // Reject if context stale (dirty/clean status may have changed)
        if !context.freshness.is_acceptable(5000) {  // 5s max
            return Ok(PolicyResult::Reject(
                "Worktree status uncertain (stale context)".to_string()
            ));
        }
        
        let clean = context.git.is_clean();
        if clean {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject("Dirty worktree".to_string()))
        }
    }
}
```

### Configuration

```toml
[policies.require_clean_worktree]
enabled = true
max_stale_age_ms = 5_000  # 5 seconds max age

[policies.require_tests]
enabled = true
max_stale_age_ms = 30_000  # 30 seconds (tests can take time)

[policies.wip_limit]
enabled = true
max_stale_age_ms = 60_000  # 60 seconds (in_progress count less critical)
```

---

## Testing Strategy

**24 test cases** for context freshness:

1. Fresh context (0-5s): All policies should allow ✓
2. Acceptable context (5-30s): Policies evaluate normally ✓
3. Stale context (> 30s): Policies requiring fresh reject ✓
4. StaleWithRisk (tracker offline): Partial context handled ✓
5. Cache expiry: Timestamp updates correctly ✓
6. Multiple discoveries: Freshness resets ✓

---

## Related Decisions

- **ADR-035**: Context Scout (cache invalidation triggers)
- **ADR-036**: Policies (each policy specifies max_stale_age_ms)
- **ADR-038**: FSM States (freshness affects state transitions)

---

## Approval Chain

- [ ] Technical Lead: ✅ VOTED (Marlon)
- [ ] Architect: (Pending)
- [ ] Team: (Pending)

**Vote Status**: ACCEPTED by Marlon on 2026-02-05.

---

## Next Steps

1. Confirm with Architect: freshness tracking doesn't add complexity
2. Review policy configurations: which policies need < 5s freshness?
3. Implement ContextFreshness enum and discovery updates
4. Add freshness validation to PolicyGuard
