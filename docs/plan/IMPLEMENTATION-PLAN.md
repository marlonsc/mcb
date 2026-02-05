# MCB Workflow v0.2.0 — Unified Implementation Plan

**Document**: Single source of truth for v0.2.0 workflow implementation  
**Version**: 1.0  
**Last Updated**: 2025-01-21  
**Status**: READY FOR TEAM REVIEW

---

## 1. Executive Summary

### Vision: "Software Factory" Orchestration

MCB v0.2.0 transitions from simple workflow automation to a **multi-tier Software Factory** that orchestrates projects, plans, tasks, sessions, and agents with policy enforcement and session persistence.

**Core Principle**: Treat software delivery as a factory where work flows through defined stages, policies are enforced at each gate, and the entire execution history is queryable and replayable.

### Goals

1.  **Session Continuity**: Workflows persist across restarts; sessions queryable at any point in time
2.  **Multi-Tier Execution**: Concurrent projects/plans/tasks (WIP-limited), sequential operator execution
3.  **Policy Enforcement**: 11 policies guard transitions; operator can override with audit trail
4.  **Compensation & Rollback**: Failed transitions revert to safe state via git branches
5.  **Event Broadcasting**: 3-channel system (Message Queue + Database + Webhooks) for external integration
6.  **Beads Integration**: Task relationships managed by Beads; workflow execution managed by MCB (no duplication)
7.  **Developer Experience**: MCP workflow tool with intuitive action-based handlers

### 9 Locked Architectural Decisions

These decisions are FINAL unless rejected in Week 0 team review:

1.  **ADR-034**: Workflow Session FSM with append-only event log
2.  **ADR-035**: VCS abstraction with git2 + worktrees (Phase 2: GitHub/GitLab)
3.  **ADR-036**: Policy enforcement at 5 lifecycle points (11 policies total)
4.  **ADR-037**: Event broadcasting on 3 channels (Queue + DB + Webhooks)
5.  **ADR-038**: Hybrid transaction model (per-operation + event log)
6.  **Architecture**: 5-tier entity model (Project → Plan → Task → Session → Operator/Agent)
7.  **Database**: SQLite MVP with WAL mode (Phase 2: PostgreSQL Option)
8.  **Beads**: Source of truth for task relationships; MCB = execution layer only
9.  **Performance**: Benchmark-driven optimization (Week 1 baseline, Week 4 hardening)

### Timeline: 4 Weeks, 150-170 Hours

| Week | Focus | Hours | Deliverables |
|------|-------|-------|--------------|
| **0** | Amendments | 14.5 | Final ADRs, team decision, branch setup |
| **1** | Foundation | 40 | Domain entities, ports, comprehensive tests |
| **2** | Providers | 40 | All implementations, integration tests |
| **3** | Integration | 40 | MCP handlers, Beads integration, benchmarks |
| **4** | Polish | 30 | Docs, E2E tests, release prep |
| **Total** | | **164.5** | v0.2.0 ready to release |

### Team & Capacity

-   **Lead Engineer**: Architecture, Week 0 + Week 4 (36.5 hours)
-   **Engineer A**: Domain + Providers (80 hours)
-   **Engineer B**: Providers + Services (80 hours)
-   **Total**: 3 engineers, ~55 hours/week, 4 weeks

---

## 2. Architectural Overview

### The 5-Tier Entity Model

```
┌──────────────────────────────────────────────────────────┐
│  MCB Workflow Architecture (ADR-034-038)                │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │ Project (Figment config)                       │    │
│  │   ├─ Metadata: name, repo, policies            │    │
│  │   └─ Multiple Plans (from Beads roadmap)       │    │
│  │                                                 │    │
│  │   ┌─────────────────────────────────────────┐  │    │
│  │   │ Plan (Release, Phase, Milestone)        │  │    │
│  │   │   ├─ Metadata: title, status            │  │    │
│  │   │   └─ Multiple Tasks (from Beads)        │  │    │
│  │   │                                          │  │    │
│  │   │   ┌──────────────────────────────────┐  │  │    │
│  │   │   │ Task (Beads issue)              │  │  │    │
│  │   │   │   ├─ Metadata: ID, title        │  │  │    │
│  │   │   │   └─ Multiple Sessions          │  │  │    │
│  │   │   │                                  │  │  │    │
│  │   │   │   ┌────────────────────────────┐ │  │  │    │
│  │   │   │   │ Session (FSM, Execution)   │ │  │  │    │
│  │   │   │   │   ├─ WorkflowState (FSM)   │ │  │  │    │
│  │   │   │   │   ├─ Git branch/worktree   │ │  │  │    │
│  │   │   │   │   ├─ Policies (guards)     │ │  │  │    │
│  │   │   │   │   ├─ Events (append-only)  │ │  │  │    │
│  │   │   │   │   └─ Operator/Agent queue  │ │  │  │    │
│  │   │   │   └────────────────────────────┘ │  │  │    │
│  │   │   └──────────────────────────────────┘  │  │    │
│  │   └─────────────────────────────────────────┘  │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
│  Execution Model:                                        │
│    • Concurrent: Projects, Plans, Tasks (WIP-limited)  │
│    • Sequential: Operator (one action at a time)       │
│    • Persistent: All state in SQLite + event log       │
│    • Replayable: Time-travel queries for any point     │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### State Machines

**TaskState** (from Beads):

```
open → in_progress → blocked → in_progress → closed
                   ↓
               cancelled (optional)
```

**WorkflowState** (FSM for Session execution):

```
initialized
    ↓
discovering_context
    ↓
validating_policies
    ↓
executing
    ↓
compensating (on failure)
    ↓
completed / failed
```

**Transitions guarded by 11 policies** across 5 lifecycle points (see ADR-036).

### Provider Abstraction

```rust
// Four core providers (trait-based, testable, swappable)
trait DatabaseProvider { ... }      // SQLite MVP → PostgreSQL Phase 2
trait VcsProvider { ... }           // git2 MVP → GitHub/GitLab Phase 2
trait ContextScoutProvider { ... }  // Discovery + caching
trait PolicyGuardProvider { ... }   // Policy evaluation + composition
```

### Event Broadcasting: 3-Channel Pattern

```
┌──────────────────────────────────┐
│  Workflow Event Emitted          │
└──────────────────────────────────┘
              │
    ┌─────────┼─────────┐
    ↓         ↓         ↓
┌────────┐┌────────┐┌────────┐
│ Queue  ││ SQLite ││Webhooks│
│(async) ││(sync)  ││(async) │
└────────┘└────────┘└────────┘
    ↓         ↓         ↓
[Internal] [History] [External]
```

All three channels fire for every event:

-   **Message Queue**: Internal async work (compensation, cleanup)
-   **SQLite**: Event log (persistence, queryability, replay)
-   **Webhooks**: External integration (CI/CD, Slack, monitoring)

---

## 3. The 9 Locked Architectural Decisions

### Decision 1: Database Abstraction

**Title**: DatabaseProvider Trait with SQLite MVP  
**Status**: PROPOSED (ADR-038)  
**Choice**: Implement `DatabaseProvider` trait with SQLite backend; PostgreSQL deferred to Phase 2

**Rationale**:

-   SQLite handles single-developer workflow at MCB scale (< 10,000 sessions/month)
-   WAL mode enables concurrent reads + sequential writes
-   Zero infrastructure overhead (file-based, portable)
-   Phase 2: Add PostgreSQL provider for multi-agent enterprise deployments

**Impact**:

-   Schema versioning via migrations (sqlx prepare mode)
-   Transaction boundaries per-operation (atomic state changes)
-   Event log remains append-only (never update/delete events)

---

### Decision 2: VCS Abstraction

**Title**: VcsProvider Trait with git2 MVP  
**Status**: PROPOSED (ADR-035)  
**Choice**: Implement `VcsProvider` trait with `git2` FFI bindings; GitHub/GitLab deferred to Phase 2

**Rationale**:

-   git2 provides low-level git operations (already proven in indexing)
-   Worktrees for session isolation (no state pollution between sessions)
-   spawn_blocking for FFI (proven pattern in mcb-indexing)
-   Phase 2: Add GitHub API provider for PRs, reviews; GitLab for CI integration

**Impact**:

-   Session → git branch/worktree mapping (1:1)
-   Compensation via `git reset --hard` (fast, safe)
-   Cleanup via `git worktree prune`

---

### Decision 3: Compensation Model

**Title**: Hybrid Auto + Operator Override  
**Status**: PROPOSED (ADR-035)  
**Choice**: Automatic compensation via git reset; operator can override with approval + audit trail

**Rationale**:

-   Git branches are natural rollback points (safe, zero data loss)
-   Operator override captures human decision-making (not just automation)
-   Audit trail (who, when, why) for compliance

**Impact**:

-   All transitions reversible (git commit → git reset)
-   Policy violations trigger automatic compensation + human notification
-   Operator override logged as separate event

---

### Decision 4: Multi-Tier Execution Model

**Title**: Project → Plan → Task → Session → Operator/Agent  
**Status**: PROPOSED (ADR-034)  
**Choice**: 5-tier hierarchy with concurrency at each level (except Operator)

**Rationale**:

-   Mirrors software delivery reality (projects contain phases, phases contain work items, etc.)
-   WIP limits prevent resource exhaustion (e.g., max 3 concurrent sessions/task)
-   Operator is sequential (ensures deterministic ordering)

**Impact**:

-   Each tier has its own state machine
-   Queues for inter-tier work (async message passing)
-   Reporting aggregates across tiers

---

### Decision 5: Hybrid Transaction Model

**Title**: Per-Operation Transactions + Append-Only Event Log  
**Status**: PROPOSED (ADR-038)  
**Choice**: Each action is an atomic transaction; entire session is append-only events

**Rationale**:

-   Per-operation transactions prevent partial state corruption (ACID)
-   Append-only event log enables replay, audit, time-travel queries
-   Dual-write pattern (state + log) ensures consistency

**Impact**:

-   No UPDATE or DELETE on events (only INSERT)
-   State can be reconstructed from event stream
-   Time-travel queries possible (state at any timestamp)

---

### Decision 6: Policy Expansion

**Title**: 11 Policies Across 5 Lifecycle Points  
**Status**: PROPOSED (ADR-036)  
**Choice**: Expand beyond 4 example policies to 11 production-ready policies

**Rationale**:

-   Covers all decision gates in workflow (discovery, validation, execution, compensation, completion)
-   Policies are composable (AND, OR, NOT logic)
-   Dry-run mode for testing policy combinations

**Impact**:

-   PolicyGuardProvider evaluates all 11 policies at each gate
-   Invalid transitions blocked (operator notified, compensation triggered)
-   Policies can be toggled on/off per project

---

### Decision 7: Event Broadcasting

**Title**: 3-Channel Event System  
**Status**: PROPOSED (ADR-037)  
**Choice**: Emit all events to Queue + SQLite + Webhooks simultaneously

**Rationale**:

-   Message Queue: Internal work (compensation, notifications)
-   SQLite: Persistence + history
-   Webhooks: External integration (no polling)

**Impact**:

-   Events guaranteed to reach all 3 channels (at-least-once delivery)
-   Backpressure handling (queue TTL, dead-letter queue for failed webhooks)
-   External systems reactive (not polling)

---

### Decision 8: Beads Integration

**Title**: Task Relationships = Beads; Execution = MCB  
**Status**: PROPOSED (ADR-034)  
**Choice**: Beads is source of truth for task relationships; MCB manages execution state only

**Rationale**:

-   No state duplication (Beads already has task + dependency model)
-   MCB reads tasks from Beads at session start (not cached)
-   Clean separation: Beads = structure, MCB = dynamics

**Impact**:

-   Session opening queries Beads for task details (dependencies, priority, etc.)
-   TaskState changes in Beads (open → in_progress), WorkflowState in MCB
-   No bidirectional sync (MCB is read-only for Beads)

---

### Decision 9: Performance Strategy

**Title**: Benchmark-Driven Optimization  
**Status**: PROPOSED (ADR-037)  
**Choice**: Measure first (Week 1 baseline); optimize only if targets missed (Week 4)

**Rationale**:

-   Premature optimization wastes time
-   Benchmarks identify real bottlenecks (not guesses)
-   Week 1: Establish baselines (FSM, context discovery, policy eval, broadcasting)
-   Week 4: If targets missed, optimize (caching, batching, indexing)

**Targets**:

-   FSM transition: < 10ms
-   Context discovery + cache: < 1s
-   Policy evaluation: < 100ms
-   Event broadcast (all 3 channels): < 500ms
-   SQLite queries: < 50ms (WAL mode, indexed)

**Impact**:

-   Benchmark suite in mcb-domain (microbenchmarks)
-   Integration benchmark suite (end-to-end workflows)
-   CI reports baseline vs. current (regression detection)

---

## 4. Detailed ADR Summaries

### ADR-034: Workflow Session FSM with Event Sourcing

**Location**: `docs/adr/034-workflow-session-fsm.md`  
**Status**: PROPOSED  
**Purpose**: Define WorkflowSession entity and FSM for executing individual sessions

**Key Entities**:

-   `WorkflowSession`: Container for FSM, events, metadata
-   `WorkflowState`: Enum (initialized, discovering_context, validating_policies, executing, compensating, completed, failed)
-   `WorkflowEvent`: Append-only event (StateChanged, PolicyViolation, CompensationStarted, etc.)
-   `Transition`: State + action → new state (guarded by policies)
-   `ProjectContext`: Discovered metadata (repo, branches, dependencies)
-   `GitContext`: VCS state (current branch, worktree, remote commits)
-   `TrackerContext`: Beads task metadata (dependencies, priority, blocked status)

**Key Decisions**:

-   Events are immutable (only INSERT, never UPDATE/DELETE)
-   State reconstructable from event stream (for replay, time-travel)
-   Each session has its own git branch/worktree (no pollution)
-   Transitions guarded by PolicyGuard (ADR-036)

**Dependencies**:

-   Enables: ADR-035 (VCS), ADR-036 (Policy), ADR-037 (Events), ADR-038 (Transactions)
-   Requires: Beads integration for task metadata

**Success Criteria**:

-   ✅ WorkflowSession entity tests (FSM transitions, serde, persistence)
-   ✅ Event sourcing tests (append-only, replay, time-travel)
-   ✅ Context discovery tests (ProjectContext, GitContext, TrackerContext)
-   ✅ Transition guard tests (PolicyGuard integration)

---

### ADR-035: VCS Abstraction with Worktrees

**Location**: `docs/adr/035-vcs-provider-worktrees.md`  
**Status**: PROPOSED  
**Purpose**: Define VcsProvider trait for git operations; implement git2 backend with worktree isolation

**Key Entities**:

-   `VcsProvider`: Trait for git operations (branch creation, worktree management, compensation)
-   `Git2Provider`: Implementation using git2 FFI bindings
-   `WorktreeManager`: Lifecycle management (create, cleanup, prune old worktrees)
-   `CompensationHandler`: Rollback via `git reset --hard` on policy violation

**Key Decisions**:

-   Worktrees provide session isolation (no state pollution)
-   Spawn_blocking for FFI calls (proven in indexing)
-   Compensation is git reset (atomic, safe, zero data loss)
-   Operator can override compensation (with audit trail)

**Dependencies**:

-   Depends on: ADR-034 (WorkflowSession)
-   Enables: ADR-036 (Policy override), ADR-037 (Compensation events)
-   Requires: git2 crate, spawn_blocking executor

**Success Criteria**:

-   ✅ VcsProvider trait tests (branch ops, worktree ops, compensation)
-   ✅ Git2Provider integration tests (real git operations)
-   ✅ WorktreeManager lifecycle tests (create, cleanup, prune)
-   ✅ CompensationHandler tests (rollback correctness)

---

### ADR-036: Policy Enforcement at 5 Lifecycle Points

**Location**: `docs/adr/036-policy-enforcement.md`  
**Status**: PROPOSED  
**Purpose**: Define 11 production-ready policies and their evaluation points

**Key Entities**:

-   `Policy`: Trait (name, description, evaluate method)
-   `PolicyResult`: Success / Violation (with reason)
-   `PolicyGuardProvider`: Composition of all 11 policies
-   `PolicyViolation`: Event logged when policy fails

**The 11 Policies** (organized by lifecycle point):

**1. Discovery Point** (discovering_context):

-   `RequiredContextAvailable`: All required contexts found (repo, Beads task, git config)
-   `DependenciesMet`: All task dependencies are completed (from Beads)
-   `BranchAvailable`: Target branch exists and is accessible

**2. Validation Point** (validating_policies):

-   `CommitMessageFormat`: Commit message matches project template
-   `FileChangesAllowed`: Modified files not in excluded list (vendor/, node_modules/, etc.)
-   `OwnershipVerified`: Operator owns the task or has override permission

**3. Execution Point** (executing):

-   `ResourcesAvailable`: Worktree space, memory, concurrent session limit not exceeded
-   `NoConflictingChanges`: Target branch has no merge conflicts with feature branch

**4. Compensation Point** (compensating):

-   `RollbackFeasible`: Previous commit exists (safe rollback target)

**5. Completion Point** (completed):

-   `AuditTrail`: Policy decisions logged (for compliance)
-   `EventsPersisted`: All 3 channels received completion event

**Key Decisions**:

-   All policies evaluated before transition (fail-fast)
-   Policies are composable (AND logic: all must pass)
-   Dry-run mode for testing policy combinations
-   Operator can override specific policies (with reason, logged)

**Dependencies**:

-   Depends on: ADR-034 (WorkflowSession), ADR-035 (VcsProvider)
-   Enables: ADR-037 (PolicyViolation events)
-   Requires: Beads integration for dependency checking

**Success Criteria**:

-   ✅ Policy trait tests (11 policies, each tested independently)
-   ✅ PolicyGuardProvider tests (composition, AND logic)
-   ✅ Transition guard tests (all 5 lifecycle points)
-   ✅ Operator override tests (with audit trail)

---

### ADR-037: Event Broadcasting on 3 Channels

**Location**: `docs/adr/037-event-broadcasting.md`  
**Status**: PROPOSED  
**Purpose**: Define event system for broadcasting to Message Queue, SQLite, and Webhooks

**Key Entities**:

-   `WorkflowEvent`: Base event type (StateChanged, PolicyViolation, CompensationStarted, etc.)
-   `EventBroadcaster`: Emits events to all 3 channels
-   `MessageQueue`: Internal async work (compensation, notifications) — uses async-channel
-   `EventLog`: SQLite append-only event table
-   `WebhookDispatcher`: HTTP POST to external systems (with retry, backpressure)

**Key Decisions**:

-   All events go to all 3 channels (fire-and-forget pattern)
-   Message Queue for internal work (no external dependency)
-   SQLite for durability + history
-   Webhooks for external integration (CI/CD, Slack, monitoring)
-   At-least-once delivery semantics (retry on failure)
-   Dead-letter queue for failed webhook deliveries

**Dependencies**:

-   Depends on: ADR-034 (WorkflowSession), ADR-036 (PolicyViolation)
-   Enables: MCP tools (subscribe to events), external integrations
-   Requires: async-channel, reqwest (for webhooks), Tokio runtime

**Success Criteria**:

-   ✅ EventBroadcaster tests (fire to all 3 channels)
-   ✅ MessageQueue tests (async work, backpressure)
-   ✅ EventLog tests (persistence, queryability)
-   ✅ WebhookDispatcher tests (retry logic, dead-letter queue)
-   ✅ Integration tests (end-to-end event flow)

---

### ADR-038: Hybrid Transaction Model

**Location**: `docs/adr/038-transaction-model.md`  
**Status**: PROPOSED  
**Purpose**: Define transaction boundaries and consistency guarantees

**Key Entities**:

-   `Transaction`: Per-operation ACID transaction (WorkflowSession update)
-   `EventLog`: Append-only events (separate transactions, no rollback)
-   `Snapshot`: Immutable state snapshot at each commit point

**Key Decisions**:

-   Each action (transition, compensation, etc.) is one transaction
-   Events logged in separate transactions (append-only, never deleted)
-   Dual-write pattern: Update state + log event atomically
-   WAL mode for SQLite (concurrent reads + sequential writes)

**Dependencies**:

-   Depends on: ADR-034 (WorkflowSession), ADR-035 (VcsProvider)
-   Enables: Time-travel queries (state at any timestamp)
-   Requires: sqlx with prepare mode (compile-time verification)

**Success Criteria**:

-   ✅ Transaction tests (atomicity, isolation)
-   ✅ Event log tests (append-only, never deleted)
-   ✅ Consistency tests (state recoverable from events)
-   ✅ Time-travel tests (state at any timestamp)

---

## 5. Implementation Timeline

### Week 0: Amendments (14.5 hours)

**Goal**: Finalize ADRs, team alignment, prepare for implementation

**Day 1 (3 hours): Team Review**

-   [ ] Lead presents ADR-034-038 summaries to team
-   [ ] Team questions, clarifications
-   [ ] Decision: Proceed with implementation? (GO/NO-GO)

**Days 2-3 (6 hours): ADR Refinement**

-   [ ] Team feedback incorporated (decisions locked)
-   [ ] ADR files updated with final rationale
-   [ ] Architecture diagrams finalized
-   [ ] Crate structure skeleton created (empty modules)

**Day 4 (3 hours): Code Review & Consistency**

-   [ ] Walk-through of architecture (domain, ports, services)
-   [ ] Crate dependencies validated (no circular)
-   [ ] Build verification (Cargo check)

**Day 5 (2.5 hours): Branch Setup & Commit**

-   [ ] Create feature branch: `feature/workflow-v0.2.0`
-   [ ] ADRs committed with evidence of team alignment
-   [ ] Crate skeleton committed (empty modules)

**Deliverables**:

-   ✅ ADR-034-038 finalized (proposal → decision)
-   ✅ Architecture diagrams in place
-   ✅ Feature branch ready for Week 1

---

### Week 1: Foundation (40 hours)

**Goal**: Build domain entities and ports; establish test infrastructure

**Engineer A (25 hours): Domain Entities & Tests**

-   `mcb-domain/src/entities/workflow.rs` (200 lines, 5 hours)
    -   `WorkflowSession`: UUID, state, metadata
    -   `WorkflowState`: Enum + Display impl
    -   `WorkflowEvent`: Append-only event types
    -   `Transition`: State + action → new state
    -   Serde support (JSON serialization)
-   `mcb-domain/src/entities/context.rs` (150 lines, 4 hours)
    -   `ProjectContext`: Repository metadata, branch list, commit history
    -   `GitContext`: Current branch, worktree status, upstream tracking
    -   `TrackerContext`: Task ID, title, dependencies, priority
-   `mcb-domain/src/entities/policy.rs` (100 lines, 3 hours)
    -   `Policy`: Trait (name, description, evaluate)
    -   `PolicyResult`: Success / Violation
    -   `PolicyViolation`: Event details
-   `mcb-domain/src/errors/workflow_error.rs` (50 lines, 1 hour)
    -   `WorkflowError`: Domain error types (NoContext, PolicyViolation, etc.)
-   Comprehensive entity tests (8 hours)
    -   FSM transition tests (30 tests)
    -   Serde roundtrip tests (10 tests)
    -   Time-travel query tests (5 tests)
    -   Context discovery tests (10 tests)
    -   Policy evaluation tests (5 tests)

**Engineer B (15 hours): Ports & Traits**

-   `mcb-domain/src/ports/database_provider.rs` (80 lines, 3 hours)
    -   `DatabaseProvider`: Trait (create_session, update_session, append_event, query_at_timestamp)
    -   Associated types for transaction handling
    -   Error types
-   `mcb-domain/src/ports/vcs_provider.rs` (120 lines, 4 hours)
    -   `VcsProvider`: Trait (branch operations, worktree ops, compensation)
    -   Git-specific concepts (branch, commit, worktree)
    -   Error types
-   `mcb-domain/src/ports/context_scout_provider.rs` (80 lines, 3 hours)
    -   `ContextScoutProvider`: Trait (discover_project, discover_git, discover_tracker)
    -   Caching strategy (in-memory with TTL)
-   `mcb-domain/src/ports/policy_guard_provider.rs` (100 lines, 3 hours)
    -   `PolicyGuardProvider`: Trait (evaluate, evaluate_all, dry_run)
    -   Composition of 11 policies
-   Port trait tests (2 hours)
    -   Mock provider tests (10 tests)

**Deliverables**:

-   ✅ All domain entities implemented + tested
-   ✅ All provider traits defined
-   ✅ 60 domain tests passing
-   ✅ Error handling complete
-   ✅ Serde support verified

---

### Week 2: Providers & Services (40 hours)

**Goal**: Implement all providers; build application services

**Engineer A (20 hours): Provider Implementations**

-   `mcb-providers/src/sqlite_provider.rs` (300 lines, 8 hours)
    -   `SqliteDatabaseProvider` implementation
    -   Schema creation (WorkflowSession table, EventLog table)
    -   Transaction handling (atomicity)
    -   Query at timestamp (time-travel)
    -   WAL mode configuration
    -   Migration framework (sqlx prepare mode)
-   `mcb-providers/src/git2_provider.rs` (400 lines, 8 hours)
    -   `Git2Provider` implementation
    -   Branch operations (create, delete, switch)
    -   Worktree management (create, cleanup, prune)
    -   Compensation (git reset --hard)
    -   spawn_blocking wrapper for FFI
    -   Error handling (git2 error types)
-   `mcb-providers/src/cached_context_scout.rs` (200 lines, 3 hours)
    -   `CachedContextScout` implementation
    -   ProjectContext discovery (git ls-remote)
    -   GitContext discovery (git status)
    -   TrackerContext discovery (Beads API call)
    -   In-memory cache with TTL
    -   Cache invalidation on events
-   Provider tests (1 hour)
    -   Mock-based tests (30 tests)

**Engineer B (20 hours): Application Services**

-   `mcb-application/src/services/workflow_service.rs` (300 lines, 10 hours)
    -   `WorkflowService` orchestration
    -   Session opening (context discovery, policy validation)
    -   Transition execution (state machine advancement)
    -   Compensation on failure (git reset, event emission)
    -   Event broadcast (all 3 channels)
    -   Operator override handling
-   `mcb-application/src/services/session_manager.rs` (200 lines, 5 hours)
    -   `SessionManager`: CRUD operations
    -   Persistence (to database)
    -   Query interface (list, get, query_at_timestamp)
-   `mcb-application/src/services/compensation_handler.rs` (150 lines, 3 hours)
    -   `CompensationHandler`: Rollback logic
    -   Policy violation → compensation trigger
    -   Audit trail logging
-   `mcb-application/src/services/event_broadcaster.rs` (200 lines, 2 hours)
    -   `EventBroadcaster`: Emit to 3 channels
    -   Message queue (async-channel)
    -   SQLite logging
    -   Webhook dispatch (reqwest)
-   Service tests (5 hours)
    -   Orchestration tests (15 tests)
    -   Integration tests (15 tests)

**Deliverables**:

-   ✅ All provider implementations tested
-   ✅ All application services implemented + tested
-   ✅ 60 provider/service tests passing
-   ✅ SQLite schema finalized
-   ✅ Event broadcasting working on all 3 channels

---

### Week 3: Integration & Benchmarks (40 hours)

**Goal**: MCP handlers, Beads integration, performance baselines

**Engineer A (20 hours): MCP & Beads Integration**

-   `mcb-server/src/handlers/workflow_handler.rs` (300 lines, 10 hours)
    -   MCP workflow tool with action-based handlers:
        -   `open-session`: Create new session
        -   `advance`: Execute transition
        -   `compensate`: Trigger rollback
        -   `query`: Time-travel state query
        -   `override-policy`: Operator override
    -   Message parsing (JSON → internal types)
    -   Response formatting
    -   Error handling
-   `mcb-application/src/services/beads_integration.rs` (200 lines, 5 hours)
    -   Beads API client (read-only)
    -   Task opening workflow (listen to Beads, create MCB session)
    -   Dependency checking (before session start)
    -   Graceful fallback (cache tasks locally if Beads unavailable)
-   `mcb-server/src/handlers/event_subscription.rs` (150 lines, 3 hours)
    -   MCP subscribe tool for event streaming
    -   Filter by event type, session ID, etc.
    -   Real-time updates
-   Integration tests (2 hours)
    -   MCP handler tests (10 tests)
    -   Beads integration tests (5 tests)

**Engineer B (20 hours): Benchmarks & System Tests**

-   `mcb-domain/benches/fsm_benchmark.rs` (100 lines, 2 hours)
    -   FSM transition latency (target: < 10ms)
    -   Context discovery latency (target: < 1s)
    -   Policy evaluation latency (target: < 100ms)
    -   Event broadcast latency (target: < 500ms)
-   `mcb-providers/benches/sqlite_benchmark.rs` (150 lines, 3 hours)
    -   SQLite query latency (target: < 50ms)
    -   Transaction throughput
    -   Event log append performance
    -   WAL mode effectiveness
-   System integration tests (10 hours)
    -   Full workflow scenario: Task → Plan → Session → Complete (10 tests)
    -   Multi-agent concurrency (5 tests)
    -   Compensation/rollback scenarios (5 tests)
    -   Policy enforcement across transitions (5 tests)
    -   Event broadcasting on all 3 channels (5 tests)
-   Performance report (2 hours)
    -   Baseline measurements
    -   Identify bottlenecks (if any)
    -   Optimization plan (if needed)

**Deliverables**:

-   ✅ MCP workflow tool end-to-end
-   ✅ Beads integration working (task opening workflow)
-   ✅ Benchmark suite established (Week 1 baseline)
-   ✅ 30 system integration tests passing
-   ✅ Performance report (targets met? where to optimize?)

---

### Week 4: Polish & Hardening (30 hours)

**Goal**: Documentation, E2E tests, release preparation

**Lead Engineer (30 hours): Documentation, E2E, Release**

-   Implementation guide (8 hours)
    -   Architecture deep-dive (reference implementation)
    -   Provider plugin system (how to add new providers)
    -   Policy custom rules (how to define custom policies)
    -   Event system integration (how to consume events)
-   E2E test scenarios (10 hours)
    -   Full workflow: Beads task → MCB session → completion (5 tests)
    -   Multi-task parallel execution (3 tests)
    -   Operator override scenarios (2 tests)
    -   Failure recovery + compensation (3 tests)
    -   Policy violations + remediation (2 tests)
-   Bug fixes & optimization (8 hours)
    -   Address any Week 3 findings
    -   Performance tuning (if benchmarks missed targets)
    -   Error handling edge cases
    -   Logging/observability improvements
-   Release preparation (4 hours)
    -   Version bump (v0.2.0)
    -   CHANGELOG update
    -   Tag creation
    -   Final CI verification

**Deliverables**:

-   ✅ Implementation guide (20 pages)
-   ✅ 15 E2E tests passing
-   ✅ All bugs fixed
-   ✅ Performance targets validated
-   ✅ v0.2.0 tagged and ready for release

---

## 6. Crate-by-Crate Breakdown

### mcb-domain (50 hours, ~600 LOC)

**Purpose**: Domain entities, FSM, event sourcing, traits

**New Files**:

-   `src/entities/workflow.rs` (200 lines)
    -   `WorkflowSession` struct
    -   `WorkflowState` enum
    -   `WorkflowEvent` enum
    -   `Transition` struct
    -   Serde implementations
-   `src/entities/context.rs` (150 lines)
    -   `ProjectContext` struct
    -   `GitContext` struct
    -   `TrackerContext` struct
    -   Context cache trait
-   `src/entities/policy.rs` (100 lines)
    -   `Policy` trait
    -   `PolicyResult` enum
    -   `PolicyViolation` struct
-   `src/ports/database_provider.rs` (80 lines)
    -   `DatabaseProvider` trait
    -   Database operations (CRUD, query, time-travel)
-   `src/ports/vcs_provider.rs` (120 lines)
    -   `VcsProvider` trait
    -   Git operations (branches, worktrees, compensation)
-   `src/ports/context_scout_provider.rs` (80 lines)
    -   `ContextScoutProvider` trait
    -   Context discovery operations
-   `src/ports/policy_guard_provider.rs` (100 lines)
    -   `PolicyGuardProvider` trait
    -   Policy evaluation operations
-   `src/errors/workflow_error.rs` (50 lines)
    -   `WorkflowError` enum
    -   Error conversion impls

**Tests** (60 tests):

-   `tests/workflow_fsm_test.rs` (30 tests)
    -   FSM transition validation
    -   Serde roundtrip (JSON serialization)
    -   Time-travel state queries
    -   Immutability checks
-   `tests/context_test.rs` (15 tests)
    -   ProjectContext discovery
    -   GitContext from git operations
    -   TrackerContext from Beads
    -   Cache behavior
-   `tests/policy_test.rs` (15 tests)
    -   Policy trait interface
    -   PolicyResult creation
    -   PolicyViolation events

---

### mcb-providers (50 hours, ~1,000 LOC)

**Purpose**: Concrete provider implementations (SQLite, git2, context caching)

**New Files**:

-   `src/sqlite_provider.rs` (300 lines)
    -   `SqliteDatabaseProvider` implementation
    -   Schema (WorkflowSession, EventLog tables)
    -   Transactions (atomicity)
    -   Time-travel queries (SELECT * WHERE timestamp <= X)
    -   WAL mode setup
-   `src/git2_provider.rs` (400 lines)
    -   `Git2Provider` implementation
    -   Branch operations
    -   Worktree management
    -   Compensation (reset logic)
    -   spawn_blocking wrappers
-   `src/cached_context_scout.rs` (200 lines)
    -   `CachedContextScout` implementation
    -   ProjectContext discovery
    -   GitContext discovery
    -   TrackerContext discovery
    -   In-memory cache with TTL
-   `src/lib.rs` (100 lines)
    -   Module organization
    -   Public exports

**Tests** (60 tests):

-   `tests/sqlite_provider_test.rs` (20 tests)
    -   Session creation/update
    -   Event appending
    -   Time-travel queries
    -   Concurrent access (WAL mode)
-   `tests/git2_provider_test.rs` (20 tests)
    -   Branch operations
    -   Worktree lifecycle
    -   Compensation correctness
    -   FFI safety
-   `tests/context_scout_test.rs` (10 tests)
    -   Context discovery
    -   Cache behavior
    -   Invalidation
-   `tests/composite_guard_test.rs` (10 tests)
    -   Policy composition
    -   AND logic

---

### mcb-application (50 hours, ~700 LOC)

**Purpose**: Application services (orchestration, session management, event broadcasting)

**New Files**:

-   `src/services/workflow_service.rs` (300 lines)
    -   `WorkflowService` struct
    -   `open_session` (discovery + policy validation)
    -   `advance` (state transition)
    -   `compensate` (rollback)
    -   `query_at_timestamp` (time-travel)
    -   `override_policy` (operator decision)
    -   Dependency injection of providers
-   `src/services/session_manager.rs` (200 lines)
    -   `SessionManager` struct
    -   CRUD operations
    -   Persistence layer
    -   Query interface
-   `src/services/compensation_handler.rs` (150 lines)
    -   `CompensationHandler` struct
    -   Rollback logic
    -   Audit trail
    -   Error recovery
-   `src/services/event_broadcaster.rs` (200 lines)
    -   `EventBroadcaster` struct
    -   Emit to message queue
    -   Emit to SQLite
    -   Emit to webhooks
    -   Retry logic
    -   Dead-letter queue
-   `src/services/beads_integration.rs` (150 lines)
    -   Beads API client
    -   Task opening workflow
    -   Dependency checking
    -   Graceful fallback
-   `src/lib.rs` (50 lines)
    -   Module organization

**Tests** (60 tests):

-   `tests/workflow_service_test.rs` (25 tests)
    -   Session opening
    -   Transition execution
    -   Policy enforcement
    -   Compensation
-   `tests/session_manager_test.rs` (15 tests)
    -   CRUD operations
    -   Persistence
    -   Query interface
-   `tests/event_broadcaster_test.rs` (15 tests)
    -   3-channel emission
    -   Retry logic
    -   Dead-letter handling
-   `tests/beads_integration_test.rs` (5 tests)
    -   Task opening
    -   Dependency checking

---

### mcb-server (30 hours, ~400 LOC)

**Purpose**: MCP handlers for workflow tool

**New Files**:

-   `src/handlers/workflow_handler.rs` (300 lines)
    -   `open-session` action handler
    -   `advance` action handler
    -   `compensate` action handler
    -   `query` action handler
    -   `override-policy` action handler
    -   Message parsing (JSON → types)
    -   Response formatting
    -   Error handling
-   `src/handlers/event_subscription.rs` (100 lines)
    -   `subscribe` tool handler
    -   Event filtering (type, session, etc.)
    -   Real-time streaming
    -   Cleanup on disconnect
-   `src/lib.rs` or `src/main.rs` updates
    -   Register new handlers
    -   Wire up providers
    -   Dependency injection

**Tests** (30 tests):

-   `tests/workflow_handler_test.rs` (20 tests)
    -   Action handler tests (mock providers)
    -   Message parsing
    -   Response formatting
    -   Error cases
-   `tests/event_subscription_test.rs` (10 tests)
    -   Subscription creation
    -   Event filtering
    -   Real-time delivery

---

### mcb-infrastructure (20 hours, ~200 LOC)

**Purpose**: Configuration, dependency injection, cache management

**New Files**:

-   `src/config/workflow_config.rs` (100 lines)
    -   WorkflowConfig struct
    -   Database path, git2 options, webhook endpoints
    -   Policy toggles
    -   Event channel sizes
-   `src/di/workflow_container.rs` (100 lines)
    -   Dependency injection container
    -   Provider singletons
    -   Service factories
    -   Shutdown handling

**Tests** (20 tests):

-   `tests/config_test.rs` (10 tests)
    -   Config loading (TOML)
    -   Validation
    -   Defaults
-   `tests/di_test.rs` (10 tests)
    -   Container setup
    -   Service creation
    -   Singletons behavior

---

## 7. Testing Strategy

### Test Coverage Breakdown

| Category | Count | Scope |
|----------|-------|-------|
| **Unit Tests** | 300 | Domain entities, services, providers |
| **Integration Tests** | 40 | Cross-crate workflows |
| **E2E Tests** | 20 | Full workflow scenarios |
| **Performance Tests** | 20 | Benchmark suite |
| **Total** | **380** | |

### Unit Tests (60 per crate × 5 = 300)

**mcb-domain** (60 tests):

-   FSM transitions (20 tests): all 7 states, all transitions, guards
-   Event sourcing (15 tests): append-only, replay, immutability
-   Context (15 tests): discovery, caching, invalidation
-   Policy (10 tests): trait interface, composition

**mcb-providers** (60 tests):

-   SQLite (20 tests): CRUD, transactions, time-travel, concurrency
-   git2 (20 tests): branches, worktrees, compensation, error handling
-   Context Scout (10 tests): discovery, caching, fallback
-   Composite Guard (10 tests): policy composition, AND logic

**mcb-application** (60 tests):

-   Workflow Service (25 tests): session opening, transitions, compensation, override
-   Session Manager (15 tests): CRUD, persistence, queries
-   Event Broadcaster (15 tests): 3-channel emission, retry, dead-letter
-   Beads Integration (5 tests): task opening, dependency checking

**mcb-server** (60 tests):

-   Workflow Handler (35 tests): 5 action handlers, message parsing, errors
-   Event Subscription (15 tests): filtering, streaming, cleanup
-   Config & DI (10 tests): loading, validation, injection

**mcb-infrastructure** (60 tests):

-   Config (20 tests): TOML parsing, validation, defaults
-   DI Container (20 tests): setup, factories, singletons
-   Cache (20 tests): TTL, invalidation, thread-safety

**Total Unit Tests**: 300

### Integration Tests (40)

**Workflow-to-Database** (10 tests):

-   Session persists across restarts
-   Event log is append-only
-   Time-travel queries work
-   Concurrent access (WAL mode)
-   Transaction rollback on error

**Git Integration** (10 tests):

-   Branch creation for session
-   Worktree isolation
-   Compensation (reset) works
-   Cleanup on completion
-   Large repo handling (git2 safety)

**Policy Enforcement** (10 tests):

-   All 11 policies evaluated
-   Violations block transitions
-   Dry-run mode works
-   Operator override creates audit trail
-   Policy composition (AND logic)

**Event Broadcasting** (10 tests):

-   Events reach queue, DB, webhooks
-   Retry logic for failed webhooks
-   Dead-letter queue for persistent failures
-   Event ordering preserved
-   Filtering works correctly

**Total Integration Tests**: 40

### E2E Tests (20)

**Full Workflow Scenarios** (5 tests):

-   Task creation → session opening → context discovery → policy validation → execution → completion
-   Multi-task parallel execution (concurrent sessions, WIP limits)
-   Operator override (policy violation → override → completion)
-   Compensation trigger (policy violation → auto compensation → revert)
-   Time-travel query (state at any point in workflow)

**Multi-Agent Concurrency** (5 tests):

-   Two concurrent sessions, same repo (no conflicts)
-   Two concurrent sessions, same branch (conflict detection)
-   Session ordering (sequential operator queue)
-   Event ordering (no race conditions)

**Failure Recovery** (5 tests):

-   Session crash + restart (persisted state)
-   Git error handling (branch not found, etc.)
-   Beads API failure (graceful fallback, cached tasks)
-   Policy violation + compensation
-   Webhook delivery failure + retry

**MCP Tool Integration** (5 tests):

-   MCP workflow tool invocation (open-session)
-   Streaming results (event subscription)
-   Error reporting (invalid action, missing context)
-   Authorization (operator override)

**Total E2E Tests**: 20

### Performance Tests (20)

**Microbenchmarks** (in code, via `cargo bench`):

-   FSM transition: < 10ms (target)
-   Policy evaluation: < 100ms (target)
-   Context discovery: < 1s (target)
-   Event broadcast: < 500ms (target)
-   SQLite query: < 50ms (target)

**Integration Benchmarks**:

-   Full session lifecycle: < 5s (target)
-   Compensation execution: < 1s (target)
-   Multi-session throughput (target: 10 sessions/min)

**Regression Detection**:

-   CI compares current vs. baseline
-   Alerts on > 10% regression
-   Week 1 baseline captured
-   Week 4 optimization validation

---

## 8. Success Criteria & Validation

### Code Quality

**Zero Technical Debt**:

-   ✅ No `unwrap()` or `expect()` in implementations (except main.rs error handling)
-   ✅ All public APIs documented (rustdoc)
-   ✅ Error types derive `Display + std::error::Error`
-   ✅ Clippy: zero warnings on `cargo clippy --all-targets --all-features`
-   ✅ fmt: `cargo fmt --all -- --check` passes

**Testing**:

-   ✅ All tests pass: `cargo test --all`
-   ✅ Code coverage ≥ 80% (measured via tarpaulin)
-   ✅ No flaky tests (run 10x in CI)
-   ✅ Integration tests pass with real git repo, real SQLite DB

**Architecture**:

-   ✅ Architecture validation passes: `cargo xtask validate`
-   ✅ No circular dependencies between crates
-   ✅ Provider traits fully abstracted (swappable implementations)
-   ✅ Dependency graph documented (docs/architecture.md)

---

### Functional Requirements

**Session Persistence**:

-   ✅ Session state persists to SQLite
-   ✅ Session recovered correctly after restart
-   ✅ Event log reconstructs full history
-   ✅ Multiple sessions isolated (git worktrees)

**Time-Travel Queries**:

-   ✅ Query state at any timestamp (SELECT WHERE timestamp <= X)
-   ✅ Rebuild state by replaying events up to timestamp
-   ✅ Performance: < 500ms even for 10,000 events

**Policy Enforcement**:

-   ✅ All 11 policies evaluated at correct lifecycle points
-   ✅ Invalid transitions blocked (error returned)
-   ✅ Operator can override (with permission check, audit logged)
-   ✅ Dry-run mode tests policy combinations (no side effects)

**Compensation & Rollback**:

-   ✅ Failed transitions trigger automatic compensation
-   ✅ Compensation = `git reset --hard` to safe commit
-   ✅ Compensation events logged
-   ✅ State consistent after compensation

**Event Broadcasting**:

-   ✅ All events broadcast to message queue
-   ✅ All events appended to SQLite event log
-   ✅ All events POSTed to webhook endpoints
-   ✅ Failed webhook deliveries retried (exponential backoff)
-   ✅ Dead-letter queue for persistent failures

**Beads Integration**:

-   ✅ MCB reads task metadata from Beads (not cached beyond TTL)
-   ✅ Task opening workflow (Beads change → MCB session creation)
-   ✅ Dependency checking (block session if dependencies not met)
-   ✅ Graceful fallback if Beads unavailable (cached tasks + warning)

**MCP Workflow Tool**:

-   ✅ `open-session` action creates workflow session
-   ✅ `advance` action executes state transition
-   ✅ `compensate` action triggers rollback
-   ✅ `query` action returns state at timestamp
-   ✅ `override-policy` action requires permission + audit trail
-   ✅ All Actions return proper MCP responses (success/error)

---

### Performance Targets

**Latency Targets** (measure in Week 1, validate in Week 4):

| Operation | Target | Measurement |
|-----------|--------|-------------|
| FSM transition | < 10ms | Microbench (fsm_benchmark.rs) |
| Context discovery | < 1s | Integration test |
| Policy evaluation (1 policy) | < 100ms | Microbench |
| Policy evaluation (all 11) | < 500ms | Integration test |
| Event broadcast (3 channels) | < 500ms | Integration test |
| SQLite query (single session) | < 50ms | Microbench (sqlite_benchmark.rs) |
| Full session lifecycle | < 5s | E2E test |
| Compensation execution | < 1s | Integration test |

**Throughput Targets**:

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Concurrent sessions | 3 (WIP limit) | Integration test |
| Sequential operator Actions | 10 Actions/min | Integration test |
| Event queue processing | 100 events/sec | Performance test |

**Resource Targets**:

| Resource | Target | Measurement |
|----------|--------|-------------|
| SQLite DB size (10,000 sessions) | < 500MB | Storage test |
| Memory (idle) | < 50MB | Process monitor |
| Memory (active session) | < 200MB | Process monitor |
| Worktree disk space (5 concurrent) | < 2GB | Disk usage test |

**Regression Detection**:

-   ✅ CI captures baseline metrics (Week 1)
-   ✅ CI compares on every merge (> 10% = alert)
-   ✅ Performance report generated (Week 4)

---

### Integration Checklist

**External Systems**:

-   ✅ Beads API client works (read-only)
-   ✅ Webhook dispatch to external systems (configurable)
-   ✅ MCP workflow tool can be invoked from Claude
-   ✅ Git operations work with real repositories

**Data Consistency**:

-   ✅ No state duplication (Beads = task relationships, MCB = execution)
-   ✅ Dual-write pattern (state + event) is atomic
-   ✅ Event log never corrupted (append-only)
-   ✅ Time-travel always correct (state reconstructable)

**Error Handling**:

-   ✅ All errors propagate cleanly (no panics)
-   ✅ User-facing errors are actionable (not "Unknown error")
-   ✅ System-level errors are logged + metrics sent
-   ✅ Graceful degradation (fallback to cached data, etc.)

---

## 9. Risk Mitigation

### High-Risk Items

| Risk | Impact | Probability | Mitigation | Owner |
|------|--------|-------------|-----------|-------|
| ADRs rejected in Week 0 | Major rework (2+ weeks) | Medium | Team alignment in Week 0 (3h review) | Lead |
| git2 FFI blocking | Complexity, performance issues | Low | spawn_blocking proven; benchmarks week 1 | Engineer A |
| SQLite WAL conflicts | Concurrent writes blocked | Low | Transactions + queue serialization; test WAL mode | Engineer A |
| Policy composition bugs | Invalid transition validation fails | Medium | Comprehensive tests (30+ tests); dry-run mode | Engineer B |
| Worktree disk explosion | Large repos use 10GB+/worktree | Medium | Clean up old worktrees weekly; monitor disk | Engineer A |
| Event queue overload | Message queue backpressure | Low | TTL + backpressure monitoring; dead-letter queue | Engineer B |
| Beads API failure | Task opening blocked | Medium | Graceful fallback; cache tasks locally; retry logic | Engineer B |
| Performance targets missed | Week 4 crunch; possible cuts | Low | Benchmark Week 1; identify bottlenecks early | Engineer B |

### Mitigation Strategies

**Week 0 Alignment**: 3-hour team review ensures all 9 decisions are understood + approved. If rejected, pivot decision made immediately (no Week 1 delays).

**Early Benchmarking**: Week 1 includes microbench suite; if targets missed, Week 3-4 optimization planned immediately (not last-minute).

**Fallback Modes**: Beads unavailable? Use cached tasks. Webhook fails? Retry with exponential backoff + dead-letter queue. Git error? Log + operator notified.

**Testing**: 380 tests catch bugs early (not in production). Integration tests use real git + SQLite (not mocks).

---

## 10. Resource Allocation

### Weekly Breakdown

**Week 0** (Lead Engineer, 14.5 hours):

-   Day 1 (3h): Team presentation + feedback
-   Days 2-3 (6h): ADR refinement + updates
-   Day 4 (3h): Code review + consistency check
-   Day 5 (2.5h): Branch setup + commit

**Week 1** (2 Engineers, 40 hours total):

-   Engineer A (25h): Domain entities + tests
-   Engineer B (15h): Port traits + trait tests

**Week 2** (2 Engineers, 40 hours total):

-   Engineer A (20h): SQLite + git2 provider implementations
-   Engineer B (20h): Application services + event broadcasting

**Week 3** (2 Engineers, 40 hours total):

-   Engineer A (20h): MCP handlers + Beads integration
-   Engineer B (20h): Benchmarks + system integration tests

**Week 4** (Lead Engineer, 30 hours):

-   Implementation guide (8h)
-   E2E tests (10h)
-   Bug fixes + optimization (8h)
-   Release prep (4h)

### Total Capacity

| Engineer | Role | Hours | Notes |
|----------|------|-------|-------|
| Lead | Architecture + Polish | 36.5 | Week 0 + Week 4 |
| Engineer A | Domain + Providers | 80 | Weeks 1-3 |
| Engineer B | Services + Integration | 80 | Weeks 1-3 |
| **Total** | | **196.5** | ~50 hours/week (4 engineers × 10 hours reasonable capacity) |

### Cost (Estimation)

Assuming $150/hour blended rate (salary + benefits):

-   Lead: 36.5h × $150 = $5,475
-   Engineer A: 80h × $150 = $12,000
-   Engineer B: 80h × $150 = $12,000
-   **Total**: ~$29,475 (1-month project, 2 engineers + lead)

---

## 11. Next Steps

### Immediate Actions (Day 1)

-   [ ] Schedule 1-hour team alignment meeting
-   [ ] Present ADR-034-038 summaries
-   [ ] Collect feedback: Accept / Request Changes / Reject

### Week 0 Decisions (By EOD Friday)

-   [ ] **Team Decision**: Proceed with ADR-034-038 as-is, or request changes?
-   [ ] **Branch**: Create `feature/workflow-v0.2.0` from main
-   [ ] **ADR Finalization**: Address feedback, lock decisions
-   [ ] **Skeleton Crates**: Create empty modules (Cargo check passes)
-   [ ] **CI Setup**: Ensure CI runs tests automatically

### Week 1 Kick-Off (Monday)

-   [ ] Engineer A: Start domain entities (entities/workflow.rs, entities/context.rs)
-   [ ] Engineer B: Start port traits (ports/database_provider.rs, etc.)
-   [ ] Both: Daily standup (15 min)
-   [ ] Lead: Unblock any questions/decisions

### Weekly Milestones

**End of Week 1**:

-   All domain entities + tests
-   All port traits defined
-   60 tests passing
-   Codebase builds + CI green

**End of Week 2**:

-   All providers implemented + tested
-   All services implemented + tested
-   120 tests passing
-   SQLite schema finalized
-   Event broadcasting working

**End of Week 3**:

-   MCP handlers working end-to-end
-   Beads integration complete
-   Benchmarks established (baselines)
-   350+ tests passing
-   Performance report ready

**End of Week 4**:

-   Implementation guide written
-   All E2E tests passing
-   v0.2.0 tagged + ready
-   Zero known bugs
-   Release notes prepared

### Decision Gates

| Gate | Decision | Owner | Deadline |
|------|----------|-------|----------|
| **Go/No-Go** | Proceed with ADRs? | Team | EOD Week 0 Day 1 |
| **Week 1 Review** | Foundation complete? Continue? | Lead | EOD Week 1 |
| **Week 2 Review** | Providers + services SOLID? Continue? | Lead | EOD Week 2 |
| **Week 3 Review** | Integration + benchmarks on track? Continue? | Lead | EOD Week 3 |
| **Release** | All tests pass + docs complete? Release v0.2.0? | Lead | EOD Week 4 |

---

## Appendix: ADR File References

All ADRs are PROPOSED (pending team approval in Week 0):

-   **ADR-034**: `docs/adr/034-workflow-session-fsm.md` — Session FSM + event sourcing
-   **ADR-035**: `docs/adr/035-vcs-provider-worktrees.md` — VCS abstraction + worktrees
-   **ADR-036**: `docs/adr/036-policy-enforcement.md` — 11 policies, 5 lifecycle points
-   **ADR-037**: `docs/adr/037-event-broadcasting.md` — 3-channel event system
-   **ADR-038**: `docs/adr/038-transaction-model.md` — Hybrid transaction model

---

## Appendix: File Structure After Implementation

### New Directories

```
mcb/
├── docs/plan/
│   └── IMPLEMENTATION-PLAN.md (this file)
├── docs/adr/
│   ├── 034-workflow-session-fsm.md
│   ├── 035-vcs-provider-worktrees.md
│   ├── 036-policy-enforcement.md
│   ├── 037-event-broadcasting.md
│   └── 038-transaction-model.md
└── crates/
    ├── mcb-domain/
    │   ├── src/
    │   │   ├── entities/
    │   │   │   ├── workflow.rs
    │   │   │   ├── context.rs
    │   │   │   └── policy.rs
    │   │   ├── ports/
    │   │   │   ├── database_provider.rs
    │   │   │   ├── vcs_provider.rs
    │   │   │   ├── context_scout_provider.rs
    │   │   │   └── policy_guard_provider.rs
    │   │   ├── errors/
    │   │   │   └── workflow_error.rs
    │   │   └── lib.rs
    │   └── tests/
    │       ├── workflow_fsm_test.rs
    │       ├── context_test.rs
    │       └── policy_test.rs
    ├── mcb-providers/
    │   ├── src/
    │   │   ├── sqlite_provider.rs
    │   │   ├── git2_provider.rs
    │   │   ├── cached_context_scout.rs
    │   │   └── lib.rs
    │   └── tests/
    │       ├── sqlite_provider_test.rs
    │       ├── git2_provider_test.rs
    │       ├── context_scout_test.rs
    │       └── composite_guard_test.rs
    ├── mcb-application/
    │   ├── src/services/
    │   │   ├── workflow_service.rs
    │   │   ├── session_manager.rs
    │   │   ├── compensation_handler.rs
    │   │   ├── event_broadcaster.rs
    │   │   └── beads_integration.rs
    │   └── tests/
    │       ├── workflow_service_test.rs
    │       ├── session_manager_test.rs
    │       ├── event_broadcaster_test.rs
    │       └── beads_integration_test.rs
    ├── mcb-server/
    │   ├── src/handlers/
    │   │   ├── workflow_handler.rs
    │   │   └── event_subscription.rs
    │   └── tests/
    │       ├── workflow_handler_test.rs
    │       └── event_subscription_test.rs
    └── mcb-infrastructure/
        ├── src/
        │   ├── config/
        │   │   └── workflow_config.rs
        │   ├── di/
        │   │   └── workflow_container.rs
        │   └── lib.rs
        └── tests/
            ├── config_test.rs
            └── di_test.rs
```

---

## Document Control

| Field | Value |
|-------|-------|
| **Document** | MCB Workflow v0.2.0 Implementation Plan |
| **Version** | 1.0 |
| **Status** | READY FOR TEAM REVIEW |
| **Created** | 2025-01-21 |
| **Last Updated** | 2025-01-21 |
| **Owner** | Lead Engineer |
| **Stakeholders** | Engineering team, Product |
| **Next Review** | Week 0 (post-feedback) |
| **Archive Date** | 2025-05-21 (90 days post-release) |

---

**END OF DOCUMENT**

---

This unified implementation plan is the single source of truth for v0.2.0. All intermediate analysis documents (.planning/*.md) will be deleted after team approval.
