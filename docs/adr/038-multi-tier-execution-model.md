---
adr: 38
title: Multi-Tier Execution Model — Integration of ADR-034–037
status: ACCEPTED
created: 
updated: 2026-02-06
related: [13, 23, 25, 29, 33]
supersedes: []
superseded_by: []
implementation_status: Complete
---

## ADR-038: Multi-Tier Execution Model — Integration of ADR-034–037

## Status

**Proposed** — 2026-02-05

-   **Deciders:** Project team
-   **Depends on:** [ADR-034](./034-workflow-core-fsm.md) (Workflow FSM), [ADR-035](./035-context-scout.md) (Context Scout), [ADR-036](./036-enforcement-policies.md) (Enforcement Policies), [ADR-037](./037-workflow-orchestrator.md) (Orchestrator)
-   **Related:** [ADR-029](./029-hexagonal-architecture-dill.md) (Hexagonal DI), [ADR-013](./013-clean-architecture-crate-separation.md) (Clean Architecture), [ADR-023](./023-provider-registration-linkme.md) (linkme), [ADR-025](./025-figment-configuration.md) (Figment), [ADR-033](./033-mcp-handler-consolidation.md) (MCP Handlers)
-   **Supersedes:** None (integrating series)
-   **Series:** [ADR-034](./034-workflow-core-fsm.md) → [ADR-035](./035-context-scout.md) → [ADR-036](./036-enforcement-policies.md) → [ADR-037](./037-workflow-orchestrator.md) → **ADR-038**

## Context

ADR-034 through ADR-037 define four sequential architectural concerns:

-   **ADR-034**: Workflow FSM with SQLite persistence (state machine, transitions, audit log)
-   **ADR-035**: Context Scout — project state discovery (git, tracker, phases)
-   **ADR-036**: Enforcement Policies — rules that guard transitions (WIP limits, test requirements, approval gates)
-   **ADR-037**: Workflow Orchestrator — MCP integration, agent coordination, operator decisions

Each ADR defines a provider trait and entities, consumed by the next layer. However, **the relationships between entities, concurrency model, and Git integration are scattered across four documents**. This makes it difficult for implementers to understand:

1.  How do `Project`, `Plan`, `Task`, `Session`, `Agent`, and `Operator` entities relate?
2.  What is the concurrency model? Can tasks run in parallel? Sessions?
3.  How does Git worktree isolation work with concurrent sessions?
4.  What is the exact operator workflow for approving code changes?

**This ADR** unifies ADR-034–037 into a complete execution model by:

-   Defining all entity relationships (entity-relationship model)
-   Documenting both state machines (Task state, Session state, Operator state)
-   Clarifying concurrency boundaries (project, plan, task, session, agent, operator levels)
-   Detailing Git integration and worktree lifecycle
-   Explaining operator workflow and compensation strategies

## Decision

### 1. Entity Model

All five entities work together to form a complete execution hierarchy:

```
┌─────────────────────────────────────────────────┐
│         Project (top-level scope)               │
│  - id, name, root_path, config                  │
│  - Lifecycle: Created → Active → Archived       │
└────────────────┬────────────────────────────────┘
                 │ contains N
                 ▼
┌─────────────────────────────────────────────────┐
│       Plan (from Beads — phase)                 │
│  - id, project_id, phase_name, tasks[]          │
│  - Status: Open → InProgress → Closed           │
└────────────────┬────────────────────────────────┘
                 │ contains N
                 ▼
┌─────────────────────────────────────────────────┐
│      Task (from Beads — atomic work unit)       │
│  - id, plan_id, title, status (Beads managed)   │
│  - Lifecycle: Open → InProgress → Merged        │
└────────────────┬────────────────────────────────┘
                 │ creates 1 exclusive
                 ▼
┌─────────────────────────────────────────────────┐
│   Session (Workflow execution context)          │
│  - id, task_id, operator_id, state (FSM)        │
│  - branch, agents[], compensation_plan          │
│  - Created → AgentsLaunched → Running → ...     │
└────────┬────────────────────┬───────────────────┘
         │ runs 1             │ decides
         ▼                     ▼
┌────────────────────┐  ┌──────────────────┐
│  Agent(s) (N)      │  │   Operator (1)   │
│ - id, session_id   │  │ - id, permissions│
│ - status, output   │  │ - active_sessions│
└────────────────────┘  └──────────────────┘
```

#### Project Entity

**Purpose**: Top-level scope for all workflow activity. Coordinates configuration and multi-tenant isolation.

**Fields**:

-   `id: String` — Unique project identifier
-   `name: String` — Display name
-   `root_path: PathBuf` — Filesystem root (git repository)
-   `config: ProjectConfig` — From Figment (ADR-025): embedding provider, vector store, VCS settings, policy overrides
-   `created_at: DateTime<Utc>`
-   `archived_at: Option<DateTime<Utc>>`

**Responsibility**:

-   Configuration loading and validation
-   Multi-plan coordination (ensure no conflicts)
-   Operator role assignment (RBAC)

**Lifecycle**:

1.  **Created**: Operator initializes project via MCP `project:init` tool
2.  **Active**: Plans and sessions execute within project scope
3.  **Archived**: No new sessions created; read-only for historical queries

**Concurrency**: Unlimited projects can run independently. No lock required at project level.

#### Plan Entity

**Purpose**: From Beads — logical grouping of tasks by phase (e.g., "Phase 1: Architecture Cleanup", "Phase 2: Git Foundation").

**Fields**:

-   `id: String` — Unique to Beads
-   `project_id: String` — Foreign key to project
-   `phase_name: String` — e.g., "01-architecture-cleanup"
-   `task_ids: Vec<String>` — Beads task IDs in this phase
-   `status: PlanStatus` — Open | InProgress | Closed
-   `created_at: DateTime<Utc>`

**Responsibility**:

-   Grouping tasks by logical phase
-   Tracking phase-level metrics (tasks completed / total)
-   Enforcing phase ordering (implicit: phases ordered by creation date)

**Lifecycle**:

1.  **Open**: Defined in Beads, no active sessions yet
2.  **InProgress**: ≥1 task has active session
3.  **Closed**: All tasks completed; operator marks via MCP

**Integration with Workflow**:

-   Workflow engine reads plans from Beads via `TrackerProvider` (ADR-035)
-   When task transitioned to InProgress, plan automatically transitions to InProgress
-   When last task in plan completed, plan is marked closed

**Concurrency**: Unlimited plans per project. Plans are independent unless tasks have explicit dependencies (rare).

#### Task Entity

**Purpose**: From Beads — atomic unit of work. Entirely managed by Beads; workflow engine only **consumes** task metadata.

**Fields** (from Beads schema):

-   `id: String` — Beads issue ID
-   `plan_id: String` — Belongs to phase
-   `title: String` — Work description
-   `blockers: Vec<String>` — Task IDs that must complete first
-   `status: TaskStatus` — Open | InProgress | PendingReview | Approved | Merged | Completed
-   `created_at, closed_at: DateTime<Utc>`

**Responsibility**:

-   Describing the work to be done (NOT implementing it)
-   Tracking work status via Beads CLI (`bd update <id> --status=in_progress`)
-   Managing task dependencies and blockers

**Lifecycle** (entirely in Beads):

```
Open
  → InProgress          (operator runs `bd update <id> --status=in_progress`)
    → PendingReview     (auto: session transitions to AwaitingOperatorReview)
      → Approved        (operator approves via `operator_decision: Approve`)
        → Merged        (auto: code merged to main)
          → Completed   (operator or automated)
```

**Workflow Integration**:

-   When task status = Open: MCP `project:ready_tasks` returns it
-   Operator picks task → workflow creates Session
-   Session state drives task status updates (no direct task mutations by workflow engine)

**Constraints**:

-   **Read-only in workflow engine**: Workflow NEVER mutates task data directly
-   Task state is single source of truth (stored in Beads)
-   Workflow reads task state; may trigger Beads status update via orchestrator

**Concurrency**: Limited by WIP (Work-in-Progress) policy (ADR-036). Default: max 3 concurrent sessions per plan.

#### Session Entity

**Purpose**: Execution context for a single task by one operator. Encapsulates the entire workflow from start (code changes) to finish (code merged).

**Fields**:

-   `id: String` — UUID, unique session identifier
-   `task_id: String` — Foreign key (1:1 mapping to task, but session can outlive task in error recovery scenarios)
-   `operator_id: String` — Operator making decisions
-   `project_id: String` — Project context
-   `state: WorkflowState` — FSM enum from ADR-034 (Initializing | Ready | Planning | Executing | Verifying | PhaseComplete | Completed | Failed)
-   `state_data: serde_json::Value` — Serialized state context (phase_id, task_id, etc.)
-   `branch_name: String` — Git feature branch (derived from task_id and session_id)
-   `worktree_path: PathBuf` — `.worktrees/{session_id}`
-   `agent_ids: Vec<String>` — Agents active in this session
-   `compensation_plan: CompensationStrategy` — AutoRevert | ManualReview | ApproveAndMerge (from ADR-034)
-   `created_at, started_at, completed_at: DateTime<Utc>`

**Responsibility**:

-   Holding workflow state and transitions (FSM)
-   Coordinating agents to execute task
-   Recording all decisions (operator approval, rejections)
-   Managing compensation if things go wrong

**Lifecycle**:

```
Created                 (operator opens task)
  ↓
Initializing            (context discovery in progress)
  ↓
Ready                   (context snapshot available)
  ↓
Planning                (operator/agents plan work)
  ↓
Executing               (agents run; operator can request intermediate reviews)
  ↓
Verifying               (operator reviews all output)
  ↓ (approval)
AwaitingMerge           (code ready, awaiting approval)
  ↓ (approved)
Merged                  (code merged to main)
  ↓
Completed               (cleanup done)

OR at any point:
  → Failed              (error + compensation triggered)
```

**State Transitions** (from ADR-034):

| From | To | Trigger | Policy Checks | Compensation |
|------|----|---------|----|---|
| Created | Initializing | auto | — | — |
| Initializing | Ready | context_discovered | — | — |
| Ready | Planning | operator_ready | WIP limit, phase open | — |
| Planning | Executing | plan_complete | phase_not_blocked | — |
| Executing | Verifying | work_complete | — | — |
| Verifying | AwaitingMerge | operator_approved | tests pass, reviews OK | — |
| AwaitingMerge | Merged | code_merged_to_main | — | — |
| Merged | Completed | cleanup_done | — | — |
| * | Failed | error / operator_reject | — | ManualReview / ApproveAndMerge |

**Concurrency**: Only 1 session per task. Multiple sessions can run in parallel across different tasks (bounded by WIP limit).

#### Operator Entity

**Purpose**: Human decision-maker. Approves code changes, overrides policies, manages session lifecycle.

**Fields**:

-   `id: String` — User ID (from OIDC or auth system)
-   `name: String` — Display name
-   `email: String` — Email address
-   `roles: Vec<Role>` — Architect | Developer | Reviewer | QA | Admin
-   `permissions: Vec<Permission>` — Read | Write | Approve | Override | Admin
-   `max_active_sessions: usize` — WIP limit per operator (default: 1, allows batching)
-   `active_session_ids: Vec<String>` — Currently assigned sessions

**Responsibility**:

-   Making decisions (approve code, request changes, reject)
-   Overriding policies (with audit logging)
-   Resuming interrupted sessions
-   Managing compensation flow

**Constraints**:

-   **Single decision at a time**: Operator can have multiple assigned sessions, but only processes ONE decision concurrently (implicit bottleneck)
-   **Can't double-approve**: Once a decision is recorded, subsequent calls are idempotent (return same decision)
-   **Can override**: Can approve despite policy failures (requires explicit `override_reason`)

**Lifecycle**:

```
Idle
  ↓ (task opened)
Assigned (has active session)
  ↓
Reviewing (reviewing code)
  ↓
Deciding (choosing approve/reject)
  ↓ (approve)
Approving → back to Assigned/Idle (next session)
  ↓ (reject)
Rejecting → back to Assigned/Idle
```

#### Agent Entity

**Purpose**: AI agents executing work within a session (e.g., code changes, test execution, documentation).

**Fields**:

-   `id: String` — Agent identifier
-   `session_id: String` — Session this agent is part of
-   `agent_type: AgentType` — CodeWriter | Tester | Documenter | CustomAgent
-   `status: AgentStatus` — Queued | Running | Completed | Failed
-   `output: Option<String>` — Captured stdout/stderr
-   `started_at, completed_at: Option<DateTime<Utc>>`

**Responsibility**:

-   Executing assigned work (code changes, tests)
-   Reporting progress to session
-   Contributing to worktree changes

**Constraints**:

-   **Multiple agents per session**: Up to 8 concurrent (configurable)
-   **Shared worktree**: All agents modify same worktree; changes are cumulative
-   **No blocking between agents**: Agents run in parallel; operator or session FSM enforces synchronization points

**Lifecycle**:

```
Queued       (waiting for resources)
  ↓
Running      (executing task)
  ↓
Completed    (output captured)

OR:
  → Failed   (exception, timeout, or cancellation)
```

### 2. State Machines

Three state machines operate at different scopes:

#### 2.1 TaskState (Beads — External)

Managed entirely by Beads. Workflow engine is a **consumer** only.

```
┌─────┐
│ Open│ ← Task created in Beads
└──┬──┘
   │ operator: bd update <id> --status=in_progress
   ▼
┌──────────┐
│InProgress│ ← Operator claims task, workflow session starts
└──┬───────┘
   │ auto: session transitions to Verifying
   ▼
┌─────────────┐
│PendingReview│ ← Code ready for review
└──┬──────────┘
   │ operator_decision: Approve (via MCP project:decide)
   ▼
┌────────┐
│Approved│ ← Ready to merge
└──┬─────┘
   │ auto: git merge, code merged to main
   ▼
┌───────┐
│Merged │ ← Code successfully merged
└──┬────┘
   │ operator: close session or auto-close
   ▼
┌──────────┐
│Completed │ ← Task fully done, session ended
└──────────┘
```

**Beads Transitions** (operators):

-   Open → InProgress: `bd update <id> --status=in_progress`
-   InProgress → PendingReview: auto (triggered by session state change)
-   PendingReview → Approved: `project:decide approve` (MCP)
-   Approved → Merged: `project:decide merge` (MCP)
-   Merged → Completed: `bd close <id>` or auto-close

#### 2.2 SessionState (Workflow FSM — ADR-034)

Managed by `WorkflowEngine` (from ADR-034).

```
┌─────────┐
│ Created │ ← Session instantiated
└────┬────┘
     │ auto (300ms timeout)
     ▼
┌──────────────┐
│Initializing  │ ← Context discovery running
└────┬─────────┘
     │ auto: context_snapshot_ready
     ▼
┌──────┐
│Ready │ ← Context available, waiting for operator signal
└──┬───┘
   │ operator: project:session_start
   │ policy check: WIP limit, phase open
   ▼
┌────────┐
│Planning│ ← Plan phase (operator can provide guidance)
└────┬───┘
   │ auto: agents_ready OR operator: session_execute
   ▼
┌──────────┐
│Executing │ ← Agents running in parallel
└────┬─────┘
   │ operator can request: session_review_checkpoint (intermediate checkpoint)
   │ auto: all_agents_done
   ▼
┌──────────┐
│Verifying │ ← Operator reviews all output, tests, code
└────┬─────┘
   │ operator_decision: Approve
   │ policy check: RequireTests (tests must pass), ReviewApproval
   ▼
┌──────────────┐
│AwaitingMerge │ ← Ready to push to main
└────┬─────────┘
   │ auto (or operator: session_merge): git merge PR
   ▼
┌──────┐
│Merged│ ← Code merged; cleanup in progress
└──┬───┘
   │ auto: cleanup done (remove worktree, branch)
   ▼
┌─────────┐
│Completed│ ← Session fully done
└─────────┘

OR at any point:
  │ error (policy failure, agent exception, operator reject)
  ▼
┌───────┐
│ Failed│ ← Compensation plan invoked
└───┬───┘
    │ compensation: AutoRevert | ManualReview | ApproveAndMerge
    └─→ Completed (after recovery)
```

**Transition Guards** (from ADR-036 — policies):

-   Ready → Planning: WIP limit check, phase not blocked
-   Planning → Executing: All prerequisites satisfied
-   Executing → Verifying: Agents completed (or timeout)
-   Verifying → AwaitingMerge: All policy checks pass (tests, reviews, security scans)
-   AwaitingMerge → Merged: (can be skipped if auto-merge enabled)
-   -   → Failed: At any point if error or operator rejection

#### 2.3 OperatorState (Decision Loop)

Operator progresses through a sequence of decisions:

```
┌─────┐
│Idle │ ← No active sessions assigned
└──┬──┘
   │ session_created (for task operator claimed)
   ▼
┌──────────┐
│Assigned  │ ← Session awaiting operator input
└──┬───────┘
   │ operator opens MCP interface, reviews context
   ▼
┌──────────┐
│Reviewing │ ← Reading code, test output, logs
└──┬───────┘
   │ operator signals ready (project:session_start or project:decide)
   ▼
┌─────────┐
│Deciding │ ← Making choice (approve / request changes / reject)
└──┬──────┘
   │
   ├─→ Approve (project:decide approve)
   │   ├─→ all tests pass?
   │   │   ├─→ Yes: immediately forward to merge
   │   │   └─→ No: return Severity::Warning (don't block)
   │   └─→ back to Assigned/Idle
   │
   ├─→ RequestChanges (project:decide request_changes)
   │   └─→ Agents re-execute from Executing state
   │       └─→ back to Reviewing
   │
   └─→ Reject (project:decide reject)
       ├─→ trigger compensation
       │   ├─→ AutoRevert: `git reset --hard` worktree
       │   ├─→ ManualReview: stay at Failed until operator decides
       │   └─→ ApproveAndMerge: immediately merge (dangerous, logged)
       └─→ back to Assigned/Idle
```

**Constraints**:

-   **Atomic decisions**: Operator can't split a decision (approve partial code)
-   **Idempotent**: Same decision can be submitted twice without side effects
-   **Timeout**: If operator doesn't decide for 72 hours, session auto-fails (configurable, triggers ManualReview compensation)

### 3. Concurrency Model

The execution model supports parallel execution at multiple levels, with explicit boundaries:

#### 3.1 Project Level: **UNLIMITED**

Multiple projects can run independently. No global lock needed.

```
Project A                Project B                Project C
  ├─ Plans               ├─ Plans                ├─ Plans
  ├─ Sessions            ├─ Sessions             ├─ Sessions
  └─ Agents              └─ Agents               └─ Agents
  
(all parallel, no contention)
```

#### 3.2 Plan Level: **UNLIMITED (with ordering constraint)**

Multiple plans per project can run in parallel. However, if explicit phase ordering is enforced, plans must respect it.

**Default**: Assume plans are independent (no ordering). If phase dependencies exist, they are enforced by task dependencies (Beads).

```
Project A
  ├─ Phase 1 (Tasks A1, A2, A3)
  ├─ Phase 2 (Tasks B1, B2)
  └─ Phase 3 (Tasks C1)
  
Plans can overlap:
  T=0:   A1 session starts
  T=1:   A2 session starts
  T=2:   B1 session starts     ← Phase 2 can start before Phase 1 done
  T=3:   A3 session starts
  etc.
```

#### 3.3 Task Level: **LIMITED by WIP Policy (ADR-036)**

Work-in-Progress limit controls max concurrent tasks per plan.

**Default WIP**: 3 concurrent tasks per plan

**Enforcement** (in ContextScout + PolicyGuard):

```rust
let in_progress_count = context.tracker.count_issues(
    plan_id: phase_id,
    status: InProgress
);

if in_progress_count >= config.wip_limit {
    return PolicyResult::Blocked("WIP limit exceeded".to_string());
}
```

If WIP limit reached, next `Ready` → `Planning` transition is blocked until another task completes.

#### 3.4 Session Level: **EXCLUSIVE (1 per task)**

Only 1 session can be active per task. If a session fails/crashes, a new session can be created for the same task (recovery scenario).

**Enforcement** (in WorkflowEngine):

```rust
let existing = session_repo.find_by_task(task_id).await?;
if existing.state != WorkflowState::Completed && existing.state != WorkflowState::Failed {
    return Err("Session already active for this task".into());
}
```

#### 3.5 Agent Level: **MULTIPLE (up to 8 per session)**

Multiple agents can run in parallel within the same session. All modifications are to the same worktree; changes accumulate.

**Bounded by**:

-   Agent pool size (default: 8)
-   System resources (CPU, memory)
-   Session timeout (default: 24 hours)

```
Session A (task_id = beads-123)
  ├─ Agent 1 (CodeWriter)
  │  └─ Modifies: src/foo.rs, tests/foo_test.rs
  │
  ├─ Agent 2 (Documenter)
  │  └─ Modifies: docs/foo.md, README.md
  │
  └─ Agent 3 (Tester)
     └─ Runs: make test (reads both previous modifications)
     
(all 3 run in parallel on same worktree)
```

**Synchronization**:

-   Agents don't synchronize with each other (free-for-all)
-   Session FSM synchronizes agents (waits for all to complete before Verifying)
-   Operator reviews final combined output

#### 3.6 Operator Level: **SEQUENTIAL**

Operator processes decisions one at a time (implicit bottleneck).

**Constraint**: While operator is reviewing session A, other sessions waiting for operator decision must wait.

**Mitigation**: Operator can have multiple sessions assigned; can batch decisions (e.g., review A, review B, then approve both).

**Concurrency Model Summary**:

| Level | Max Concurrent | Bounded By | Lock Required |
|-------|---|---|---|
| Project | ∞ | System resources | No |
| Plan | ∞ | Task dependencies | No |
| Task | 1 (exclusive) | Design | Per-task Mutex |
| Session | Limited by WIP | Policy (default 3 per plan) | Per-session Mutex |
| Agent | 8 | Agent pool size | Per-session (coordinated) |
| Operator | 1 decision at a time | Human speed | Implicit (sequential processing) |

### 4. Git Integration & Worktree Management

Each session gets **exclusive ownership** of a Git worktree, enabling true isolation and parallel execution.

#### 4.1 Worktree Lifecycle

**Naming Convention**:

```
.worktrees/{session_id}
e.g., .worktrees/sess-a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

**Branch Naming Convention**:

```
feature/{task_id}/{session_id}
e.g., feature/beads-123/sess-a1b2c3d4
```

#### 4.2 Lifecycle Stages

**Stage 1: Created (SessionState = Initializing)**

```
Trigger: Session created for task
Action:
  1. Call VcsProvider.create_worktree(branch_name)
     └─ git worktree add .worktrees/{session_id} -b feature/{task_id}/{session_id} origin/main
  2. Clone repository to worktree (from main branch)
  3. Update Session.worktree_path, branch_name in SQLite
  4. State transition: Initializing → Ready
```

**Stage 2: Active (SessionState = Planning | Executing)**

```
Agents modify:
  - src/ files
  - test files
  - docs/

All changes committed to worktree branch:
  git add .
  git commit -m "Feature: {task_title}" --in-worktree
```

**Stage 3: Review (SessionState = Verifying | AwaitingMerge)**

```
Operator reviews:
  - git log {worktree_branch}...main (show commits)
  - git diff main (show changes)
  - test output, lint results

Operator decides:
  - Approve: next stage
  - RequestChanges: agents re-execute
  - Reject: compensation (see below)
```

**Stage 4: Merge (SessionState = AwaitingMerge → Merged)**

```
Actions:
  1. Create PR (GitHub API or manual note)
  2. git merge --ff-only feature/{task_id}/{session_id} -m "Merge: {task_title}"
     (or git merge --no-ff if history preservation needed)
  3. git push origin main
  4. State transition: AwaitingMerge → Merged
```

**Stage 5: Cleanup (SessionState = Merged → Completed)**

```
Actions:
  1. git worktree remove .worktrees/{session_id} --force
  2. git branch -D feature/{task_id}/{session_id}
  3. Delete Session record (or mark archived) in SQLite
  4. State transition: Merged → Completed
```

#### 4.3 Safety Properties

**Isolation**: Each worktree is independent. No git conflicts between concurrent sessions.

```
Session A: feature/beads-123/sess-aaaa
Session B: feature/beads-124/sess-bbbb

Both derived from main, but diverge immediately.
No merge conflicts (different files/features).
```

**Rollback**: If operator rejects, entire worktree discarded.

```
Compensation: AutoRevert
  1. git reset --hard origin/main
  2. Delete all changes in worktree
  3. Session → Failed → Completed
```

**Merge Conflict Avoidance**:

-   If main has moved ahead, rebase worktree branch before merge:

    ```
    git rebase origin/main
    ```

-   If conflicts, operator decides: resolve manually (RequestChanges) or reject (AutoRevert).

#### 4.4 Operator Testing

Operator can test code locally before push:

```
cd .worktrees/{session_id}
make build
make test
cargo clippy
```

Results inform operator decision (Approve/Reject).

### 5. Operator Workflow & Compensation

The operator is the bottleneck and decision-maker. The workflow accommodates three operator failure modes:

#### 5.1 Compensation Strategies (from ADR-034)

**Strategy 1: AutoRevert**

-   Automatic rollback on error
-   Git reset --hard to main
-   Used for agent failures (test failures, syntax errors)
-   Minimal operator involvement
-   Session → Failed → Completed (no recovery)

**Strategy 2: ManualReview**

-   Human operator decides next step
-   Operator reviews error, code, logs
-   Three options:
    1.  **Retry**: Re-run agents from Executing
    2.  **Fix**: Modify code manually, resubmit
    3.  **Abort**: Reject and rollback
-   Used for policy failures, merge conflicts, unclear errors
-   Operator overhead: ~10-30 minutes per incident

**Strategy 3: ApproveAndMerge**

-   Automatic approval and merge (dangerous)
-   Used for non-blocking policies (warnings)
-   Minimal operator involvement
-   Requires explicit audit logging and team review
-   Used only in automated CI/CD pipelines (not for manual workflows)

#### 5.2 Decision Flow

**Operator receives notification**: Session ready for decision (Verifying state)

```
MCP Tool: project:sessions_ready
Returns: [Session { id, task_id, operator_id, state, context }]

Operator reviews:
  1. cd .worktrees/{session_id}
  2. git diff main            (see changes)
  3. git log -5 --oneline     (see commits)
  4. cat /tmp/test_results    (see test output)
  5. grep "FAIL\|ERROR" logs/ (check for problems)
```

**Operator decides**:

```
project:decide <session_id> {
  decision: Approve | RequestChanges | Reject,
  reason: "...",
  override_policies: [...],  // optional, if overriding policies
  compensation: AutoRevert | ManualReview | ApproveAndMerge  // if rejecting
}
```

**Decision 1: Approve**

```
Preconditions:
  - Tests pass (RequireTests policy)
  - No blocker issues (ReviewApproval policy)
  - Optional: security scan passed

Actions:
  1. Transition: Verifying → AwaitingMerge
  2. Store decision in session.compensation_plan
  3. Trigger: project:session_merge (if auto-merge enabled)
  4. Outcome: Code merged, session completed

Override example:
  - Tests failed, but operator overrides: 
    project:decide approve --reason "known failure, non-blocking"
    --override_policies RequireTests
    (stored as audit event for review)
```

**Decision 2: RequestChanges**

```
Preconditions:
  - None (operator can request changes at any time)

Actions:
  1. Store decision in session_decisions table
  2. Agents notified (via message bus)
  3. Transition: Verifying → Executing (agents re-run)
  4. Operator review loop repeats

Example:
  - Operator finds: missing docstring, suboptimal variable name
  - Requests: "Add docstring to X function; rename variable B to C for clarity"
  - Agents re-run with modified instructions
  - Operator reviews again
```

**Decision 3: Reject**

```
Preconditions:
  - Code has blocker issue (security, correctness, etc.)

Actions:
  1. Store decision in session_decisions table
  2. Select compensation strategy based on error type
  3. Transition: Verifying → Failed
  4. Compensation triggered:
     - AutoRevert: reset worktree, end session
     - ManualReview: session stays at Failed, awaits operator next action
     - ApproveAndMerge: (shouldn't happen for reject)
     
Example:
  - Operator finds: security vulnerability
  - Decision: Reject with AutoRevert
  - Action: Worktree discarded
  - Outcome: Session failed, task still Open in Beads
  - Next step: Operator reopens task, creates new session
```

#### 5.3 Timeout Recovery

If operator doesn't decide within 72 hours:

```
Trigger: SessionTimeout event

Actions:
  1. Send notification to operator (escalation)
  2. After 7 days: auto-compensation triggers (ManualReview)
  3. Session stays at Verifying (blocked, awaiting operator)
  4. Manager notified to reassign or close

Configurable timeouts:
  - project.operator_decision_timeout_hours = 72
  - project.auto_escalation_days = 7
```

## Consequences

### Positive Consequences

-   ✅ **Clear entity relationships**: Five entities with defined responsibilities and lifecycle make implementation straightforward
-   ✅ **Type-safe states**: Rust enums + FSM ensure invalid transitions caught at compile time
-   ✅ **Audit trail**: Event log (ADR-037) captures all decisions for compliance and debugging
-   ✅ **Parallel execution**: Tasks, agents, and projects run independently; WIP policy prevents resource exhaustion
-   ✅ **Git isolation**: Worktrees enable safe concurrent development with zero merge conflicts
-   ✅ **Operator control**: Compensation strategies accommodate all failure modes without requiring code changes
-   ✅ **Testability**: Each layer (FSM, policies, context discovery, orchestration) can be tested independently
-   ✅ **Clean Architecture**: Entities are in `mcb-domain`; providers are in `mcb-providers`; use cases in `mcb-application`
-   ✅ **Scalability**: No global locks; concurrency bounded by WIP, operator speed, and system resources

### Negative Consequences

-   ❌ **Complexity**: 5 entity types × 2 state machines × 3 concurrency levels = significant cognitive load for implementers
-   ❌ **Git overhead**: Worktree per session consumes disk space (~500MB per worktree for large repos). ~10 concurrent sessions → 5GB disk overhead. Needs monitoring.
-   ❌ **Policy composition**: Designing policies is hard (AND vs OR vs sequential checks). Needs clear guidelines and templates.
-   ❌ **Event broadcasting**: 3 channels to manage (transitions, decisions, errors). Risk of inconsistent state if not carefully coordinated.
-   ❌ **Operator bottleneck**: Decision-making is sequential; backlog can accumulate if operator is slow or unavailable
-   ❌ **Database transactions**: SQLite concurrency (multiple writers) requires careful transaction design; easy to introduce race conditions
-   ❌ **Testing complexity**: Integration tests must cover FSM transitions × policy combinations × compensation strategies. ~200+ test cases needed.

## Alternatives Considered

### Alternative 1: Stateless Workflow (No SQLite Persistence)

-   **Description**: Keep all state in memory; rely on process restart for recovery (traditional shell script approach)
-   **Pros**: Simpler implementation, no database schema, no concurrency concerns
-   **Cons**: Lost state on crash, no audit trail, no time-travel debugging, impossible to resume long-running tasks
-   **Rejection Reason**: Violates core requirement (session continuity). Chosen in-memory state only for testing/development.

### Alternative 2: Single-Session-Per-Project

-   **Description**: Only one session allowed per project at a time (sequential execution)
-   **Pros**: Eliminates concurrency complexity, no WIP policy needed, simpler Git (no worktrees)
-   **Cons**: Severely limits throughput, projects with multiple independent tasks serialize unnecessarily, operator can't parallelize work
-   **Rejection Reason**: Poor throughput. Chosen WIP-limited concurrency instead.

### Alternative 3: Operator as Central Bottleneck

-   **Description**: All decisions go through a central decision queue (similar to code review tools like Gerrit)
-   **Pros**: Clear audit trail, uniform approval process
-   **Cons**: Single point of failure (if operator unavailable, all sessions block), hard to distribute decisions across teams
-   **Rejection Reason**: Chosen distributed decisions with operator notification instead.

### Alternative 4: Automatic Merge (No Operator Review)

-   **Description**: Skip operator review; merge code immediately after tests pass
-   **Pros**: Eliminates operator bottleneck, fastest deployment
-   **Cons**: No human judgment, risky for production code, violates compliance requirements (audit)
-   **Rejection Reason**: Chosen hybrid: policies can auto-merge (ApproveAndMerge compensation), but require audit override.

## Implementation Notes

### Dependency Map

```
mcb-domain
  ├─ workflows (entities, FSM, errors)
  ├─ ports (ContextScoutProvider, PolicyGuard, VcsProvider, TrackerProvider)
  └─ events (WorkflowEvent, OperatorDecision, CompensationTriggered)

mcb-application
  ├─ workflow_engine (WorkflowService, transition orchestration)
  ├─ policy_service (PolicyGuard implementation)
  ├─ context_service (ContextScout implementation)
  └─ orchestration (MCP handlers → workflow actions)

mcb-infrastructure
  ├─ repositories (SessionRepository, WorkflowEventRepository)
  ├─ providers (VcsProvider impl using git2, TrackerProvider impl using Beads)
  ├─ event_bus (3 channels: transitions, decisions, compensation)
  └─ dill (DI config: register all services)

mcb-server
  └─ mcp_handlers (8 MCP tools: session, project, operator, agent, etc.)
```

### Database Schema (SQLite)

```sql
-- Workflow sessions
CREATE TABLE workflow_sessions (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  operator_id TEXT NOT NULL,
  state TEXT NOT NULL,  -- enum: Initializing, Ready, Planning, ...
  state_data JSONB NOT NULL,  -- context for current state
  branch_name TEXT,
  worktree_path TEXT,
  compensation_plan TEXT,  -- enum
  created_at TIMESTAMP NOT NULL,
  started_at TIMESTAMP,
  completed_at TIMESTAMP,
  FOREIGN KEY (project_id) REFERENCES projects(id),
  UNIQUE (task_id, state != 'Completed')  -- only 1 active session per task
);

-- Workflow transitions (append-only log)
CREATE TABLE workflow_transitions (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  from_state TEXT NOT NULL,
  to_state TEXT NOT NULL,
  trigger TEXT,  -- e.g., "context_discovered", "operator_ready"
  reason TEXT,
  created_at TIMESTAMP NOT NULL,
  FOREIGN KEY (session_id) REFERENCES workflow_sessions(id)
);

-- Operator decisions
CREATE TABLE operator_decisions (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  operator_id TEXT NOT NULL,
  decision TEXT NOT NULL,  -- enum: Approve, RequestChanges, Reject
  reason TEXT,
  override_policies JSONB,
  created_at TIMESTAMP NOT NULL,
  FOREIGN KEY (session_id) REFERENCES workflow_sessions(id)
);

-- Agents running in session
CREATE TABLE session_agents (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  agent_type TEXT NOT NULL,
  status TEXT NOT NULL,  -- Queued, Running, Completed, Failed
  output TEXT,  -- captured stdout/stderr
  started_at TIMESTAMP,
  completed_at TIMESTAMP,
  FOREIGN KEY (session_id) REFERENCES workflow_sessions(id)
);
```

### Implementation Roadmap

**Phase 1: Core Entities & FSM (Weeks 1-2, 40 hours)**

-   Implement `Project`, `Session`, `Operator`, `Agent` entities
-   Implement `WorkflowState` enum + `transition()` logic
-   Implement `SessionRepository` (SQLite CRUD)
-   Write domain tests (50+ test cases)

**Phase 2: Providers & Services (Weeks 3-4, 60 hours)**

-   Implement `VcsProvider` (git2-based worktree management)
-   Implement `ContextScout` (project state discovery)
-   Implement `PolicyGuard` (enforcement of policies)
-   Write provider tests (60+ test cases)

**Phase 3: Orchestration & MCP (Weeks 5-6, 50 hours)**

-   Implement `WorkflowOrchestrator` (MCP integration)
-   Implement event broadcasting (3 channels)
-   Implement operator decision flow (approve/reject/request changes)
-   Write integration tests (90+ test cases)

**Total: ~150–200 hours (5 engineers × 4 weeks)**

### Testing Strategy

-   **Unit tests** (80+): FSM transitions, entity validation, policy evaluation
-   **Integration tests** (60+): Full workflows (task → session → merged), compensation scenarios
-   **E2E tests** (60+): Real git repository, operator decisions, worktree isolation
-   **Concurrency tests** (20+): Race conditions, deadlocks, stale state

**Target coverage**: >85% code coverage for mcb-domain, mcb-application, mcb-infrastructure

### Rollback Plan

If implementation reveals critical issues (e.g., SQLite concurrency problems, policy conflicts):

1.  **Disable at MCP level**: Remove workflow tools from MCP handler, revert to shell scripts
2.  **Keep database**: Leave SQLite data for analysis and migration
3.  **Document lessons learned**: ADR update with failure analysis
4.  **Reassess**: Decide on redesign (alternative: async actor model like Tokio with state machines)

## References

-   [ADR-034: Workflow Core FSM](./034-workflow-core-fsm.md) — State machine design
-   [ADR-035: Context Scout](./035-context-scout.md) — Project state discovery
-   [ADR-036: Enforcement Policies](./036-enforcement-policies.md) — Policy evaluation and guards
-   [ADR-037: Workflow Orchestrator](./037-workflow-orchestrator.md) — MCP integration and orchestration
-   [ADR-029: Hexagonal Architecture with dill](./029-hexagonal-architecture-dill.md) — DI container
-   [ADR-013: Clean Architecture Crate Separation](./013-clean-architecture-crate-separation.md) — Crate boundaries
-   [ADR-025: Figment Configuration Migration](./025-figment-configuration.md) — Configuration loading
-   [Planning Documents](./../planning/COMPREHENSIVE-VALIDATION-REPORT.md) — Exhaustive validation analysis
-   [Critical Analysis](./../planning/ADR-034-037-CRITICAL-ANALYSIS.md) — 5 P1 issues, 8 P2 gaps, recommendations
