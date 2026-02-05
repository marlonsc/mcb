---
adr: 34
title: Workflow Core — Finite State Machine and Persistence
status: PROPOSED
created: 
updated: 2026-02-05
related: [13, 19, 23, 25, 29]
supersedes: [32]
superseded_by: []
implementation_status: Complete
---

# ADR-034: Workflow Core — Finite State Machine and Persistence

## Status

**Proposed** — 2026-02-05

-   **Deciders:** Project team
-   **Supersedes:** [ADR-032](./032-agent-quality-domain-extension.md) (Agent & Quality Domain Extension)
-   **Related:** [ADR-029](./029-hexagonal-architecture-dill.md) (Hexagonal DI), [ADR-023](./023-provider-registration-linkme.md) (linkme), [ADR-025](./025-figment-configuration.md) (Figment), [ADR-019](./019-error-handling-thiserror.md) (thiserror), [ADR-013](./013-clean-architecture-crates.md) (Clean Architecture)
-   **Series:** ADR-034 → [ADR-035](./035-context-scout.md) → [ADR-036](./036-enforcement-policies.md) → [ADR-037](./037-workflow-orchestrator.md)

## Context

MCB currently provides semantic code search (indexing, embedding, vector store). The `oh-my-opencode` workflow layer depends on external shell scripts, markdown skill files, and disconnected tools (Beads CLI, GSD `.planning/` files) that have no shared state, no type safety, and no persistence across sessions.

ADR-032 proposed extending MCB's domain with 24 MCP tools and 9 SQLite tables for agent/quality/project tracking. This ADR supersedes that proposal with a narrower, layered approach: four sequential ADRs (034–037) that each define one architectural concern and expose traits consumed by the next layer.

**This ADR** defines the foundational layer: a finite state machine (FSM) for workflow sessions with SQLite-backed persistence and transition history.

### Problem Statement

1.  **No session continuity** — Workflow state is lost between OpenCode sessions. A resumed session cannot know where the previous session stopped.
2.  **No transition audit** — There is no record of what state transitions occurred, who triggered them, or why they failed.
3.  **No state validation** — Invalid transitions (e.g., executing before planning) are not enforced at the type level.
4.  **No time travel** — Impossible to reconstruct what the workflow state was at a specific point in time.

### Requirements

-   Persist workflow state across process restarts
-   Enforce valid transitions at runtime with clear error messages
-   Log every transition with before/after state and trigger
-   Support state reconstruction from transition history ("time travel")
-   Fit within the existing Clean Architecture crate hierarchy (no new crates)

## Decision

### 1. Enum-Based Manual FSM (Runtime)

Use a hand-written `#[derive(Serialize, Deserialize)]` enum for workflow states with `match`-based transition logic. No external FSM crate.

**Rationale:** Evaluated `statig` (no serde support), `smlang-rs` (macro-generated code less transparent), `sm` (no async, no serde). The enum-based approach provides native serde for SQLite persistence, full `async` compatibility, transparent code for `mcb-validate` architecture rules, and direct compatibility with `Arc<dyn Trait>`.

### 2. Domain Entities

```rust
// mcb-domain/src/entities/workflow.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Workflow session states (12-state production model). Each variant carries context-specific data.
///
/// **Decision (Voted 2026-02-05):** 12-state model approved for production-ready feature set.
/// Includes Suspended, Timeout, Cancelled, and Abandoned states for comprehensive workflow management.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", content = "data")]
pub enum WorkflowState {
    /// Session created, awaiting context discovery.
    Initializing,
    /// Context discovered, ready to plan or execute.
    Ready {
        context_snapshot_id: String,
    },
    /// Planning phase in progress.
    Planning {
        phase_id: String,
    },
    /// Executing tasks within a phase.
    Executing {
        phase_id: String,
        task_id: Option<String>,
    },
    /// Verifying phase completion.
    Verifying {
        phase_id: String,
    },
    /// Phase completed, ready for next phase.
    PhaseComplete {
        phase_id: String,
    },
    /// Session ended normally.
    Completed,
    /// Error state with recovery information.
    Failed {
        error: String,
        recoverable: bool,
    },
    /// Session paused by operator (resume possible).
    Suspended {
        reason: String,
        suspended_at: DateTime<Utc>,
    },
    /// Workflow deadline exceeded.
    Timeout {
        deadline: DateTime<Utc>,
        exceeded_by_ms: u64,
    },
    /// Session cancelled by operator or policy.
    Cancelled {
        reason: String,
        cancelled_by: String,
    },
    /// Session abandoned (no activity for N days, operator approval required to resume).
    Abandoned {
        last_activity: DateTime<Utc>,
        days_inactive: u32,
    },
}

impl std::fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initializing => write!(f, "initializing"),
            Self::Ready { .. } => write!(f, "ready"),
            Self::Planning { .. } => write!(f, "planning"),
            Self::Executing { .. } => write!(f, "executing"),
            Self::Verifying { .. } => write!(f, "verifying"),
            Self::PhaseComplete { .. } => write!(f, "phase_complete"),
            Self::Completed => write!(f, "completed"),
            Self::Failed { .. } => write!(f, "failed"),
            Self::Suspended { .. } => write!(f, "suspended"),
            Self::Timeout { .. } => write!(f, "timeout"),
            Self::Cancelled { .. } => write!(f, "cancelled"),
            Self::Abandoned { .. } => write!(f, "abandoned"),
        }
    }
}

/// Events that trigger state transitions (extended for 12-state model).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "trigger", content = "data")]
pub enum TransitionTrigger {
    // Original triggers (7-state)
    ContextDiscovered { context_snapshot_id: String },
    StartPlanning { phase_id: String },
    StartExecution { phase_id: String },
    ClaimTask { task_id: String },
    CompleteTask { task_id: String },
    StartVerification,
    VerificationPassed,
    VerificationFailed { reason: String },
    CompletePhase,
    EndSession,
    Error { message: String },
    Recover,
    
    // New triggers (extended states)
    Suspend { reason: String },                    // → Suspended
    Resume,                                        // Suspended → Planning|Executing
    TimeoutDetected { deadline: DateTime<Utc> },  // → Timeout
    Cancel { reason: String, by: String },        // → Cancelled
    MarkAbandoned { days_inactive: u32 },         // → Abandoned (auto)
}

/// Recorded transition with full audit context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
    pub session_id: String,
    pub from_state: WorkflowState,
    pub to_state: WorkflowState,
    pub trigger: TransitionTrigger,
    pub guard_result: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Workflow session entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSession {
    pub id: String,
    pub project_id: String,
    pub current_state: WorkflowState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 2.1 Database Provider Abstraction

Rather than coupling directly to SQLite, the workflow engine depends on an abstract `DatabaseProvider` trait that enables multiple backend implementations (SQLite MVP → PostgreSQL Phase 2 → other backends).

**Rationale:** Database independence allows migration between backends without refactoring the workflow domain. Using a provider trait aligns with ADR-029 (dill dependency injection) and ADR-023 (linkme provider registration), enabling compile-time discovery of database implementations.

**Port Trait Definition:**

```rust
// mcb-domain/src/ports/database_provider.rs

use crate::entities::workflow::{Transition, WorkflowSession, WorkflowState};
use crate::errors::WorkflowError;
use async_trait::async_trait;

/// Session filter criteria for queries.
#[derive(Debug, Clone)]
pub struct SessionFilter {
    pub project_id: Option<String>,
    pub state: Option<WorkflowState>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Database provider trait for workflow persistence.
///
/// Abstracts SQLite, PostgreSQL, or other backends. Registered via linkme.
/// Consumed by WorkflowEngine implementations in mcb-providers.
#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    /// Create a new workflow session in the database.
    async fn create_session(
        &self,
        session: &WorkflowSession,
    ) -> Result<(), WorkflowError>;

    /// Update session state atomically.
    async fn update_session(
        &self,
        id: &str,
        state: &WorkflowState,
    ) -> Result<(), WorkflowError>;

    /// Record a state transition in the audit log (append-only).
    async fn record_transition(
        &self,
        transition: &Transition,
    ) -> Result<(), WorkflowError>;

    /// Record a workflow event in the immutable log.
    async fn record_event(
        &self,
        event: &WorkflowEvent,
    ) -> Result<(), WorkflowError>;

    /// Find sessions matching filter criteria.
    async fn find_sessions(
        &self,
        filter: SessionFilter,
    ) -> Result<Vec<WorkflowSession>, WorkflowError>;

    /// Retrieve transition history for a session (newest first).
    async fn get_transitions(
        &self,
        session_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Transition>, WorkflowError>;

    /// Reconstruct session state at a specific timestamp (time travel).
    async fn state_at_timestamp(
        &self,
        session_id: &str,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<WorkflowState>, WorkflowError>;
}

/// Workflow event logged in append-only event store.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum WorkflowEvent {
    StateTransition {
        session_id: String,
        from_state: WorkflowState,
        to_state: WorkflowState,
        trigger: TransitionTrigger,
    },
    CompensationAction {
        session_id: String,
        action: CompensationAction,
    },
    GuardEvaluation {
        session_id: String,
        policy: String,
        result: bool,
    },
    Error {
        session_id: String,
        message: String,
    },
}
```

**Location:** `mcb-domain/src/ports/database_provider.rs`

**Provider Registration (linkme):**

```rust
// mcb-application/src/registry/database.rs

use mcb_domain::ports::database_provider::DatabaseProvider;
use std::sync::Arc;

pub struct DatabaseProviderEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub factory: fn(&figment::Figment) -> Result<Arc<dyn DatabaseProvider>, Box<dyn std::error::Error + Send + Sync>>,
}

#[linkme::distributed_slice]
pub static DATABASE_PROVIDERS: [DatabaseProviderEntry] = [..];
```

**References:**

-   [ADR-029: Hexagonal Architecture with dill](./029-hexagonal-architecture-dill.md) — Handle-based DI pattern
-   [ADR-023: Provider Registration with linkme](./023-provider-registration-linkme.md) — Compile-time plugin discovery

---

### 2.2 Operator Ownership & Compensation Model

MCB workflows are not autonomous agents — they operate under human supervision. Each workflow session is owned by an operator (human or bot), associated with a Beads task, and embedded within a project context. The compensation model is hybrid: automatic rollback for safe operations, manual review for high-risk changes.

**Extended WorkflowSession Entity:**

```rust
// mcb-domain/src/entities/workflow.rs (extended)

/// Agent execution handle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHandle {
    pub id: String,              // Agent instance ID
    pub name: String,            // Agent type (e.g., "claude-code")
    pub status: AgentStatus,     // Running, paused, completed, failed
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Running,
    Paused { reason: String },
    Completed { output: Option<String> },
    Failed { error: String },
}

/// Compensation strategy for session failures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompensationPlan {
    /// Automatic rollback via git reset/revert. No operator intervention.
    AutoRevert {
        target_branch: String,
    },
    /// Operator reviews and decides: commit, amend, revert, or manual fix.
    ManualReview {
        reason: String,
    },
    /// Automatic attempt to merge PR if all CI checks pass.
    ApproveAndMerge {
        pr_url: String,
        auto_merge_enabled: bool,
    },
}

/// Compensation action executed during recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationAction {
    pub id: String,
    pub session_id: String,
    pub plan: CompensationPlan,
    pub action_type: CompensationActionType,
    pub executed_at: DateTime<Utc>,
    pub result: CompensationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompensationActionType {
    GitReset { target_ref: String },
    GitRevert { commit_hash: String },
    PRMerge { pr_id: String },
    ManualReviewNeeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompensationResult {
    Success,
    Pending { reason: String },
    Failed { error: String },
}

/// Extended workflow session with operator and compensation context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSession {
    pub id: String,
    pub project_id: String,           // GSD project (mcb-workspace)
    pub operator_id: String,          // Human operator or bot (e.g., "claude", "user-123")
    pub task_id: String,              // Beads task ID for tracking
    pub current_state: WorkflowState,
    pub agents: Vec<AgentHandle>,     // Parallel agents executing subtasks
    pub branch: String,               // Git branch or worktree for this session
    pub compensation: CompensationPlan, // Recovery strategy
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Compensation Semantics:**

-   **AutoRevert**: Session uses a feature branch. On error, `git reset --hard` to safe commit. Fast, reversible. Best for exploratory work.
-   **ManualReview**: Session pauses on error. Operator logs in, reviews git diff, decides: amend, revert, or fix manually. Slow but safest.
-   **ApproveAndMerge**: Session attempts to merge PR if CI passes. If CI fails, escalate to ManualReview. Best for auto-commit PRs.

**Location:** `mcb-domain/src/entities/workflow.rs` (extended), `mcb-domain/src/entities/compensation.rs` (new module)

---

### 2.3 Hybrid Transaction Model

The workflow engine uses **two complementary persistence layers**: per-operation ACID transactions (SQLite) + append-only event log (immutable audit trail). This hybrid approach provides both ACID compliance and unbounded temporal history.

**Hybrid Transaction Pattern:**

1.  **Per-Operation Transactions**: Every `transition()` call wraps read + validate + write in a SQLite transaction (10-20ms per operation).
   -   Ensures no lost updates if multiple sessions compete for the same resource.
   -   Provides rollback on validation failure.

2.  **Append-Only Event Log**: After every transition, write immutable event to `workflow_events` table.
   -   Never updated or deleted — only INSERT.
   -   Enables time-travel queries without replaying mutations.
   -   Supports compliance audits and post-mortem analysis.

**SQL Schema:**

```sql
-- State table: mutable, current state only
CREATE TABLE IF NOT EXISTS workflow_sessions (
    id                TEXT PRIMARY KEY,
    operator_id       TEXT NOT NULL,                    -- Human or bot
    task_id           TEXT NOT NULL,                    -- Beads task reference
    project_id        TEXT NOT NULL,                    -- Project context
    branch            TEXT NOT NULL,                    -- Git branch/worktree
    current_state     TEXT NOT NULL,                    -- Display name: "initializing", "ready", etc.
    state_data        TEXT NOT NULL,                    -- JSON: full WorkflowState serde
    compensation_plan TEXT NOT NULL,                    -- JSON: CompensationPlan serde
    created_at        INTEGER NOT NULL,                 -- Unix timestamp
    updated_at        INTEGER NOT NULL
);

CREATE INDEX idx_workflow_sessions_project
    ON workflow_sessions(project_id);
CREATE INDEX idx_workflow_sessions_operator
    ON workflow_sessions(operator_id);
CREATE INDEX idx_workflow_sessions_task
    ON workflow_sessions(task_id);
CREATE INDEX idx_workflow_sessions_state
    ON workflow_sessions(current_state);

-- Append-only event log: immutable history
CREATE TABLE IF NOT EXISTS workflow_events (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL REFERENCES workflow_sessions(id),
    event_type      TEXT NOT NULL,                      -- StateTransition, CompensationAction, Error, etc.
    from_state      TEXT,                               -- Display name (nullable for non-transition events)
    to_state        TEXT,                               -- Display name
    trigger         TEXT,                               -- JSON: full TransitionTrigger serde
    data            TEXT NOT NULL,                      -- JSON: full event payload
    timestamp       INTEGER NOT NULL                    -- Unix timestamp (immutable)
);

CREATE INDEX idx_workflow_events_session
    ON workflow_events(session_id, timestamp);
CREATE INDEX idx_workflow_events_type
    ON workflow_events(event_type);

-- Compensation log: tracks recovery actions
CREATE TABLE IF NOT EXISTS workflow_compensations (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL REFERENCES workflow_sessions(id),
    plan            TEXT NOT NULL,                      -- JSON: CompensationPlan serde
    action_type     TEXT NOT NULL,                      -- GitReset, GitRevert, PRMerge, etc.
    result          TEXT NOT NULL,                      -- JSON: CompensationResult serde
    executed_at     INTEGER NOT NULL
);

CREATE INDEX idx_workflow_compensations_session
    ON workflow_compensations(session_id);
```

**Rationale for Hybrid Model:**

| Concern | Per-Operation TX | Append-Only Log | Coverage |
|---------|------------------|-----------------|----------|
| **Consistency** | ✅ ACID per-operation | Immutable writes only | Complete |
| **Durability** | ✅ WAL mode | ✅ INSERT-only, no rewrites | Complete |
| **Isolation** | ✅ SQLite serialization | N/A (read-only) | Complete |
| **Auditability** | Limited (no history) | ✅ Full history | Complete |
| **Time-Travel** | ❌ Lost on update | ✅ Replay events | Complete |
| **Compliance** | ✅ Current state | ✅ Immutable trail | Complete |

**Time-Travel Implementation:**

```rust
// mcb-providers/src/workflow/sqlite_workflow.rs

impl WorkflowEngine for SqliteWorkflowEngine {
    async fn state_at(
        &self,
        session_id: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<WorkflowState, WorkflowError> {
        // 1. Fetch initial state from session creation
        let initial_state = sqlx::query_scalar::<_, String>(
            "SELECT state_data FROM workflow_sessions WHERE id = ?"
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        let mut state: WorkflowState = serde_json::from_str(&initial_state)
            .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        // 2. Replay all transitions up to timestamp
        let events = sqlx::query_as::<_, (String, String)>(
            "SELECT trigger, event_type FROM workflow_events
             WHERE session_id = ? AND timestamp <= ?
             ORDER BY timestamp ASC"
        )
        .bind(session_id)
        .bind(timestamp.timestamp())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        for (trigger_json, event_type) in events {
            if event_type == "StateTransition" {
                let trigger: TransitionTrigger = serde_json::from_str(&trigger_json)
                    .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

                // Create temporary session for transition validation
                let mut tmp = WorkflowSession {
                    id: session_id.to_string(),
                    current_state: state.clone(),
                    ..Default::default()
                };
                tmp.try_transition(trigger)?;
                state = tmp.current_state;
            }
        }

        Ok(state)
    }
}
```

**Location:** `mcb-domain/src/ports/database_provider.rs` (schema + trait definitions), `mcb-providers/src/workflow/sqlite_workflow.rs` (implementation)

---

### 3. Transition Matrix

Valid transitions are enforced at runtime. Invalid transitions return `WorkflowError::InvalidTransition`.

```
From \ Trigger         │ CtxDisc │ StartPlan │ StartExec │ ClaimTask │ ComplTask │ StartVer │ VerPass │ VerFail │ CompPhase │ EndSess │ Error │ Recover
───────────────────────┼─────────┼───────────┼───────────┼───────────┼───────────┼──────────┼─────────┼─────────┼───────────┼─────────┼───────┼────────
Initializing           │ Ready   │     ✗     │     ✗     │     ✗     │     ✗     │    ✗     │    ✗    │    ✗    │     ✗     │ Compl   │ Fail  │   ✗
Ready                  │    ✗    │ Planning  │ Executing │     ✗     │     ✗     │    ✗     │    ✗    │    ✗    │     ✗     │ Compl   │ Fail  │   ✗
Planning               │    ✗    │     ✗     │ Executing │     ✗     │     ✗     │    ✗     │    ✗    │    ✗    │     ✗     │ Compl   │ Fail  │   ✗
Executing              │    ✗    │     ✗     │     ✗     │ Executing │ Executing │ Verify   │    ✗    │    ✗    │     ✗     │ Compl   │ Fail  │   ✗
Verifying              │    ✗    │     ✗     │     ✗     │     ✗     │     ✗     │    ✗     │ PhComp  │ Exec    │     ✗     │ Compl   │ Fail  │   ✗
PhaseComplete          │    ✗    │ Planning  │ Executing │     ✗     │     ✗     │    ✗     │    ✗    │    ✗    │  Compl    │ Compl   │ Fail  │   ✗
Failed (recoverable)   │    ✗    │     ✗     │     ✗     │     ✗     │     ✗     │    ✗     │    ✗    │    ✗    │     ✗     │ Compl   │   ✗   │ Ready
Failed (unrecoverable) │    ✗    │     ✗     │     ✗     │     ✗     │     ✗     │    ✗     │    ✗    │    ✗    │     ✗     │ Compl   │   ✗   │   ✗
Completed              │    ✗    │     ✗     │     ✗     │     ✗     │     ✗     │    ✗     │    ✗    │    ✗    │     ✗     │   ✗     │   ✗   │   ✗
```

### 4. Transition Implementation

```rust
// mcb-providers/src/workflow/transitions.rs

impl WorkflowSession {
    /// Attempt a state transition. Returns error if transition is invalid.
    pub fn try_transition(
        &mut self,
        trigger: TransitionTrigger,
    ) -> Result<Transition, WorkflowError> {
        let from_state = self.current_state.clone();

        let to_state = match (&self.current_state, &trigger) {
            // Initializing
            (WorkflowState::Initializing, TransitionTrigger::ContextDiscovered { context_snapshot_id }) => {
                WorkflowState::Ready { context_snapshot_id: context_snapshot_id.clone() }
            }

            // Ready → Planning or Executing
            (WorkflowState::Ready { .. }, TransitionTrigger::StartPlanning { phase_id }) => {
                WorkflowState::Planning { phase_id: phase_id.clone() }
            }
            (WorkflowState::Ready { .. }, TransitionTrigger::StartExecution { phase_id }) => {
                WorkflowState::Executing { phase_id: phase_id.clone(), task_id: None }
            }

            // Planning → Executing
            (WorkflowState::Planning { .. }, TransitionTrigger::StartExecution { phase_id }) => {
                WorkflowState::Executing { phase_id: phase_id.clone(), task_id: None }
            }

            // Executing → Executing (claim/complete task) or Verifying
            (WorkflowState::Executing { phase_id, .. }, TransitionTrigger::ClaimTask { task_id }) => {
                WorkflowState::Executing { phase_id: phase_id.clone(), task_id: Some(task_id.clone()) }
            }
            (WorkflowState::Executing { phase_id, .. }, TransitionTrigger::CompleteTask { .. }) => {
                WorkflowState::Executing { phase_id: phase_id.clone(), task_id: None }
            }
            (WorkflowState::Executing { phase_id, .. }, TransitionTrigger::StartVerification) => {
                WorkflowState::Verifying { phase_id: phase_id.clone() }
            }

            // Verifying → PhaseComplete or back to Executing
            (WorkflowState::Verifying { phase_id, .. }, TransitionTrigger::VerificationPassed) => {
                WorkflowState::PhaseComplete { phase_id: phase_id.clone() }
            }
            (WorkflowState::Verifying { phase_id, .. }, TransitionTrigger::VerificationFailed { .. }) => {
                WorkflowState::Executing { phase_id: phase_id.clone(), task_id: None }
            }

            // PhaseComplete → next Planning/Executing or Completed
            (WorkflowState::PhaseComplete { .. }, TransitionTrigger::StartPlanning { phase_id }) => {
                WorkflowState::Planning { phase_id: phase_id.clone() }
            }
            (WorkflowState::PhaseComplete { .. }, TransitionTrigger::StartExecution { phase_id }) => {
                WorkflowState::Executing { phase_id: phase_id.clone(), task_id: None }
            }
            (WorkflowState::PhaseComplete { .. }, TransitionTrigger::CompletePhase) => {
                WorkflowState::Completed
            }

            // Failed (recoverable) → Ready
            (WorkflowState::Failed { recoverable: true, .. }, TransitionTrigger::Recover) => {
                WorkflowState::Ready { context_snapshot_id: String::new() }
            }

            // Any state → Failed (on Error trigger)
            (state, TransitionTrigger::Error { message }) if !matches!(state, WorkflowState::Completed | WorkflowState::Failed { .. }) => {
                WorkflowState::Failed { error: message.clone(), recoverable: true }
            }

            // Any non-terminal state → Completed (on EndSession)
            (state, TransitionTrigger::EndSession) if !matches!(state, WorkflowState::Completed) => {
                WorkflowState::Completed
            }

            // Invalid transition
            (from, trigger) => {
                return Err(WorkflowError::InvalidTransition {
                    from: from.to_string(),
                    trigger: format!("{trigger:?}"),
                });
            }
        };

        let transition = Transition {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: self.id.clone(),
            from_state,
            to_state: to_state.clone(),
            trigger,
            guard_result: None,
            timestamp: Utc::now(),
        };

        self.current_state = to_state;
        self.updated_at = Utc::now();

        Ok(transition)
    }
}
```

### 5. Port Trait

```rust
// mcb-domain/src/ports/providers/workflow.rs

use crate::entities::workflow::{
    Transition, TransitionTrigger, WorkflowSession, WorkflowState,
};
use crate::errors::WorkflowError;

/// Port for the workflow state machine engine.
///
/// Implementations handle persistence (SQLite) and transition validation.
/// Consumed by ADR-037 orchestrator via `Arc<dyn WorkflowEngine>`.
#[async_trait::async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// Create a new workflow session.
    async fn create_session(
        &self,
        project_id: &str,
    ) -> Result<WorkflowSession, WorkflowError>;

    /// Get current state of a session.
    async fn current_state(
        &self,
        session_id: &str,
    ) -> Result<WorkflowState, WorkflowError>;

    /// Attempt a state transition. Persists atomically.
    async fn transition(
        &self,
        session_id: &str,
        trigger: TransitionTrigger,
    ) -> Result<Transition, WorkflowError>;

    /// Transition history for a session (newest first).
    async fn history(
        &self,
        session_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Transition>, WorkflowError>;

    /// Reconstruct state at a specific point in time.
    async fn state_at(
        &self,
        session_id: &str,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<WorkflowState, WorkflowError>;

    /// List active (non-completed, non-failed) sessions.
    async fn active_sessions(&self) -> Result<Vec<WorkflowSession>, WorkflowError>;
}
```

### 6. Error Types

```rust
// mcb-domain/src/errors/workflow.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Invalid transition from '{from}' via trigger '{trigger}'")]
    InvalidTransition { from: String, trigger: String },

    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Persistence error: {message}")]
    Persistence { message: String },

    #[error("Guard policy violated: {message}")]
    PolicyViolation { message: String },

    #[error("Context discovery failed: {message}")]
    ContextError { message: String },

    #[error("Session already completed: {session_id}")]
    SessionCompleted { session_id: String },
}
```

### 7. SQLite Persistence Schema

```sql
-- Workflow session state
CREATE TABLE IF NOT EXISTS workflow_sessions (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL,
    state       TEXT NOT NULL,         -- Display name: "initializing", "ready", etc.
    state_data  TEXT NOT NULL,         -- JSON: full WorkflowState serde
    created_at  INTEGER NOT NULL,      -- Unix timestamp (project convention)
    updated_at  INTEGER NOT NULL
);

CREATE INDEX idx_workflow_sessions_project
    ON workflow_sessions(project_id);
CREATE INDEX idx_workflow_sessions_state
    ON workflow_sessions(state);

-- Transition audit log (append-only)
CREATE TABLE IF NOT EXISTS workflow_transitions (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES workflow_sessions(id),
    from_state  TEXT NOT NULL,         -- Display name
    to_state    TEXT NOT NULL,         -- Display name
    trigger     TEXT NOT NULL,         -- JSON: full TransitionTrigger serde
    guard_result TEXT,                 -- JSON: PolicyResult if guards were evaluated
    created_at  INTEGER NOT NULL
);

CREATE INDEX idx_workflow_transitions_session
    ON workflow_transitions(session_id, created_at);
```

### 8. SQLite Provider Implementation (Skeleton)

```rust
// mcb-providers/src/workflow/sqlite_workflow.rs

use mcb_domain::ports::providers::workflow::WorkflowEngine;
use sqlx::SqlitePool;

pub struct SqliteWorkflowEngine {
    pool: SqlitePool,
}

#[async_trait::async_trait]
impl WorkflowEngine for SqliteWorkflowEngine {
    async fn create_session(
        &self,
        project_id: &str,
    ) -> Result<WorkflowSession, WorkflowError> {
        let session = WorkflowSession {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            current_state: WorkflowState::Initializing,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let state_data = serde_json::to_string(&session.current_state)
            .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        sqlx::query(
            "INSERT INTO workflow_sessions (id, project_id, state, state_data, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&session.id)
        .bind(&session.project_id)
        .bind(session.current_state.to_string())
        .bind(&state_data)
        .bind(session.created_at.timestamp())
        .bind(session.updated_at.timestamp())
        .execute(&self.pool)
        .await
        .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        Ok(session)
    }

    async fn transition(
        &self,
        session_id: &str,
        trigger: TransitionTrigger,
    ) -> Result<Transition, WorkflowError> {
        // Load session, apply transition, persist atomically
        let mut tx = self.pool.begin().await
            .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        // 1. Load current session
        let row = sqlx::query("SELECT state_data FROM workflow_sessions WHERE id = ?")
            .bind(session_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?
            .ok_or_else(|| WorkflowError::SessionNotFound {
                session_id: session_id.to_string(),
            })?;

        let state_json: String = row.get("state_data");
        let current_state: WorkflowState = serde_json::from_str(&state_json)
            .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        // 2. Build in-memory session and apply transition
        let mut session = WorkflowSession {
            id: session_id.to_string(),
            project_id: String::new(),
            current_state,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let transition = session.try_transition(trigger)?;

        // 3. Persist new state
        let new_state_data = serde_json::to_string(&session.current_state)
            .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        sqlx::query(
            "UPDATE workflow_sessions SET state = ?, state_data = ?, updated_at = ? WHERE id = ?"
        )
        .bind(session.current_state.to_string())
        .bind(&new_state_data)
        .bind(session.updated_at.timestamp())
        .bind(session_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        // 4. Log transition
        let trigger_json = serde_json::to_string(&transition.trigger)
            .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        sqlx::query(
            "INSERT INTO workflow_transitions (id, session_id, from_state, to_state, trigger, guard_result, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&transition.id)
        .bind(session_id)
        .bind(transition.from_state.to_string())
        .bind(transition.to_state.to_string())
        .bind(&trigger_json)
        .bind(&transition.guard_result)
        .bind(transition.timestamp.timestamp())
        .execute(&mut *tx)
        .await
        .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        // 5. Commit
        tx.commit().await
            .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        Ok(transition)
    }

    async fn state_at(
        &self,
        session_id: &str,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<WorkflowState, WorkflowError> {
        // Reconstruct state by replaying transitions up to timestamp
        let rows = sqlx::query(
            "SELECT trigger FROM workflow_transitions
             WHERE session_id = ? AND created_at <= ?
             ORDER BY created_at ASC"
        )
        .bind(session_id)
        .bind(timestamp.timestamp())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;

        let mut session = WorkflowSession {
            id: session_id.to_string(),
            project_id: String::new(),
            current_state: WorkflowState::Initializing,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        for row in rows {
            let trigger_json: String = row.get("trigger");
            let trigger: TransitionTrigger = serde_json::from_str(&trigger_json)
                .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;
            session.try_transition(trigger)?;
        }

        Ok(session.current_state)
    }

    // ... remaining methods follow same pattern
}
```

### 9. Provider Registration (linkme)

```rust
// mcb-application/src/registry/workflow.rs

use mcb_domain::ports::providers::workflow::WorkflowEngine;
use std::sync::Arc;

pub struct WorkflowProviderEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub factory: fn(&figment::Figment) -> Result<Arc<dyn WorkflowEngine>, Box<dyn std::error::Error + Send + Sync>>,
}

#[linkme::distributed_slice]
pub static WORKFLOW_PROVIDERS: [WorkflowProviderEntry] = [..];
```

```rust
// mcb-providers/src/workflow/sqlite_workflow.rs

#[linkme::distributed_slice(WORKFLOW_PROVIDERS)]
static SQLITE_WORKFLOW: WorkflowProviderEntry = WorkflowProviderEntry {
    name: "sqlite",
    description: "SQLite-backed workflow state machine with transition history",
    factory: sqlite_workflow_factory,
};

fn sqlite_workflow_factory(
    config: &figment::Figment,
) -> Result<Arc<dyn WorkflowEngine>, Box<dyn std::error::Error + Send + Sync>> {
    let workflow_config: WorkflowConfig = config.extract_inner("workflow")?;
    let pool = SqlitePool::connect_lazy(&workflow_config.database_url)?;
    Ok(Arc::new(SqliteWorkflowEngine { pool }))
}
```

### 10. Module Locations

| Crate | Path | Content |
|-------|------|---------|
| `mcb-domain` | `src/entities/workflow.rs` | `WorkflowState`, `TransitionTrigger`, `Transition`, `WorkflowSession` |
| `mcb-domain` | `src/ports/providers/workflow.rs` | `WorkflowEngine` trait |
| `mcb-domain` | `src/errors/workflow.rs` | `WorkflowError` enum |
| `mcb-application` | `src/registry/workflow.rs` | `WORKFLOW_PROVIDERS` slice, `WorkflowProviderEntry` |
| `mcb-providers` | `src/workflow/mod.rs` | Module root |
| `mcb-providers` | `src/workflow/sqlite_workflow.rs` | `SqliteWorkflowEngine` + linkme registration |
| `mcb-providers` | `src/workflow/transitions.rs` | `try_transition()` logic |
| `mcb-infrastructure` | `src/config/workflow.rs` | `WorkflowConfig` (Figment) |

## Refinements (ADR-034 Phase 2)

### Refinement 1: Database Provider Pattern

**Rationale**: Workflow state persistence requires abstraction to support multiple backends (SQLite for development, PostgreSQL for production). This refinement introduces a `DatabaseProvider` port following the established provider pattern (ADR-029, ADR-023).

**Port Definition** (`mcb-domain/src/ports/providers/database.rs`):

```rust
use async_trait::async_trait;
use sqlx::sqlite::SqlitePool;

pub struct DatabaseConnectionConfig {
    pub url: String,
    pub max_connections: u32,
    pub enable_wal: bool,  // SQLite WAL mode for concurrent reads
}

#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    /// Get connection pool (implementation-specific)
    async fn get_pool(&self) -> Result<SqlitePool, WorkflowError>;

    /// Execute DDL (schema creation) with idempotency guarantee
    async fn ensure_schema(&self) -> Result<(), WorkflowError>;

    /// Health check
    async fn health_check(&self) -> Result<(), WorkflowError>;
}
```

**Implementation** (`mcb-providers/src/database/sqlite.rs`):

```rust
#[linkme::distributed_slice(DATABASE_PROVIDERS)]
static SQLITE_DB: DatabaseProviderEntry = DatabaseProviderEntry {
    name: "sqlite",
    description: "SQLite workflow database with WAL mode",
    factory: sqlite_db_factory,
};

async fn sqlite_db_factory(config: &Figment) -> Result<Arc<dyn DatabaseProvider>> {
    let db_config: DatabaseConnectionConfig = config.extract_inner("workflow.database")?;
    
    let pool = SqlitePool::connect_lazy(&db_config.url)?;
    pool.acquire().await?;  // Verify connection
    
    // Enable WAL mode and transaction isolation
    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA transaction_isolation = DEFERRED;")
        .execute(&pool)
        .await?;
    
    Ok(Arc::new(SqliteDatabase { pool }))
}
```

**Registration**: Providers auto-register via linkme into `mcb-application/src/registry/database.rs::DATABASE_PROVIDERS`.

---

### Refinement 2: Compensation and Rollback Logic

**Problem**: When a workflow transitions fail during execution (e.g., task fails verification → rollback to Executing), there must be a clear strategy for compensating side effects.

**Classification**: MCB workflows operate under human supervision — not autonomous agents. Compensation is **hybrid**:
- **Automatic**: Safe operations (in-memory state, git revert)
- **Manual**: High-risk operations (external API calls, database mutations) → prompt operator for approval

**Compensation Entity** (`mcb-domain/src/entities/workflow.rs`):

```rust
/// Side effect produced during task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEffect {
    pub task_id: String,
    pub effect_type: EffectType,
    pub timestamp: DateTime<Utc>,
    pub reversible: bool,  // Can this effect be rolled back?
    pub compensation_required: bool,  // Needs operator approval?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EffectType {
    GitCommit { hash: String, message: String },
    FileModification { path: String, old_hash: String, new_hash: String },
    ExternalApiCall { service: String, id: String },
    DatabaseMutation { table: String, operation: String },
}

impl ExecutionEffect {
    /// Compute compensation action (if reversible)
    pub fn compensation(&self) -> Option<CompensationAction> {
        match &self.effect_type {
            EffectType::GitCommit { hash, .. } => {
                Some(CompensationAction::GitRevert { hash: hash.clone() })
            }
            EffectType::FileModification { path, old_hash, .. } => {
                Some(CompensationAction::RestoreFile { path: path.clone(), hash: old_hash.clone() })
            }
            EffectType::ExternalApiCall { .. } | EffectType::DatabaseMutation { .. } => {
                None  // Requires manual intervention
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum CompensationAction {
    GitRevert { hash: String },
    RestoreFile { path: String, hash: String },
    ManualReview { reason: String, operator_decision_required: bool },
}
```

**Transition with Compensation** (`mcb-providers/src/workflow/transitions.rs`):

```rust
pub async fn transition_with_compensation(
    session: &mut WorkflowSession,
    trigger: TransitionTrigger,
    effects: Vec<ExecutionEffect>,
) -> Result<Transition> {
    match transition(&mut session.clone(), trigger.clone()) {
        Ok(transition) => Ok(transition),
        Err(WorkflowError::VerificationFailed { reason }) => {
            // Verification failed → compute compensations
            let mut compensations = Vec::new();
            let mut manual_needed = false;
            
            for effect in effects {
                if let Some(comp) = effect.compensation() {
                    compensations.push(comp);
                } else if effect.compensation_required {
                    manual_needed = true;
                }
            }
            
            if manual_needed {
                // Transition to ManualReview state (requires operator decision)
                session.current_state = WorkflowState::Failed {
                    error: reason,
                    recoverable: true,  // Operator can decide recovery
                };
            } else {
                // Auto-execute compensations
                for comp in compensations {
                    execute_compensation(comp).await?;
                }
                // Re-attempt transition
                transition(session, TransitionTrigger::Recover)?;
            }
            
            Err(WorkflowError::VerificationFailed { reason })
        }
        Err(e) => Err(e),
    }
}
```

**Schema** (`workflow_effects` table):

```sql
CREATE TABLE workflow_effects (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    effect_type TEXT NOT NULL,  -- JSON serialized EffectType
    reversible BOOLEAN NOT NULL,
    compensation_required BOOLEAN NOT NULL,
    timestamp DATETIME NOT NULL,
    FOREIGN KEY (session_id) REFERENCES workflow_sessions(id)
);

CREATE INDEX idx_effects_by_session ON workflow_effects(session_id);
```

---

### Refinement 3: Transaction Isolation and Concurrency Control

**Problem (from Critical Analysis)**: SQLite concurrent access not properly specified. Two concurrent `transition()` calls on the same session could violate FSM invariants (race condition on state update).

**Solution**: Define explicit concurrency model with transaction isolation levels.

**Concurrency Model**:

1. **Per-session mutual exclusion**: Only one thread may call `transition()` per session concurrently.
   - Enforced via RwLock in `SqliteWorkflowEngine`
   - **Implementation**: `Arc<RwLock<WorkflowSession>>`

2. **SQLite transaction isolation**: Use SERIALIZABLE isolation for `workflow_sessions` updates.
   - **Schema change**: Add `version` column for optimistic concurrency detection.

```sql
ALTER TABLE workflow_sessions ADD COLUMN version INTEGER DEFAULT 0;

-- Transition updates must increment version atomically
UPDATE workflow_sessions
  SET state_data = ?, version = version + 1, updated_at = NOW()
  WHERE id = ? AND version = ?;  -- Detects concurrent writes
```

3. **Multi-session parallelism**: Different sessions may transition in parallel (no global lock).
   - SQLite WAL mode enables concurrent reads from one writer.
   - Use connection pool to service multiple sessions simultaneously.

**Implementation** (`mcb-providers/src/workflow/sqlite_workflow.rs`):

```rust
pub struct SqliteWorkflowEngine {
    pool: SqlitePool,
    session_locks: Arc<DashMap<String, Arc<RwLock<()>>>>,  // Per-session lock
}

impl SqliteWorkflowEngine {
    pub async fn transition_atomic(
        &self,
        session_id: &str,
        trigger: TransitionTrigger,
    ) -> Result<Transition> {
        // Acquire per-session write lock
        let lock = self.session_locks
            .entry(session_id.to_string())
            .or_insert_with(|| Arc::new(RwLock::new(())))
            .clone();
        
        let _guard = lock.write().await;  // Exclusive access for this session
        
        // Load session (read)
        let mut session = self.load_session(session_id).await?;
        
        // Compute new state
        let transition = mcb_providers::workflow::transitions::transition(
            &mut session,
            trigger,
        )?;
        
        // Atomic write with version check (optimistic concurrency)
        let rows_affected = sqlx::query(
            "UPDATE workflow_sessions 
             SET state_data = ?, version = version + 1, updated_at = NOW()
             WHERE id = ? AND version = ?",
        )
        .bind(serde_json::to_string(&transition.to_state)?)
        .bind(session_id)
        .bind(session.version)
        .execute(&self.pool)
        .await?
        .rows_affected();
        
        if rows_affected == 0 {
            return Err(WorkflowError::OptimisticLockConflict {
                session_id: session_id.to_string(),
            });
        }
        
        // Log transition
        self.log_transition(transition.clone()).await?;
        
        Ok(transition)
    }
}
```

**Testing**: Verify that concurrent transitions on the same session are serialized; concurrent transitions on different sessions run in parallel.

---

## Consequences

### Positive

-   **Session continuity**: Workflow state survives process restarts via SQLite.
-   **Full audit trail**: Every transition is recorded with trigger, timestamps, and guard results.
-   **Time travel**: State at any point in time can be reconstructed from the transition log.
-   **Type-safe state**: `WorkflowState` enum prevents invalid state representations at compile time.
-   **Clean Architecture**: Port trait in `mcb-domain`, implementation in `mcb-providers` — zero architectural violations.
-   **Zero new crates**: Distributed across existing crate hierarchy.
-   **Foundation for ADR-035/036/037**: `WorkflowEngine` trait is consumed by context scout (035), policy guard (036), and orchestrator (037).

### Negative

-   **Boilerplate**: Enum-based FSM requires manual `match` logic (~150 lines for transition matrix). Declarative crates like `smlang-rs` would reduce this.
-   **Runtime-only validation**: Invalid transitions detected at runtime, not compile time. Mitigated by comprehensive test coverage.
-   **SQLite dependency**: `sqlx` async SQLite driver adds ~50KB to binary and requires `libsqlite3`.
-   **Single-writer constraint**: SQLite WAL mode supports one writer at a time. Concurrent sessions on the same database require careful transaction management.

## Alternatives Considered

### Alternative 1: statig (Hierarchical State Machine Crate)

-   **Description:** Proc-macro based FSM with hierarchical states, entry/exit Actions, and async support. 3M+ downloads.
-   **Pros:** Hierarchical states reduce duplication. Compile-time state machine generation. Entry/exit hooks.
-   **Cons:** No built-in serde support — must manually serialize states. Macro-generated code harder for `mcb-validate` to analyze.
-   **Rejection reason:** Lack of native serialization makes SQLite persistence painful. The transparency loss from macros conflicts with architecture validation.

### Alternative 2: smlang-rs (Declarative DSL)

-   **Description:** `statemachine!{}` macro with declarative transition table syntax, built-in serde via `states_attr`, and first-class guard support.
-   **Pros:** Clean declarative syntax. Built-in serde. Explicit guard expressions. Good async support.
-   **Cons:** No hierarchical states. 526K downloads (less ecosystem validation). Macro generates less transparent code.
-   **Rejection reason:** Viable alternative. Rejected for transparency — enum-based approach is fully visible to `mcb-validate` and requires no macro understanding for contributors.

### Alternative 3: sm (Typestate Pattern)

-   **Description:** Compile-time typestate FSM using generic types. Invalid transitions caught at compile time.
-   **Pros:** Strongest compile-time guarantees. Zero runtime overhead.
-   **Cons:** No async support. No serialization. Lower maintenance activity.
-   **Rejection reason:** Incompatible with async-first requirement and SQLite persistence.

### Alternative 4: In-Memory Only (No SQLite)

-   **Description:** Keep state in `HashMap<String, WorkflowSession>` with no persistence.
-   **Pros:** Simpler implementation. No SQLite dependency.
-   **Cons:** State lost on restart. No audit trail. No time travel.
-   **Rejection reason:** Session continuity across restarts is a core requirement.

## Implementation Notes

### Code Changes

1.  Add `workflow.rs` entities to `mcb-domain/src/entities/`
2.  Add `workflow.rs` port trait to `mcb-domain/src/ports/providers/`
3.  Add `WorkflowError` to `mcb-domain/src/errors/`
4.  Add `WORKFLOW_PROVIDERS` slice to `mcb-application/src/registry/`
5.  Add `workflow/` module to `mcb-providers/src/` with SQLite implementation
6.  Add `WorkflowConfig` to `mcb-infrastructure/src/config/`
7.  Add `[workflow]` section to `config/default.toml`

### Migration

-   New tables only (`workflow_sessions`, `workflow_transitions`). No existing tables modified.
-   Migration SQL embedded in provider initialization with `CREATE TABLE IF NOT EXISTS`.

### Testing

-   Unit tests: Transition matrix (every valid transition + every invalid transition rejected).
-   Unit tests: Serde round-trip for every `WorkflowState` variant.
-   Integration tests: SQLite persistence (create → transition → reload → verify state).
-   Integration tests: Time travel (create → N transitions → reconstruct at T).
-   Estimated: ~60 tests.

### Performance Targets

-   State read: < 5ms (single row lookup by primary key)
-   Transition (read + validate + write + log): < 20ms (single SQLite transaction)
-   Time travel (replay N transitions): < 50ms for N ≤ 100
-   SQLite WAL mode for concurrent reads during writes

### Security

-   No user-facing credentials in workflow state.
-   `state_data` JSON may contain project paths — ensure no secrets leak into transition logs.

## References

-   [statig crate](https://crates.io/crates/statig) — Hierarchical state machine (evaluated, not selected)
-   [smlang-rs](https://crates.io/crates/smlang) — Declarative FSM macro (evaluated, not selected)
-   [sqlx](https://crates.io/crates/sqlx) — Async SQLite driver
-   [ADR-029: Hexagonal Architecture with dill](./029-hexagonal-architecture-dill.md) — DI pattern
-   [ADR-023: Provider Registration with linkme](./023-provider-registration-linkme.md) — Auto-registration
-   [ADR-032: Agent & Quality Domain Extension](./032-agent-quality-domain-extension.md) — Superseded
-   [docs/design/workflow-management/SCHEMA.md](../design/workflow-management/SCHEMA.md) — Schema reference
