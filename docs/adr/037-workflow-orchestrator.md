---
adr: 37
title: Workflow Orchestrator — Coordination and MCP Integration
status: ACCEPTED
created:
updated: 2026-02-06
related: [23, 25, 29, 33]
supersedes: []
superseded_by: []
implementation_status: Complete
---

## ADR-037: Workflow Orchestrator — Coordination and MCP Integration

## Status

**Accepted** — 2026-02-06

- **Deciders:** Project team
- **Depends on:** [ADR-034](./034-workflow-core-fsm.md) (Workflow Core FSM), [ADR-035](./035-context-scout.md) (Context Scout), [ADR-036](./036-enforcement-policies.md) (Enforcement Policies)
- **Related:** [ADR-029](./029-hexagonal-architecture-dill.md) (Hexagonal DI), [ADR-023](./023-inventory-to-linkme-migration.md) (linkme), [ADR-033](./033-mcp-handler-consolidation.md) (Handler Consolidation), [ADR-025](./025-figment-configuration.md) (Figment)
- **Series:** [ADR-034](./034-workflow-core-fsm.md) → [ADR-035](./035-context-scout.md) → [ADR-036](./036-enforcement-policies.md) → **ADR-037**

## Context

ADRs 034–036 define three independent providers:

| Provider | ADR | Responsibility |
| ---------- | ----- | --------------- |
| `WorkflowEngine` | 034 | FSM state transitions, persistence, history |
| `ContextScoutProvider` | 035 | Git/tracker/config state discovery |
| `PolicyGuardProvider` | 036 | Policy evaluation before transitions |

Each provider has a clean port trait, a linkme-registered implementation, and is injected via `Arc<dyn Trait>`. However, no component **coordinates** them into a unified workflow lifecycle.

**This ADR** defines:

1. `WorkflowService` — an application service (in `mcb-application`) that orchestrates all three providers
2. A `workflow` MCP tool (following ADR-033 action-based pattern) exposed via `mcb-server`
3. An event system for workflow state changes
4. DI registration integrating all workflow components into the existing `build_catalog()`

### Requirements

- Single service coordinating FSM + context + policies
- Session lifecycle: create → discover context → evaluate policies → transition → repeat
- MCP tool with action-based API (ADR-033 pattern)
- Event broadcasting for workflow state changes
- Integration with existing `AppContext` and dill `Catalog` (ADR-029)
- Session management (concurrent sessions, cleanup, crash recovery)
- **Multi-tier execution model** (Project → Plan → Task → Session → Agent → Operator)
- **Event broadcasting** across 3 channels (Message Queue, Database, Webhooks)
- **Beads integration** (task orientation, no state duplication)
- **Session/Compensation managers** (lifecycle, crash recovery, compensation)

## Decision

### 1. Multi-Tier Execution Model

The workflow system uses a hierarchical, multi-tier execution model that aligns work from strategic planning down to individual agent execution:

```
Project (scope boundary)
  └─ Plan (multi-phase roadmap from Beads)
      └─ Task (atomic work unit from Beads)
          └─ Session (execution context, FSM state from ADR-034)
              └─ Agent(s) (AI agents executing in parallel, bounded pool)
                  └─ Operator (human decision maker, sequential approval/override)
```

#### Entity Definitions

| Tier | Definition | Source | Responsibility |
| ------ | ----------- | -------- | ----------------- |
| **Project** | Top-level scope boundary | User-provided | Contains all work, configurations, and history |
| **Plan** | Multi-phase roadmap with dependencies | Beads issue tracker | Organizes work into logical phases |
| **Task** | Atomic work unit (feature, bug, refactor) | Beads task/issue | Single unit of work with clear acceptance criteria |
| **Session** | Execution context + FSM state | WorkflowEngine (ADR-034) | Tracks state transitions, history, operator decisions |
| **Agent** | AI agent executing within a session | OpenCode, MCP clients | Performs code changes, research, testing in parallel |
| **Operator** | Human making decisions (approve, override, merge) | OpenCode UI/MCP | Sequential decision gate before state transitions |

#### Concurrency Model

```rust
// Concurrency boundaries
#[derive(Debug, Clone)]
pub struct MultiTierConcurrency {
    /// Multiple projects run independently (isolated workspaces)
    pub projects: ConcurrencyPolicy {
        max_parallel: Unlimited,
        requires_operator_approval: false,
    },

    /// Multiple tasks per plan (bounded by WIP policy from ADR-036)
    pub tasks_per_plan: ConcurrencyPolicy {
        max_parallel: dynamic, // from PolicyGuardProvider
        requires_operator_approval: false, // policies block, don't require approval
    },

    /// Multiple sessions per task (operator controls via session manager)
    pub sessions_per_task: ConcurrencyPolicy {
        max_parallel: 10, // configurable, per-operator limit
        requires_operator_approval: true, // operator confirms resuming abandoned session
    },

    /// Multiple agents per session (bounded thread pool)
    pub agents_per_session: ConcurrencyPolicy {
        max_parallel: 4, // configured in WorkflowService
        requires_operator_approval: false,
    },

    /// Operator decisions are sequential (one at a time per task)
    pub operator_decisions: ConcurrencyPolicy {
        max_parallel: 1, // per task
        requires_operator_approval: "is the operator",
    },
}
```

#### Execution Flow

1. **Project Creation**: Operator creates or opens a project (workspace root)
2. **Plan Discovery**: WorkflowService queries Beads for plans/phases in this project
3. **Task Selection**: Operator selects a task from Beads (ready, no blockers)
4. **Session Start**: Create WorkflowSession with:

- `task_id` (reference to Beads task, not copy)
- `operator_id` (current human operator)
- `state: Initializing`

1. **Context Discovery**: ContextScoutProvider discovers Git, project structure, dependencies
2. **Policy Evaluation**: PolicyGuardProvider evaluates concurrency, branching, merge policies
3. **Agent Pool Start**: Spawn agents (bounded, configurable pool size)
4. **Agents Execute**: Multiple agents run in parallel within session

- Code changes, tests, commits happen in isolated worktrees
- Each agent heartbeats to session manager

1. **Operator Gate**: On completion, await operator approval:

- Review changes, run tests, approve merge
- Or: trigger compensation (AutoRevert, ManualReview, ApproveAndMerge)

1. **State Transition**: Execute FSM transition (Ready → Executing → Completed or Failed)
2. **Cleanup**: Close session, cleanup worktrees, record final state

---

### 2. Event Broadcasting (3 Channels)

Events occur at every state transition, agent action, and operator decision. The system broadcasts these events across **three independent channels** for different consumers:

#### WorkflowEvent Enum (Complete)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    // Session lifecycle
    SessionCreated {
        session_id: String,
        task_id: String,
        operator_id: String,
        timestamp: DateTime<Utc>,
    },
    SessionStarted {
        session_id: String,
        project_id: String,
    },
    SessionCompleted {
        session_id: String,
        reason: String, // "success", "cancelled", "timeout"
    },
    SessionFailed {
        session_id: String,
        error: String,
        compensation_triggered: bool,
    },

    // State transitions
    StateTransitioned {
        session_id: String,
        from: WorkflowState,
        to: WorkflowState,
        trigger: TransitionTrigger,
        timestamp: DateTime<Utc>,
    },
    TransitionBlocked {
        session_id: String,
        policy_violation: String,
        blocker: String,
    },

    // Compensation lifecycle
    CompensationTriggered {
        session_id: String,
        action_type: CompensationType, // AutoRevert, ManualReview, ApproveAndMerge
    },
    CompensationCompleted {
        session_id: String,
        details: String,
    },
    CompensationFailed {
        session_id: String,
        error: String,
    },

    // Operator decisions
    OperatorApproved {
        session_id: String,
        operator_id: String,
        reason: Option<String>,
    },
    OperatorRejected {
        session_id: String,
        operator_id: String,
        reason: String,
    },
    OperatorOverride {
        session_id: String,
        operator_id: String,
        override_type: String, // "force_transition", "cancel_agents", "skip_policy"
        reason: String,
    },

    // Agent lifecycle
    AgentStarted {
        session_id: String,
        agent_id: String,
        task_type: String, // "code_change", "testing", "review"
    },
    AgentCompleted {
        session_id: String,
        agent_id: String,
        result_summary: String,
    },
    AgentFailed {
        session_id: String,
        agent_id: String,
        error: String,
    },

    // Context discovery
    ContextDiscovered {
        session_id: String,
        context_snapshot_id: String,
        changes_detected: Vec<String>,
    },
}
```

#### Channel 1: Message Queue (Async, Durable, Scalable)

**Purpose**: Distribute events to external systems asynchronously.

**Technology**: Redis, RabbitMQ, or NATS (pluggable via provider trait)

**Consumers**:

- External webhooks (Slack, GitHub, PagerDuty notifications)
- Dashboard real-time updates
- Analytics/monitoring systems
- Audit trail subscribers

```rust
pub trait EventQueueProvider: Send + Sync {
    async fn publish(&self, channel: &str, event: &WorkflowEvent) -> Result<()>;
    async fn subscribe(&self, channel: &str) -> Result<Receiver<WorkflowEvent>>;
}

// Registered implementation (e.g., Redis-backed)
pub struct RedisEventQueue {
    client: redis::Client,
}

impl EventQueueProvider for RedisEventQueue {
    async fn publish(&self, channel: &str, event: &WorkflowEvent) -> Result<()> {
        let serialized = serde_json::to_string(event)?;
        self.client.publish(channel, serialized).await?;
        Ok(())
    }
}
```

**Topic Structure**:

```
workflow.sessions.created
workflow.sessions.started
workflow.sessions.completed
workflow.sessions.failed
workflow.transitions.state_changed
workflow.transitions.blocked
workflow.compensation.triggered
workflow.compensation.completed
workflow.operators.approved
workflow.operators.rejected
workflow.agents.started
workflow.agents.completed
workflow.agents.failed
```

#### Channel 2: Database (Immutable Append-Only Log)

**Purpose**: Source of truth for session history, audit trail, time-travel queries.

**Storage**: `workflow_events` table (SQLite, PostgreSQL, or configurable SQL dialect)

**Properties**:

- Immutable: events are INSERT-only, never UPDATE/DELETE
- Indexed: Fast queries by session_id, timestamp, event_type
- Ordered: Timestamp ordering enables time-travel debugging
- Complete: Every state change and decision recorded

```sql
CREATE TABLE workflow_events (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    session_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_data JSON NOT NULL,
    operator_id TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    INDEX idx_session (session_id),
    INDEX idx_timestamp (timestamp),
    INDEX idx_event_type (event_type),
    INDEX idx_operator (operator_id)
);
```

**Query Examples**:

```sql
-- Timeline of a session
SELECT * FROM workflow_events
WHERE session_id = 'sess-abc123'
ORDER BY timestamp ASC;

-- State transitions only
SELECT * FROM workflow_events
WHERE session_id = 'sess-abc123'
AND event_type = 'StateTransitioned'
ORDER BY timestamp;

-- What blocked this session?
SELECT * FROM workflow_events
WHERE session_id = 'sess-abc123'
AND event_type = 'TransitionBlocked'
LIMIT 1;

-- All decisions by operator
SELECT * FROM workflow_events
WHERE operator_id = 'op-user123'
AND event_type IN ('OperatorApproved', 'OperatorRejected', 'OperatorOverride')
ORDER BY timestamp DESC;
```

#### Channel 3: Webhooks (HTTP Callbacks, External Integrations)

**Purpose**: Notify external systems of workflow events in real-time.

**Configured per project**: Each project defines webhooks (URL + event filter)

**Delivery**: Best-effort async (fire-and-forget with retry on 5xx)

```rust
pub struct WebhookConfig {
    pub id: String,
    pub project_id: String,
    pub url: String,
    pub event_filter: Vec<String>, // ["SessionCompleted", "OperatorApproved"]
    pub headers: HashMap<String, String>, // auth tokens, custom headers
    pub retries: u32,
    pub timeout_secs: u32,
}

pub struct WebhookExecutor {
    client: reqwest::Client,
    config_provider: Arc<dyn ConfigProvider>,
}

impl WebhookExecutor {
    pub async fn execute(&self, webhook: &WebhookConfig, event: &WorkflowEvent) -> Result<()> {
        let body = serde_json::to_string(event)?;
        let request = self.client
            .post(&webhook.url)
            .headers(to_headers(&webhook.headers))
            .body(body)
            .timeout(Duration::from_secs(webhook.timeout_secs));

        // Retry on 5xx errors
        for attempt in 0..webhook.retries {
            match request.send().await {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                Ok(resp) if resp.status().is_server_error() => {
                    // exponential backoff
                    tokio::time::sleep(
                        Duration::from_secs(2_u64.pow(attempt as u32))
                    ).await;
                }
                Ok(_) => return Err("client error".into()),
                Err(e) if e.is_timeout() => return Err(e.into()),
                Err(e) => return Err(e.into()),
            }
        }
        Err("webhook failed after retries".into())
    }
}
```

#### Emit Pattern (All 3 Channels at Once)

```rust
impl WorkflowService {
    pub async fn emit_event(&self, event: WorkflowEvent) -> Result<()> {
        let session_id = extract_session_id(&event);

        // 1. Store in database (source of truth, immutable)
        self.db.record_event(&event).await?;

        // 2. Publish to message queue (for external subscribers)
        let topic = format!("workflow.{}", event_type(&event));
        self.queue.publish(&topic, &event).await?;

        // 3. Call registered webhooks (external integrations)
        if let Some(webhooks) = self.webhook_registry.get(&session_id) {
            for webhook in webhooks {
                if webhook.matches(&event) {
                    // Fire async without blocking
                    let executor = self.webhook_executor.clone();
                    tokio::spawn(async move {
                        let _ = executor.execute(&webhook, &event).await;
                    });
                }
            }
        }

        Ok(())
    }
}
```

---

### 3. Beads Integration

Beads is the **task orientation system** — it describes what work exists, dependencies, and status. Workflow is the **execution system** — it instantiates and runs tasks from Beads. The two systems must coordinate without duplicating state.

#### Design Principle: Single Source of Truth

**Beads owns**: Task definitions, dependencies, status metadata, priority, assignees
**Workflow owns**: Session state (FSM), execution history, operator decisions, agent execution

**NO state duplication**: Workflow never copies task data. It references by `task_id` and queries Beads when needed.

#### Integration Points

##### 1. Opening a Task from Beads

```rust
impl WorkflowService {
    /// Open a task from Beads, creating a WorkflowSession for it.
    pub async fn open_task(&self, beads_task_id: &str) -> Result<WorkflowSession> {
        // 1. Fetch task from Beads (read-only reference)
        let beads_task = self.beads_client.get_task(beads_task_id).await?;

        // 2. Check dependencies (Beads tells us what blocks this task)
        let blockers = self.beads_client.get_blockers(beads_task_id).await?;
        if !blockers.is_empty() {
            return Err(WorkflowError::TaskBlockedByDependencies {
                task_id: beads_task_id.to_string(),
                blockers,
            });
        }

        // 3. Create WorkflowSession (NOT copying task data, just storing reference)
        let session = WorkflowSession {
            id: generate_session_id(),
            task_id: beads_task_id.to_string(), // REFERENCE, not copy
            project_id: beads_task.project_id.clone(),
            operator_id: current_operator_id().to_string(),
            current_state: WorkflowState::Initializing,
            created_at: now(),
            last_activity: now(),
            ..Default::default()
        };

        // 4. Store session in workflow database (NOT synced back to Beads)
        self.db.create_session(&session).await?;

        // 5. Emit event
        self.emit_event(WorkflowEvent::SessionCreated {
            session_id: session.id.clone(),
            task_id: beads_task_id.to_string(),
            operator_id: session.operator_id.clone(),
            timestamp: now(),
        }).await?;

        // 6. Start session (discover context, evaluate policies)
        self.start_session(&session).await?;

        Ok(session)
    }
}
```

##### 2. On Session Completion, Query Beads for Dependents

```rust
pub async fn on_session_completed(
    &self,
    session_id: &str,
) -> Result<()> {
    let session = self.db.get_session(session_id).await?;

    // Query Beads: what tasks depend on this one?
    let dependents = self.beads_client
        .get_dependents(&session.task_id)
        .await?;

    // Auto-create sessions for unblocked dependents (optional, may require approval)
    for dependent_task_id in dependents {
        let blockers = self.beads_client
            .get_blockers(&dependent_task_id)
            .await?;

        if blockers.is_empty() {
            // This dependent is now unblocked
            tracing::info!("Dependent task unblocked: {}", dependent_task_id);

            // Emit event (dashboard can auto-offer to open it)
            self.emit_event(WorkflowEvent::TaskUnblocked {
                task_id: dependent_task_id.clone(),
            }).await?;
        }
    }

    Ok(())
}
```

##### 3. Before Transitioning, Check Beads Dependencies

```rust
pub async fn transition(
    &self,
    session_id: &str,
    trigger: TransitionTrigger,
) -> Result<Transition> {
    let session = self.db.get_session(session_id).await?;

    // Re-check Beads: did dependencies change?
    let blockers = self.beads_client
        .get_blockers(&session.task_id)
        .await?;

    if !blockers.is_empty() {
        return Err(WorkflowError::TaskBlockedByDependencies {
            task_id: session.task_id.clone(),
            blockers,
        });
    }

    // Continue with normal transition (policy checks, FSM)
    // ...
    Ok(transition)
}
```

##### 4. No Sync Back to Beads

Workflow does **NOT** update Beads task status. Beads is the source of truth for task metadata:

- If operator closes session as "completed", Beads task status is updated via:
- Manual operator action in OpenCode UI
- Separate Beads API call (not from Workflow)
- Not automatic from Workflow completion

This preserves the separation: Beads is task-oriented (planning), Workflow is execution-oriented (doing).

---

### 4. Session Manager & Compensation Handler

#### Session Manager: Lifecycle Control

Manages concurrent sessions per operator, timeout recovery, and crash detection.

```rust
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum concurrent sessions per operator
    pub max_sessions_per_operator: usize,
    /// Max concurrent sessions across all operators
    pub max_total_sessions: usize,
    /// Session timeout after inactivity (seconds)
    pub session_timeout_secs: u64,
    /// Heartbeat interval for agent health checks (seconds)
    pub heartbeat_interval_secs: u64,
    /// Detect orphaned sessions after N missed heartbeats
    pub orphan_threshold: u32,
}

pub struct SessionManager {
    db: Arc<dyn SessionStorage>,
    config: SessionConfig,
    heartbeat_tracker: Arc<RwLock<HashMap<String, Instant>>>,
}

impl SessionManager {
    pub async fn new(db: Arc<dyn SessionStorage>, config: SessionConfig) -> Self {
        let manager = Self {
            db,
            config,
            heartbeat_tracker: Arc::new(RwLock::new(HashMap::new())),
        };

        // Spawn background cleanup task
        manager.spawn_cleanup_task();

        manager
    }

    /// Register a new session.
    pub async fn register_session(&self, session: &WorkflowSession) -> Result<()> {
        // Check limits
        let operator_sessions = self.db
            .count_sessions_by_operator(&session.operator_id)
            .await?;

        if operator_sessions >= self.config.max_sessions_per_operator {
            return Err(WorkflowError::SessionLimitExceeded {
                operator_id: session.operator_id.clone(),
                limit: self.config.max_sessions_per_operator,
            });
        }

        let total = self.db.count_all_sessions().await?;
        if total >= self.config.max_total_sessions {
            return Err(WorkflowError::GlobalSessionLimitExceeded {
                limit: self.config.max_total_sessions,
            });
        }

        // Record heartbeat
        self.heartbeat_tracker
            .write()
            .await
            .insert(session.id.clone(), Instant::now());

        Ok(())
    }

    /// Heartbeat from agent (refresh activity timestamp).
    pub async fn heartbeat(&self, session_id: &str) -> Result<()> {
        self.heartbeat_tracker
            .write()
            .await
            .insert(session_id.to_string(), Instant::now());

        // Update last_activity in database
        self.db.touch_session(session_id).await?;

        Ok(())
    }

    /// Detect and recover orphaned sessions.
    pub async fn detect_orphaned(&self) -> Result<Vec<String>> {
        let mut orphaned = Vec::new();
        let tracker = self.heartbeat_tracker.read().await;
        let now = Instant::now();
        let threshold = Duration::from_secs(
            self.config.heartbeat_interval_secs * self.config.orphan_threshold as u64
        );

        for (session_id, last_beat) in tracker.iter() {
            if now.duration_since(*last_beat) > threshold {
                orphaned.push(session_id.clone());
            }
        }

        // For each orphaned session: emit event, pause agents, offer recovery
        for session_id in &orphaned {
            self.emit_event(WorkflowEvent::SessionOrphaned {
                session_id: session_id.clone(),
                last_activity: self.db.get_session_activity(session_id).await?,
            }).await?;
        }

        Ok(orphaned)
    }

    /// Allow operator to resume abandoned session.
    pub async fn resume_session(&self, session_id: &str) -> Result<()> {
        // Re-register heartbeat
        self.heartbeat_tracker
            .write()
            .await
            .insert(session_id.to_string(), Instant::now());

        // Update session state
        self.db.touch_session(session_id).await?;

        // Agents can resume
        Ok(())
    }

    /// Background cleanup task (runs every 60 seconds).
    fn spawn_cleanup_task(&self) {
        let db = self.db.clone();
        let timeout = Duration::from_secs(self.config.session_timeout_secs);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;

                if let Ok(expired) = db.list_expired_sessions(timeout).await {
                    for session_id in expired {
                        let _ = db.cleanup_session(&session_id).await;
                    }
                }
            }
        });
    }
}
```

#### Compensation Handler: Recovery Actions

Handles cleanup and recovery when a session fails, is cancelled, or needs manual intervention.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompensationType {
    /// Revert all changes (git reset, cleanup worktrees)
    AutoRevert {
        keep_branch: bool,
    },
    /// Pause and wait for operator decision
    ManualReview {
        deadline_secs: Option<u64>,
    },
    /// Approve and merge (auto-create PR, run checks, merge)
    ApproveAndMerge {
        target_branch: String,
        require_checks: bool,
    },
}

pub trait CompensationHandler: Send + Sync {
    async fn execute(
        &self,
        session_id: &str,
        action: CompensationType,
    ) -> Result<CompensationResult>;
}

pub struct CompensationResult {
    pub success: bool,
    pub message: String,
    pub details: HashMap<String, String>,
}

/// Default implementation
pub struct DefaultCompensationHandler {
    git_client: Arc<dyn GitProvider>,
    pr_client: Arc<dyn GitHubClient>,
    db: Arc<dyn SessionStorage>,
}

impl CompensationHandler for DefaultCompensationHandler {
    async fn execute(
        &self,
        session_id: &str,
        action: CompensationType,
    ) -> Result<CompensationResult> {
        let session = self.db.get_session(session_id).await?;

        match action {
            CompensationType::AutoRevert { keep_branch } => {
                self.emit_event(WorkflowEvent::CompensationTriggered {
                    session_id: session_id.to_string(),
                    action_type: "AutoRevert".to_string(),
                }).await?;

                // 1. Reset to pre-session state
                self.git_client
                    .reset(&session.branch, &session.initial_commit)
                    .await?;

                // 2. Close worktree
                self.git_client
                    .close_worktree(&session.worktree_path)
                    .await?;

                // 3. Optional: delete branch
                if !keep_branch {
                    self.git_client
                        .delete_branch(&session.branch)
                        .await?;
                }

                self.emit_event(WorkflowEvent::CompensationCompleted {
                    session_id: session_id.to_string(),
                    details: "Reverted all changes, closed worktree".to_string(),
                }).await?;

                Ok(CompensationResult {
                    success: true,
                    message: "Session reverted".to_string(),
                    details: {
                        let mut m = HashMap::new();
                        m.insert("branch_deleted".to_string(), (!keep_branch).to_string());
                        m.insert("worktree_closed".to_string(), "true".to_string());
                        m
                    },
                })
            }

            CompensationType::ManualReview { deadline_secs } => {
                self.emit_event(WorkflowEvent::CompensationTriggered {
                    session_id: session_id.to_string(),
                    action_type: "ManualReview".to_string(),
                }).await?;

                // Pause agents, wait for operator decision
                // Event will be shown in dashboard; operator approves or rejects

                Ok(CompensationResult {
                    success: true,
                    message: "Awaiting operator decision".to_string(),
                    details: {
                        let mut m = HashMap::new();
                        if let Some(deadline) = deadline_secs {
                            m.insert("deadline_secs".to_string(), deadline.to_string());
                        }
                        m
                    },
                })
            }

            CompensationType::ApproveAndMerge {
                target_branch,
                require_checks,
            } => {
                self.emit_event(WorkflowEvent::CompensationTriggered {
                    session_id: session_id.to_string(),
                    action_type: "ApproveAndMerge".to_string(),
                }).await?;

                // 1. Create PR if not exists
                let pr = self.pr_client
                    .create_or_get_pr(
                        &session.branch,
                        &target_branch,
                        &session.task_id,
                    )
                    .await?;

                // 2. Run checks (CI)
                if require_checks {
                    let checks = self.pr_client.wait_for_checks(&pr.id, 30 * 60).await?;
                    if !checks.all_passed {
                        return Err(WorkflowError::ChecksFailed {
                            pr_id: pr.id,
                            details: checks.failures,
                        });
                    }
                }

                // 3. Merge PR
                self.pr_client.merge_pr(&pr.id).await?;

                // 4. Close worktree
                self.git_client
                    .close_worktree(&session.worktree_path)
                    .await?;

                self.emit_event(WorkflowEvent::CompensationCompleted {
                    session_id: session_id.to_string(),
                    details: format!("Merged PR #{} into {}", pr.id, target_branch),
                }).await?;

                Ok(CompensationResult {
                    success: true,
                    message: "Changes merged successfully".to_string(),
                    details: {
                        let mut m = HashMap::new();
                        m.insert("pr_id".to_string(), pr.id.to_string());
                        m.insert("merged_to".to_string(), target_branch);
                        m
                    },
                })
            }
        }
    }
}

// Side effects cleanup helper
pub struct SideEffectsManager;

impl SideEffectsManager {
    /// Clean up temporary files, close branches, notify team
    pub async fn cleanup(
        &self,
        session: &WorkflowSession,
        git_client: &Arc<dyn GitProvider>,
        notifier: &Arc<dyn NotificationProvider>,
    ) -> Result<()> {
        // 1. Close worktree
        git_client.close_worktree(&session.worktree_path).await?;

        // 2. Delete temporary files
        if let Some(temp_dir) = &session.temp_dir {
            tokio::fs::remove_dir_all(temp_dir).await?;
        }

        // 3. Notify team (if integration configured)
        notifier.send_message(
            "workflow_completed",
            &format!("Session {} completed for task {}", session.id, session.task_id),
        ).await?;

        Ok(())
    }
}
```

---

### 5. WorkflowService (Application Layer)

```rust
// mcb-application/src/services/workflow_service.rs

use mcb_domain::entities::context::ProjectContext;
use mcb_domain::entities::policy::PolicyResult;
use mcb_domain::entities::workflow::{
    Transition, TransitionTrigger, WorkflowSession, WorkflowState,
};
use mcb_domain::errors::WorkflowError;
use mcb_domain::ports::providers::context_scout::ContextScoutProvider;
use mcb_domain::ports::providers::policy_guard::PolicyGuardProvider;
use mcb_domain::ports::providers::workflow::WorkflowEngine;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Workflow events emitted on state changes.
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    SessionStarted {
        session_id: String,
        project_id: String,
    },
    ContextDiscovered {
        session_id: String,
        context_snapshot_id: String,
    },
    StateTransitioned {
        session_id: String,
        from: String,
        to: String,
        trigger: String,
    },
    PolicyEvaluated {
        session_id: String,
        result: PolicyResult,
    },
    SessionCompleted {
        session_id: String,
    },
}

/// Orchestrates workflow engine, context scout, and policy guard.
///
/// This is an application service (not a domain entity). It lives in
/// mcb-application and depends only on port traits from mcb-domain.
pub struct WorkflowService {
    engine: Arc<dyn WorkflowEngine>,
    scout: Arc<dyn ContextScoutProvider>,
    guard: Arc<dyn PolicyGuardProvider>,
    event_tx: broadcast::Sender<WorkflowEvent>,
}

impl WorkflowService {
    pub fn new(
        engine: Arc<dyn WorkflowEngine>,
        scout: Arc<dyn ContextScoutProvider>,
        guard: Arc<dyn PolicyGuardProvider>,
        event_tx: broadcast::Sender<WorkflowEvent>,
    ) -> Self {
        Self { engine, scout, guard, event_tx }
    }

    /// Start a new workflow session.
    ///
    /// 1. Creates session in FSM (state: Initializing)
    /// 2. Discovers project context
    /// 3. Evaluates policies on ContextDiscovered trigger
    /// 4. Transitions to Ready if policies pass
    pub async fn start_session(
        &self,
        project_root: &std::path::Path,
        project_id: &str,
    ) -> Result<WorkflowSession, WorkflowError> {
        // 1. Create session
        let session = self.engine.create_session(project_id).await?;
        let _ = self.event_tx.send(WorkflowEvent::SessionStarted {
            session_id: session.id.clone(),
            project_id: project_id.to_string(),
        });

        // 2. Discover context
        let context = self.scout.discover(project_root).await?;
        let _ = self.event_tx.send(WorkflowEvent::ContextDiscovered {
            session_id: session.id.clone(),
            context_snapshot_id: context.id.clone(),
        });

        // 3. Evaluate policies
        let trigger = TransitionTrigger::ContextDiscovered {
            context_snapshot_id: context.id.clone(),
        };
        let policy_result = self.guard.evaluate(&trigger, &context).await?;
        let _ = self.event_tx.send(WorkflowEvent::PolicyEvaluated {
            session_id: session.id.clone(),
            result: policy_result.clone(),
        });

        // 4. Transition to Ready (policies are advisory on session start)
        let transition = self.engine.transition(&session.id, trigger).await?;
        let _ = self.event_tx.send(WorkflowEvent::StateTransitioned {
            session_id: session.id.clone(),
            from: transition.from_state.to_string(),
            to: transition.to_state.to_string(),
            trigger: format!("{:?}", transition.trigger),
        });

        // Return updated session
        let state = self.engine.current_state(&session.id).await?;
        Ok(WorkflowSession {
            current_state: state,
            ..session
        })
    }

    /// Execute a guarded transition.
    ///
    /// 1. Discovers fresh context
    /// 2. Evaluates policies for the trigger
    /// 3. If policies pass (no Error violations), executes FSM transition
    /// 4. If policies fail, returns PolicyViolation error
    pub async fn transition(
        &self,
        session_id: &str,
        project_root: &std::path::Path,
        trigger: TransitionTrigger,
    ) -> Result<Transition, WorkflowError> {
        // 1. Fresh context
        let context = self.scout.discover(project_root).await?;

        // 2. Policy check
        let policy_result = self.guard.evaluate(&trigger, &context).await?;
        let _ = self.event_tx.send(WorkflowEvent::PolicyEvaluated {
            session_id: session_id.to_string(),
            result: policy_result.clone(),
        });

        if !policy_result.allowed {
            return Err(WorkflowError::PolicyViolation {
                message: policy_result.format_violations(),
            });
        }

        // 3. Execute transition
        let transition = self.engine.transition(session_id, trigger).await?;
        let _ = self.event_tx.send(WorkflowEvent::StateTransitioned {
            session_id: session_id.to_string(),
            from: transition.from_state.to_string(),
            to: transition.to_state.to_string(),
            trigger: format!("{:?}", transition.trigger),
        });

        Ok(transition)
    }

    /// End a session (transition to Completed).
    pub async fn end_session(&self, session_id: &str) -> Result<Transition, WorkflowError> {
        let transition = self.engine
            .transition(session_id, TransitionTrigger::EndSession)
            .await?;

        let _ = self.event_tx.send(WorkflowEvent::SessionCompleted {
            session_id: session_id.to_string(),
        });

        Ok(transition)
    }

    /// Get current session status with context.
    pub async fn status(
        &self,
        session_id: &str,
        project_root: &std::path::Path,
    ) -> Result<WorkflowStatus, WorkflowError> {
        let state = self.engine.current_state(session_id).await?;
        let context = self.scout.discover(project_root).await?;
        let policies = self.guard.list_policies().await?;

        Ok(WorkflowStatus {
            session_id: session_id.to_string(),
            state,
            context,
            active_policies: policies,
        })
    }

    /// Get transition history.
    pub async fn history(
        &self,
        session_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Transition>, WorkflowError> {
        self.engine.history(session_id, limit).await
    }

    /// Discover context without triggering a transition.
    pub async fn discover_context(
        &self,
        project_root: &std::path::Path,
    ) -> Result<ProjectContext, WorkflowError> {
        self.scout.discover(project_root).await
    }

    /// Check policies without executing a transition (dry-run).
    pub async fn check_policies(
        &self,
        project_root: &std::path::Path,
        trigger: &TransitionTrigger,
    ) -> Result<PolicyResult, WorkflowError> {
        let context = self.scout.discover(project_root).await?;
        self.guard.evaluate(trigger, &context).await
    }

    /// List active sessions.
    pub async fn active_sessions(&self) -> Result<Vec<WorkflowSession>, WorkflowError> {
        self.engine.active_sessions().await
    }

    /// Subscribe to workflow events.
    pub fn subscribe(&self) -> broadcast::Receiver<WorkflowEvent> {
        self.event_tx.subscribe()
    }
}

/// Combined status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub session_id: String,
    pub state: WorkflowState,
    pub context: ProjectContext,
    pub active_policies: Vec<PolicyConfig>,
}
```

### 6. MCP Tool Handler (ADR-033 Pattern)

```rust
// mcb-server/src/handlers/workflow.rs

use serde::{Deserialize, Serialize};

/// `workflow` MCP tool following ADR-033 action-based pattern.
///
/// Single tool with multiple actions, replacing what would be 7+ separate tools.
#[derive(Debug, Deserialize)]
pub struct WorkflowArgs {
    /// Action to perform.
    pub action: WorkflowAction,
    /// Session ID (required for most actions, optional for start/list).
    pub session_id: Option<String>,
    /// Project root path (auto-detected if not specified).
    pub project_root: Option<String>,
    /// Transition trigger (required for action=transition).
    pub trigger: Option<TransitionTrigger>,
    /// Maximum items to return (for history/list actions).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowAction {
    /// Start a new workflow session.
    Start,
    /// Get current session status with context.
    Status,
    /// Execute a guarded state transition.
    Transition,
    /// Get transition history for a session.
    History,
    /// Discover project context (without transition).
    DiscoverContext,
    /// Dry-run policy evaluation (without transition).
    CheckPolicies,
    /// List all active sessions.
    ListSessions,
    /// End a session.
    EndSession,
    /// List registered policies and their configuration.
    ListPolicies,
}

/// MCP tool handler.
pub async fn handle_workflow(
    args: WorkflowArgs,
    service: &WorkflowService,
) -> Result<serde_json::Value, WorkflowError> {
    let project_root = args.project_root
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    match args.action {
        WorkflowAction::Start => {
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let session = service.start_session(&project_root, &project_id).await?;
            Ok(serde_json::to_value(session)?)
        }

        WorkflowAction::Status => {
            let session_id = require_session_id(&args)?;
            let status = service.status(&session_id, &project_root).await?;
            Ok(serde_json::to_value(status)?)
        }

        WorkflowAction::Transition => {
            let session_id = require_session_id(&args)?;
            let trigger = args.trigger
                .ok_or_else(|| WorkflowError::InvalidTransition {
                    from: "unknown".into(),
                    trigger: "missing trigger in args".into(),
                })?;
            let transition = service.transition(&session_id, &project_root, trigger).await?;
            Ok(serde_json::to_value(transition)?)
        }

        WorkflowAction::History => {
            let session_id = require_session_id(&args)?;
            let history = service.history(&session_id, args.limit).await?;
            Ok(serde_json::to_value(history)?)
        }

        WorkflowAction::DiscoverContext => {
            let context = service.discover_context(&project_root).await?;
            Ok(serde_json::to_value(context)?)
        }

        WorkflowAction::CheckPolicies => {
            let trigger = args.trigger.unwrap_or(TransitionTrigger::ContextDiscovered {
                context_snapshot_id: String::new(),
            });
            let result = service.check_policies(&project_root, &trigger).await?;
            Ok(serde_json::to_value(result)?)
        }

        WorkflowAction::ListSessions => {
            let sessions = service.active_sessions().await?;
            Ok(serde_json::to_value(sessions)?)
        }

        WorkflowAction::EndSession => {
            let session_id = require_session_id(&args)?;
            let transition = service.end_session(&session_id).await?;
            Ok(serde_json::to_value(transition)?)
        }

        WorkflowAction::ListPolicies => {
            let policies = service.check_policies(&project_root, &TransitionTrigger::ContextDiscovered {
                context_snapshot_id: String::new(),
            }).await;
            let policy_list = service.guard.list_policies().await?;
            Ok(serde_json::to_value(policy_list)?)
        }
    }
}

fn require_session_id(args: &WorkflowArgs) -> Result<String, WorkflowError> {
    args.session_id.clone().ok_or_else(|| WorkflowError::SessionNotFound {
        session_id: "(not provided)".to_string(),
    })
}
```

### 7. MCP Tool Schema (JSON)

```json
{
  "name": "workflow",
  "description": "Manage workflow sessions, transitions, policies, and project context",
  "inputSchema": {
    "type": "object",
    "required": ["action"],
    "properties": {
      "action": {
        "type": "string",
        "enum": ["start", "status", "transition", "history", "discover_context", "check_policies", "list_sessions", "end_session", "list_policies"],
        "description": "Action to perform"
      },
      "session_id": {
        "type": "string",
        "description": "Workflow session ID (required for status/transition/history/end_session)"
      },
      "project_root": {
        "type": "string",
        "description": "Project root path (auto-detected if omitted)"
      },
      "trigger": {
        "type": "object",
        "description": "Transition trigger (required for action=transition, optional for check_policies)"
      },
      "limit": {
        "type": "integer",
        "description": "Max items to return (for history/list actions)"
      }
    }
  }
}
```

### 8. DI Integration (dill Catalog)

```rust
// mcb-infrastructure/src/di/workflow_catalog.rs

use mcb_application::registry::{
    context::CONTEXT_PROVIDERS,
    guard::GUARD_PROVIDERS,
    workflow::WORKFLOW_PROVIDERS,
};
use mcb_application::services::workflow_service::WorkflowService;
use mcb_domain::ports::providers::{
    context_scout::ContextScoutProvider,
    policy_guard::PolicyGuardProvider,
    workflow::WorkflowEngine,
};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Handle for runtime switching of workflow engine provider.
pub struct WorkflowEngineHandle {
    inner: RwLock<Arc<dyn WorkflowEngine>>,
}

impl WorkflowEngineHandle {
    pub fn new(provider: Arc<dyn WorkflowEngine>) -> Self {
        Self { inner: RwLock::new(provider) }
    }

    pub async fn get(&self) -> Arc<dyn WorkflowEngine> {
        self.inner.read().await.clone()
    }

    pub async fn switch(&self, provider: Arc<dyn WorkflowEngine>) {
        *self.inner.write().await = provider;
    }
}

/// Handle for context scout provider.
pub struct ContextScoutHandle {
    inner: RwLock<Arc<dyn ContextScoutProvider>>,
}

impl ContextScoutHandle {
    pub fn new(provider: Arc<dyn ContextScoutProvider>) -> Self {
        Self { inner: RwLock::new(provider) }
    }

    pub async fn get(&self) -> Arc<dyn ContextScoutProvider> {
        self.inner.read().await.clone()
    }
}

/// Handle for policy guard provider.
pub struct PolicyGuardHandle {
    inner: RwLock<Arc<dyn PolicyGuardProvider>>,
}

impl PolicyGuardHandle {
    pub fn new(provider: Arc<dyn PolicyGuardProvider>) -> Self {
        Self { inner: RwLock::new(provider) }
    }

    pub async fn get(&self) -> Arc<dyn PolicyGuardProvider> {
        self.inner.read().await.clone()
    }
}

/// Build workflow components and register in Catalog.
pub async fn register_workflow(
    config: &figment::Figment,
    catalog_builder: &mut dill::CatalogBuilder,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Resolve providers from linkme registries
    let engine: Arc<dyn WorkflowEngine> = WORKFLOW_PROVIDERS
        .iter()
        .find(|p| p.name == "sqlite")
        .map(|p| (p.factory)(config))
        .ok_or("No workflow provider found")??;

    let scout: Arc<dyn ContextScoutProvider> = CONTEXT_PROVIDERS
        .iter()
        .find(|p| p.name == "cached")
        .map(|p| (p.factory)(config))
        .ok_or("No context provider found")??;

    let guard: Arc<dyn PolicyGuardProvider> = GUARD_PROVIDERS
        .iter()
        .find(|p| p.name == "configurable")
        .map(|p| (p.factory)(config))
        .ok_or("No guard provider found")??;

    // 2. Create handles (RwLock wrappers for runtime switching)
    let engine_handle = Arc::new(WorkflowEngineHandle::new(engine.clone()));
    let scout_handle = Arc::new(ContextScoutHandle::new(scout.clone()));
    let guard_handle = Arc::new(PolicyGuardHandle::new(guard.clone()));

    // 3. Create event channel
    let (event_tx, _) = broadcast::channel::<WorkflowEvent>(256);

    // 4. Create WorkflowService
    let service = Arc::new(WorkflowService::new(
        engine, scout, guard, event_tx,
    ));

    // 5. Register in catalog
    catalog_builder
        .add_value(engine_handle)
        .add_value(scout_handle)
        .add_value(guard_handle)
        .add_value(service);

    Ok(())
}
```

### 9. AppContext Extension

```rust
// mcb-infrastructure/src/app_context.rs (extension)

impl AppContext {
    // Existing fields: embedding_handle, vector_store_handle, cache_handle, ...

    // New workflow fields
    pub fn workflow_service(&self) -> &Arc<WorkflowService> {
        &self.workflow_service
    }

    pub fn workflow_engine_handle(&self) -> &Arc<WorkflowEngineHandle> {
        &self.workflow_engine_handle
    }

    pub fn context_scout_handle(&self) -> &Arc<ContextScoutHandle> {
        &self.context_scout_handle
    }

    pub fn policy_guard_handle(&self) -> &Arc<PolicyGuardHandle> {
        &self.policy_guard_handle
    }
}
```

### 10. Session Management

```rust
// mcb-application/src/services/session_manager.rs

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

struct SessionEntry {
    session_id: String,
    created_at: Instant,
    last_activity: Instant,
}

/// Manages active workflow sessions with timeout and cleanup.
pub struct SessionManager {
    sessions: RwLock<HashMap<String, SessionEntry>>,
    max_sessions: usize,
    session_timeout: Duration,
}

impl SessionManager {
    pub fn new(max_sessions: usize, timeout_seconds: u64) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            max_sessions,
            session_timeout: Duration::from_secs(timeout_seconds),
        }
    }

    /// Start background cleanup task.
    pub fn spawn_cleanup(&self) -> tokio::task::JoinHandle<()> {
        let sessions = self.sessions.clone();
        let timeout = self.session_timeout;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut map = sessions.write().await;
                let now = Instant::now();
                map.retain(|_, entry| now.duration_since(entry.last_activity) < timeout);
            }
        })
    }

    pub async fn register(&self, session_id: String) -> Result<(), WorkflowError> {
        let mut map = self.sessions.write().await;
        if map.len() >= self.max_sessions {
            return Err(WorkflowError::Persistence {
                message: format!("Max sessions ({}) reached", self.max_sessions),
            });
        }
        map.insert(session_id.clone(), SessionEntry {
            session_id,
            created_at: Instant::now(),
            last_activity: Instant::now(),
        });
        Ok(())
    }

    pub async fn touch(&self, session_id: &str) {
        if let Some(entry) = self.sessions.write().await.get_mut(session_id) {
            entry.last_activity = Instant::now();
        }
    }

    pub async fn remove(&self, session_id: &str) {
        self.sessions.write().await.remove(session_id);
    }
}
```

### 11. Configuration

```toml

# config/default.toml — [orchestrator] section

[orchestrator]

# Maximum concurrent workflow sessions.
max_sessions = 10

# Session timeout in seconds (auto-cleanup after inactivity).
session_timeout_seconds = 3600

# Event channel capacity.
event_channel_capacity = 256
```

```rust
// mcb-infrastructure/src/config/orchestrator.rs

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct OrchestratorConfig {
    #[serde(default = "default_max_sessions")]
    pub max_sessions: usize,
    #[serde(default = "default_timeout")]
    pub session_timeout_seconds: u64,
    #[serde(default = "default_channel_capacity")]
    pub event_channel_capacity: usize,
}

fn default_max_sessions() -> usize { 10 }
fn default_timeout() -> u64 { 3600 }
fn default_channel_capacity() -> usize { 256 }
```

### 12. Module Locations

| Crate | Path | Content |
| ------- | ------ | --------- |
| `mcb-application` | `src/services/workflow_service.rs` | `WorkflowService`, `WorkflowEvent`, `WorkflowStatus` |
| `mcb-application` | `src/services/session_manager.rs` | `SessionManager` |
| `mcb-server` | `src/handlers/workflow.rs` | `WorkflowArgs`, `WorkflowAction`, `handle_workflow()` |
| `mcb-infrastructure` | `src/di/workflow_catalog.rs` | `register_workflow()`, handle types |
| `mcb-infrastructure` | `src/config/orchestrator.rs` | `OrchestratorConfig` |

## Consequences

### Positive

- **Single coordination point**: `WorkflowService` orchestrates all three providers without any provider knowing about the others.
- **Guarded transitions**: Every transition passes through policy evaluation — no way to bypass guards.
- **Event-driven**: `broadcast::Sender` allows any consumer to subscribe to workflow state changes without coupling.
- **ADR-033 compliant**: Single `workflow` tool with 9 Actions replaces what would be 9 separate MCP tools.
- **Clean DI**: Handles + dill Catalog follow the exact pattern of existing providers (embedding, vector store, cache).
- **Session management**: Max sessions, timeout, and cleanup prevent resource leaks.
- **Zero new crates**: Service in `mcb-application`, handler in `mcb-server`, DI in `mcb-infrastructure`.

### Negative

- **Context re-discovery**: Each guarded transition discovers fresh context (30ms cold, <1ms warm). Trade-off for correctness — stale context could allow invalid transitions.
- **Broadcast channel overhead**: `broadcast::channel(256)` allocates a ring buffer. Minimal cost (~2KB) but non-zero.
- **Service complexity**: `WorkflowService` has 8 public methods. This is the maximum — any new features should extend existing methods, not add new ones.
- **Session manager is in-memory**: Lost on restart. Active sessions survive via SQLite (FSM state), but the in-memory session map is rebuilt on startup.

## Alternatives Considered

### Alternative 1: Tokio Actor Model (actix-style)

- **Description:** Each workflow session as a Tokio task with an `mpsc` mailbox. Messages (triggers) sent to actor, actor manages state internally.
- **Pros:** Natural concurrency. Each session isolated. Clean shutdown semantics.
- **Cons:** Significant complexity increase. Actor lifecycle management. Message serialization overhead. Debugging harder.
- **Rejection reason:** MCB's workload is low-concurrency (1–10 sessions). Actor overhead unjustified. Simple `Arc<WorkflowService>` with `RwLock` handles is sufficient.

### Alternative 2: Multiple MCP Tools

- **Description:** Separate tools: `workflow_start`, `workflow_status`, `workflow_transition`, etc.
- **Pros:** Each tool is simpler. Follows UNIX "do one thing" philosophy.
- **Cons:** Violates ADR-033 consolidation pattern. 9 tools instead of 1. More handler boilerplate.
- **Rejection reason:** ADR-033 explicitly moves toward action-based consolidation. Regression to multiple tools is architectural inconsistency.

### Alternative 3: Direct Provider Access (No Service Layer)

- **Description:** MCP handler calls `WorkflowEngine`, `ContextScoutProvider`, and `PolicyGuardProvider` directly.
- **Pros:** Simpler. No intermediate service.
- **Cons:** Handler contains orchestration logic. Duplicated if CLI is added later. No event broadcasting. No session management.
- **Rejection reason:** Violates Clean Architecture — orchestration belongs in the application layer, not in handlers (infrastructure/adapter layer).

## Implementation Notes

### Code Changes

1. Add `workflow_service.rs` to `mcb-application/src/services/`
2. Add `session_manager.rs` to `mcb-application/src/services/`
3. Add `workflow.rs` handler to `mcb-server/src/handlers/`
4. Add `workflow_catalog.rs` to `mcb-infrastructure/src/di/`
5. Add handle types (`WorkflowEngineHandle`, `ContextScoutHandle`, `PolicyGuardHandle`)
6. Add `OrchestratorConfig` to `mcb-infrastructure/src/config/`
7. Extend `AppContext` with workflow fields
8. Register `workflow` tool in MCP server tool list
9. Add `[orchestrator]` section to `config/default.toml`

### Migration

- No existing code modified (additive only).
- `register_workflow()` called in `build_catalog()` after existing provider registration.
- `workflow` tool added to MCP tool registry alongside existing tools.

### Testing

- Unit tests: `WorkflowService` lifecycle (start → transition → end) with mock providers.
- Unit tests: Guarded transition (policy blocks → error returned).
- Unit tests: Event emission (subscribe, receive events).
- Unit tests: `SessionManager` (register, touch, timeout, max capacity).
- Integration tests: Full `handle_workflow()` with all Actions.
- Integration tests: MCP tool invocation via JSON-RPC.
- Estimated: ~55 tests.

### Performance Targets

| Operation | Target |
| ----------- | -------- |
| `start_session()` | < 50ms (create + discover + evaluate + transition) |
| `transition()` (guarded) | < 40ms (discover + evaluate + transition) |
| `status()` | < 35ms (state read + discover) |
| `history()` | < 10ms (SQLite query) |
| `discover_context()` | < 30ms cold / < 1ms warm |
| Event broadcast | < 1ms |

### Security

- `WorkflowService` enforces policies on every transition. No bypass path.
- Session IDs are UUIDs — not guessable.
- `project_root` from MCP args is validated to be an existing directory.
- No credentials stored in workflow state or events.

## References

- [Tokio::sync::broadcast](https://docs.rs/tokio/latest/tokio/sync/broadcast/) — Event channel
- [ADR-034: Workflow Core FSM](./034-workflow-core-fsm.md) — `WorkflowEngine` trait
- [ADR-035: Context Scout](./035-context-scout.md) — `ContextScoutProvider` trait
- [ADR-036: Enforcement Policies](./036-enforcement-policies.md) — `PolicyGuardProvider` trait
- [ADR-033: MCP Handler Consolidation](./033-mcp-handler-consolidation.md) — Action-based tool pattern
- [ADR-029: Hexagonal Architecture with dill](./029-hexagonal-architecture-dill.md) — DI pattern
- [ADR-025: Figment Configuration](./025-figment-configuration.md) — Config pattern
