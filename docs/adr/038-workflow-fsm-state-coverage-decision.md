# ADR-038: Workflow FSM State Coverage Decision

**Status:** ACCEPTED ✅  
**Date:** 2026-02-05  
**Decision Maker:** Marlon (Technical Lead)  
**Approval:** VOTED & APPROVED

---

## Decision

**Implement 12-state Workflow FSM (Production-Ready model)** instead of 7-state MVP.

States:
```
Ready → Planning → Executing → Verifying → {
  ✓ Completed
  ✗ Failed
  ↩️ RolledBack
}

Plus Extended States:
⏸️ Suspended (pause workflow)
⏱️ Timeout (deadline exceeded)
🚫 Cancelled (user abort)
👻 Abandoned (orphaned session)
```

---

## Rationale

### Against MVP (7-state) Model

The initial recommendation was MVP 7-state for speed (2-3 weeks faster). However:

1. **No Pause/Resume in Production**: Cannot suspend workflows mid-execution
   - Real workflows need pause (e.g., awaiting approval, manual gate)
   - MVP would lack this critical feature until Phase 9+
   - **Cost**: Users stuck with forced completion or abandonment

2. **No Timeout Handling**: Extended workflows blocked indefinitely
   - Tests hanging, external APIs slow → workflow hangs
   - No signal that timeout occurred (just stuck)
   - **Cost**: Operational nightmare at scale

3. **No Cancellation Path**: Cannot abort workflows
   - User starts wrong workflow → stuck with no exit
   - **Cost**: Manual database cleanup required

4. **Phase 9 Would Duplicate Work**:
   - Refactoring transitions matrix: ~150 lines of `match` arms
   - Schema changes (new workflow_sessions fields)
   - Test matrix expansion (each new state = 4-5 new edge cases)
   - **Cost**: 1 week of rework in Phase 9

### For Production-Ready Model

1. **Ship Complete Feature**: All states in one go
   - Users get pause/resume, timeout handling, cancellation from day 1
   - No feature gap between v0.2 and v0.3
   
2. **Only +2 Weeks Additional**:
   - Adding 5 new states = ~150 LOC per transition matrix
   - Mostly mechanical (add enum variants, match arms, tests)
   - Clear precedent in existing ADR-034 design

3. **Better For Production**: Anticipate real-world workflows
   - GitHub Actions: has pause/timeout/cancel
   - CircleCI: has pause/timeout/cancel
   - Kubernetes Jobs: has pause/timeout/cancel
   - **Precedent**: Industry standard

4. **Test Coverage Improves**: More state combinations force better edge case testing
   - Suspended → Planning transitions
   - Timeout transitions from any state
   - Cancelled → cleanup paths
   - **Benefit**: Safer implementation overall

---

## Consequences

### Positive ✅

- Users can pause/resume workflows
- Timeout detection prevents hangs
- Users can cancel stuck workflows
- No Phase 9 rework needed
- Production-ready from v0.2.0 ship
- Industry-standard feature set

### Trade-offs ⚠️

- **+2 weeks development** (4-5 weeks total instead of 2-3)
- More complex transition matrix (12 states vs 7)
- More test cases (~60 vs 40 for state transitions)
- Schema slightly larger (workflow_sessions has suspend/timeout fields)

### No Impact ❌

- Performance unaffected (enum matching is zero-cost)
- User-facing API same (WorkflowState enum is internal)
- Configuration unchanged (states not user-configurable)

---

## Decision Point: When Were You Uncertain?

The MVP recommendation assumed:
- Speed to market > Feature completeness
- Phase 9 could add states without disruption
- 7-state model sufficient for v0.2

**Decided Against These Assumptions:**
- Disruption in Phase 9 is real (rework penalty)
- Production workflows need pause/timeout/cancel now
- +2 weeks is acceptable for production safety

---

## Implementation Notes

### Schema Changes

```sql
-- Add to workflow_sessions table:
suspended_at: DATETIME  -- When suspended (NULL if not suspended)
suspended_reason: TEXT  -- Why suspended
timeout_deadline: DATETIME  -- When workflow times out
cancelled_by: TEXT  -- Who cancelled (NULL if not cancelled)
cancelled_reason: TEXT  -- Why cancelled
```

### Transition Rules

From each state, allowed transitions:

```
Ready →        Planning, Cancelled, Abandoned
Planning →     Executing, Failed, Cancelled, Suspended, Abandoned
Executing →    Verifying, Failed, Suspended, Cancelled, Abandoned, Timeout
Verifying →    Completed, RolledBack, Failed, Suspended, Cancelled, Abandoned, Timeout
Failed →       RolledBack, Cancelled, Abandoned
RolledBack →   Cancelled, Abandoned
Suspended →    Planning, Executing, Cancelled, Abandoned
Timeout →      Failed, RolledBack, Cancelled, Abandoned
Cancelled →    (terminal)
Completed →    (terminal)
Abandoned →    (terminal)
```

### Event Emissions

Each transition emits event for subscribers:

```rust
enum WorkflowEvent {
    Ready, Planning, Executing, Verifying,
    Completed, Failed, RolledBack,
    Suspended(reason: String),
    Timeout(deadline: DateTime),
    Cancelled(by: String, reason: String),
    Abandoned,
}
```

---

## Testing Strategy

**60 test cases** (vs 40 for MVP):

- State transition matrix: 24 cases (all valid transitions)
- Invalid transitions: 12 cases (blocked paths)
- Suspension lifecycle: 8 cases (suspend/resume/cancel from Suspended)
- Timeout handling: 8 cases (timeout from any state)
- Recovery paths: 8 cases (RolledBack transitions)

Total: ~150 new assertions across mcb-application and mcb-providers tests.

---

## Related Decisions

- **ADR-034**: Workflow Core FSM (updated for 12 states)
- **ADR-039**: Context Freshness (explicit tracking)
- **ADR-040**: Policy Coverage (11+ policies)
- **ADR-041**: Error Recovery (full compensation)

---

## Approval Chain

- [ ] Technical Lead: ✅ VOTED (Marlon)
- [ ] Architect: (Pending)
- [ ] Product: (Pending)
- [ ] Team Leads: (Pending)

**Vote Status**: Awaiting team review. Initial decision: ACCEPTED by Marlon.

---

## Next Steps

1. Share with Architect for feedback
2. Architect confirms schema + transition rules sound
3. Product confirms feature set aligns with v0.2 vision
4. Begin Week 1 implementation with 12-state design
