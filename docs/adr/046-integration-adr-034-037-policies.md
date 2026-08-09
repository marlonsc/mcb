<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 46
title: Integration with ADR-034-037 & Policies
status: PROPOSED
created:
updated: 2026-02-05
related: []
supersedes: []
superseded_by: []
implementation_status: Complete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR-046: Integration with ADR-034-037 & Policies

> **v0.3.0 Note**: `mcb-application` crate was removed. Use cases moved to `mcb-infrastructure::di::modules::use_cases`.

**Status**: Proposed
**Date**: 2026-02-05
**Deciders**: MCB Architecture Team
**Related**: ADR-034-037 (Workflow series), ADR-041-045 (Context system)
**Series Finale**: Completes ADR-041-046 (v0.4.0 architecture)

## Context

ADR-041-045 define the integrated context system layers. ADR-046**bridges**this to ADR-034-037 (workflow FSM, policies, compensation) and specifies**how they interact**.

Key integration points:

1. **FSM gates context freshness requirements** (state determines what freshness is acceptable)
2. **Policies define context boundaries** (scope isolation, access control)
3. **Compensation triggers context re-validation** (if operation fails, re-check context)
4. **Context snapshots enable rollback** (time-travel to pre-operation state)

## Decision

### 1. FSM ↔ Context: State-Based Freshness Requirements

```rust
// mcb-domain/entities/workflow.rs

#[derive(Clone, Debug)]
pub struct WorkflowState {
    pub state: ExecutionState,
    pub context_requirements: ContextRequirements,
}

pub struct ContextRequirements {
    pub freshness_minimum: ContextFreshness,
    pub scope_level: ScopeLevel,
    pub required_policies: Vec<PolicyId>,
    pub update_frequency: Duration,  // How often context must be refreshed
}

pub enum ExecutionState {
    Planning {
        // Planning: any context freshness is OK (exploring options)
        // No policies enforced yet
        requirements: ContextRequirements {
            freshness_minimum: ContextFreshness::Stale,
            scope_level: ScopeLevel::Project,
            required_policies: vec![],
            update_frequency: Duration::from_secs(300),  // Hourly refresh OK
        },
    },

    Ready {
        // Ready to execute: need recent context, all non-blocking policies
        requirements: ContextRequirements {
            freshness_minimum: ContextFreshness::Acceptable,
            scope_level: ScopeLevel::Crate,
            required_policies: vec![PolicyId::WipLimit, PolicyId::CleanWorktree],
            update_frequency: Duration::from_secs(60),  // Per-minute refresh
        },
    },

    Executing {
        // Actively executing: need fresh context, all policies strict
        requirements: ContextRequirements {
            freshness_minimum: ContextFreshness::Fresh,  // < 5s old
            scope_level: ScopeLevel::Module,
            required_policies: vec![
                PolicyId::CleanWorktree,
                PolicyId::TestsPassing,
                PolicyId::CoverageThreshold,
                PolicyId::NoBrokenImports,
            ],
            update_frequency: Duration::from_secs(5),  // Real-time refresh
        },
    },

    Suspended {
        // Paused: context can be stale (user reviewing)
        requirements: ContextRequirements {
            freshness_minimum: ContextFreshness::Acceptable,
            scope_level: ScopeLevel::Project,
            required_policies: vec![],
            update_frequency: Duration::from_secs(600),
        },
    },
}

// Validation at state transitions
pub async fn validate_transition(
    from: &ExecutionState,
    to: &ExecutionState,
    context: &ContextSnapshot,
) -> Result<TransitionValidation> {
    let requirements = &to.context_requirements;

    // Check freshness
    if context.freshness.severity() < requirements.freshness_minimum.severity() {
        return Err(TransitionError::InsufficientFreshness {
            required: requirements.freshness_minimum,
            actual: context.freshness,
        });
    }

    // Check policies
    let policy_violations = validate_policies(context, &requirements.required_policies).await?;
    if !policy_violations.is_empty() {
        return Err(TransitionError::PolicyViolations(policy_violations));
    }

    // Check scope
    if !context.scope.matches(&requirements.scope_level) {
        return Err(TransitionError::ScopeInsufficient);
    }

    Ok(TransitionValidation::allowed())
}
```

### 2. Policies ↔ Context: Scope Isolation & Access Control

```rust
pub struct ContextPolicy {
    pub id: PolicyId,
    pub name: String,
    pub scope: ScopeLevel,              // What level of code this policy guards
    pub evaluation_triggers: Vec<PolicyTrigger>,
    pub deny_wins: bool,               // One deny = whole context denied
}

pub enum PolicyTrigger {
    OnTransition { from: ExecutionState, to: ExecutionState },
    OnContextRefresh,
    OnCodeChange,
    OnSchedule { interval: Duration },
}

pub enum ScopeLevel {
    Project,   // All code in project
    Crate,     // Single crate
    Module,    // Single module + public API
    File,      // Single file only
}

pub struct PolicyEvaluationResult {
    pub policy_id: PolicyId,
    pub passed: bool,
    pub violations: Vec<PolicyViolation>,
    pub evaluated_at: SystemTime,
    pub context_snapshot_id: ContextId,  // Which context was evaluated
    pub reason: String,
}

#[async_trait]
pub trait PolicyGuard: Send + Sync {
    async fn evaluate(
        &self,
        context: &ContextSnapshot,
        scope: &ScopeLevel,
    ) -> Result<PolicyEvaluationResult>;
}

// Example: SecurityScan policy
pub struct SecurityScanPolicy;

#[async_trait]
impl PolicyGuard for SecurityScanPolicy {
    async fn evaluate(
        &self,
        context: &ContextSnapshot,
        scope: &ScopeLevel,
    ) -> Result<PolicyEvaluationResult> {
        // Collect code in scope
        let code_nodes = context.graph.nodes_in_scope(scope)?;

        // Run security checks (SAST)
        let violations = security_scanner.scan(&code_nodes)?;

        Ok(PolicyEvaluationResult {
            policy_id: PolicyId::SecurityScan,
            passed: violations.is_empty(),
            violations,
            evaluated_at: SystemTime::now(),
            context_snapshot_id: context.id.clone(),
            reason: format!("Found {} security issues", violations.len()),
        })
    }
}
```

### 3. Compensation ↔ Context: Rollback via Snapshots

**Architecture Correction 8**: `CompensationHandler` is an**infrastructure concern**(rollback, retry, logging), not business logic. It belongs in `mcb-infrastructure/src/compensation/handler.rs`. The**application layer**defines a `CompensationPolicy` port trait;**infrastructure** implements it.

```rust
// mcb-domain/src/ports/compensation.rs (PORT TRAIT - APPLICATION LAYER)
#[async_trait]
pub trait CompensationPolicy: Send + Sync {
    async fn execute(&self, action: &CompensationAction) -> Result<()>;
    async fn revalidate_after_compensation(&self, snapshot: &ContextSnapshot) -> Result<()>;
}

pub enum CompensationAction {
    DeleteBranch { branch_name: String },
    ResetCommit { target_commit: String },
    RestoreSnapshot { snapshot_id: ContextId },
    RollbackChanges { file_paths: Vec<String> },
    ManualReview { reason: String },
}

// mcb-infrastructure/src/compensation/handler.rs (IMPLEMENTATION - INFRASTRUCTURE LAYER)
pub struct CompensationHandler {
    context_store: Arc<dyn ContextRepository>,
    vcs_provider: Arc<dyn VcsProvider>,
    logger: Arc<dyn Logger>,
}

#[async_trait]
impl CompensationPolicy for CompensationHandler {
    async fn execute(&self, action: &CompensationAction) -> Result<()> {
        self.logger.info(&format!("Executing compensation: {:?}", action));

        match action {
            CompensationAction::RestoreSnapshot { snapshot_id } => {
                // Time-travel to previous context
                let snapshot = self.context_store.snapshot(snapshot_id).await?;

                // Revert code to match snapshot
                self.vcs_provider.reset_to_commit(
                    &snapshot.vcs_state.commit_hash
                ).await?;

                // Revalidate policies on restored context
                self.revalidate_after_compensation(&snapshot).await?;

                self.logger.info("Snapshot restoration succeeded");
                Ok(())
            },
            CompensationAction::DeleteBranch { branch_name } => {
                self.vcs_provider.delete_branch(branch_name).await?;
                self.logger.info(&format!("Branch deleted: {}", branch_name));
                Ok(())
            },
            _ => {
                self.logger.warn("Unsupported compensation action");
                Err(Error::UnsupportedCompensation)
            },
        }
    }

    async fn revalidate_after_compensation(&self, snapshot: &ContextSnapshot) -> Result<()> {
        // Re-check all policies after compensation
        let results = policies.evaluate_all(snapshot).await?;

        if results.iter().any(|r| !r.passed) {
            self.logger.error("Policies still failing after compensation");
            return Err(CompensationError::StillFailing {
                violations: results.iter()
                    .filter(|r| !r.passed)
                    .collect(),
            });
        }

        self.logger.info("All policies passing after compensation");
        Ok(())
    }
}
```

### 4. Event-Driven Orchestration

**Architecture Correction 2**: Reuse existing `EventBusProvider` port trait from mcb-domain instead of creating a new `WorkflowEventBus` type. Define `WorkflowEvent` as a variant that can be published through the existing event bus infrastructure.

```rust
// mcb-domain/src/ports/event_bus.rs (EXISTING PORT TRAIT)
#[async_trait]
pub trait EventBusProvider: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> Result<()>;
    async fn subscribe(&self, handler: Arc<dyn EventHandler>) -> Result<()>;
}

// mcb-domain/src/entities/workflow_event.rs (NEW VARIANT)
pub enum WorkflowEvent {
    // FSM transitions
    StateTransitioned { from: ExecutionState, to: ExecutionState, timestamp: SystemTime },

    // Context changes
    ContextRefreshed { snapshot_id: ContextId, freshness: ContextFreshness },
    ContextStale { snapshot_id: ContextId, reason: String },
    ContextInvalidated { reason: StalenessSignal },

    // Policy checks
    PolicyEvaluated { result: PolicyEvaluationResult },
    PolicyViolation { policy_id: PolicyId, severity: Severity },

    // Compensation
    CompensationTriggered { action: CompensationAction, reason: String },
    CompensationSucceeded { action: CompensationAction },
    CompensationFailed { action: CompensationAction, error: String },

    // Session lifecycle
    SessionStarted { session_id: SessionId, initial_context: ContextId },
    SessionEnded { session_id: SessionId, final_context: ContextId },
}

// Implement DomainEvent trait for WorkflowEvent
impl DomainEvent for WorkflowEvent {
    fn event_type(&self) -> &'static str {
        "WorkflowEvent"
    }
}

// Usage: Publish through existing EventBusProvider
pub async fn on_state_transition(
    from: &ExecutionState,
    to: &ExecutionState,
    event_bus: Arc<dyn EventBusProvider>,
) -> Result<()> {
    event_bus.publish(DomainEvent::Workflow(WorkflowEvent::StateTransitioned {
        from: from.clone(),
        to: to.clone(),
        timestamp: SystemTime::now(),
    })).await
}

// Subscribers implement EventHandler trait (EXISTING PATTERN)
pub struct CompensationSubscriber {
    handler: Arc<CompensationHandler>,
}

#[async_trait]
impl EventHandler for CompensationSubscriber {
    async fn handle(&self, event: &DomainEvent) -> Result<()> {
        if let DomainEvent::Workflow(WorkflowEvent::PolicyViolation { policy_id, severity }) = event {
            if *severity == Severity::Critical {
                self.handler.trigger_compensation(policy_id).await?;
            }
        }
        Ok(())
    }
}
```

### 5. MCP Tools: Unified Interface

**Architecture Correction 9**: Context tools registration follows**ADR-033 pattern** . Handlers are in `mcb-server/src/handlers/context_handlers.rs`, registered via `router.rs` tool_definitions() like existing handlers.

```rust
// mcb-domain/src/ports/mcp_handler.rs (PORT TRAIT - ADR-033)
#[async_trait]
pub trait MpcHandler: Send + Sync {
    fn resource_type(&self) -> &'static str;
    async fn handle(&self, action: &str, input: serde_json::Value) -> Result<serde_json::Value>;
}

// mcb-server/src/handlers/context_handlers.rs (HANDLER - ADR-033)
pub struct ContextHandler {
    search_engine: Arc<HybridSearchEngine>,
    context_store: Arc<dyn ContextRepository>,
    policy_guard: Arc<PolicyGuard>,
}

#[async_trait]
impl MpcHandler for ContextHandler {
    fn resource_type(&self) -> &'static str {
        "context"
    }

    async fn handle(&self, action: &str, input: serde_json::Value) -> Result<serde_json::Value> {
        match action {
            "search" => self.handle_search(input).await,
            "snapshot" => self.handle_snapshot(input).await,
            "timeline" => self.handle_timeline(input).await,
            "validate" => self.handle_validate(input).await,
            _ => Err(Error::UnknownAction(action.to_string())),
        }
    }
}

impl ContextHandler {
    async fn handle_search(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let query: String = input.get("query")?.as_str()?.into();
        let task_id: String = input.get("task_id")?.as_str()?.into();
        let filters: SearchFilters = serde_json::from_value(input.get("filters")?)?;

        // Implementation (same as before)
        let results = self.search_engine.search(&query, &filters).await?;
        Ok(serde_json::to_value(results)?)
    }

    async fn handle_snapshot(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let id: String = input.get("id")?.as_str()?.into();
        let snapshot = self.context_store.snapshot(&id).await?;
        Ok(serde_json::to_value(snapshot)?)
    }

    async fn handle_timeline(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let task_id: String = input.get("task_id")?.as_str()?.into();
        let start: SystemTime = serde_json::from_value(input.get("start")?)?;
        let end: SystemTime = serde_json::from_value(input.get("end")?)?;

        let timeline = self.context_store.timeline(&task_id, start, end).await?;
        Ok(serde_json::to_value(timeline)?)
    }

    async fn handle_validate(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let snapshot_id: String = input.get("snapshot_id")?.as_str()?.into();
        let policies: Vec<PolicyId> = serde_json::from_value(input.get("policies")?)?;

        let snapshot = self.context_store.snapshot(&snapshot_id).await?;
        let results = futures::join_all(
            policies.iter()
                .map(|p| self.policy_guard.evaluate(&snapshot, p))
        ).await;

        Ok(serde_json::to_value(results?)?)
    }
}

// mcb-server/src/router.rs (REGISTRATION - ADR-033)
pub fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        // ... existing tools ...
        ToolDefinition {
            name: "context_search".to_string(),
            description: "Search context with semantic + BM25 fusion".to_string(),
            resource: "context".to_string(),
            action: "search".to_string(),
        },
        ToolDefinition {
            name: "context_snapshot".to_string(),
            description: "Retrieve context snapshot at point in time".to_string(),
            resource: "context".to_string(),
            action: "snapshot".to_string(),
        },
        ToolDefinition {
            name: "context_timeline".to_string(),
            description: "Get context changes over time range".to_string(),
            resource: "context".to_string(),
            action: "timeline".to_string(),
        },
        ToolDefinition {
            name: "context_validate".to_string(),
            description: "Validate context against policies".to_string(),
            resource: "context".to_string(),
            action: "validate".to_string(),
        },
    ]
}

impl ContextToolHandler {
    pub async fn handle(&self, tool: &ContextTool) -> Result<ToolResult> {
        match tool {
            ContextTool::ContextSearch { query, task_id, filters } => {
                // **Architecture Correction 7**: BeadsTask is an EXTERNAL DTO from the beads issue tracker.
                // It must be mapped to an internal WorkflowTask entity at the infrastructure boundary.

                // 1. Load external BeadsTask DTO from Beads issue tracker
                let beads_task = self.beads_client.get_task(task_id).await?;

                // 2. Map external DTO → internal entity at infrastructure boundary
                let workflow_task = self.task_adapter.adapt_beads_to_workflow(beads_task).await?;

                // 3. Get current context snapshot
                let context = self.context_store.get_current().await?;

                // 4. Validate freshness for task
                if context.freshness == ContextFreshness::StaleWithRisk {
                    // Trigger context refresh
                    let new_context = self.refresh_context(&workflow_task).await?;
                    context = new_context;
                }

                // 5. Route search by task type (using internal WorkflowTask)
                let routed_results = self.route_search(&context, &workflow_task, query).await?;

                // 6. Return results with provenance
                Ok(ToolResult::SearchResults(routed_results))
            },

            ContextTool::ContextValidate { snapshot_id, policies } => {
                let snapshot = self.context_store.snapshot(snapshot_id).await?;

                // Evaluate requested policies
                let results = futures::join_all(
                    policies.iter()
                        .map(|p| self.policy_guard.evaluate(&snapshot, p))
                ).await;

                Ok(ToolResult::ValidationResults {
                    snapshot_id: snapshot_id.clone(),
                    policy_results: results?,
                    compensation_needed: results.iter().any(|r| !r.passed),
                })
            },

            _ => Err(Error::UnknownTool),
        }
    }
}
```

## Architecture Corrections

Applied Corrections (v0.4.0 Alignment):

1. **Correction 2 (mcb-z1f)**: WorkflowEventBus → Reuse EventBusProvider

- ✅ Removed duplicate `WorkflowEventBus` type
- ✅ Defined `WorkflowEvent` as variant publishable through existing `EventBusProvider` port
- ✅ Subscribers implement `EventHandler` trait (existing pattern)
- **Impact**: Single event bus infrastructure, no duplication

1. **Correction 7 (mcb-d26)**: BeadsTask contract clarification

- ✅ Documented BeadsTask as EXTERNAL DTO from beads issue tracker
- ✅ Added mapping: external BeadsTask → internal WorkflowTask at infrastructure boundary
- ✅ Task routing: external DTO → adapter → internal entity → orchestrator
- **Impact**: Clear contract, proper separation of concerns

1. **Correction 8 (mcb-ehk)**: CompensationHandler layer placement

- ✅ Moved from application to infrastructure: `mcb-infrastructure/src/compensation/handler.rs`
- ✅ Application layer defines `CompensationPolicy` port trait
- ✅ Infrastructure implements `CompensationPolicy` with rollback, retry, logging
- **Impact**: Proper layer separation, infrastructure concerns isolated

1. **Correction 9 (mcb-tmg)**: MCP tool registration pattern

- ✅ Replaced with ADR-033 ConsolidatedHandler pattern
- ✅ Handlers in `mcb-server/src/handlers/context_handlers.rs`
- ✅ Registration via `router.rs` tool_definitions() like existing handlers
- ✅ Match on action/resource for unified dispatch
- **Impact**: Consistent with existing MCP handler architecture

## Integration Checklist

- ✅ FSM state determines context freshness requirements
- ✅ Policies enforce scope boundaries (ScopeLevel)
- ✅ Compensation uses context snapshots for rollback
- ✅ Events published for all major state changes (via EventBusProvider)
- ✅ MCP tools provide unified query interface (ADR-033 pattern)
- ✅ Beads task context flows through all layers (with proper DTO mapping)

## Testing

- **State transition tests** (8): Freshness checks, policy validation
- **Compensation tests** (6): Rollback correctness, policy re-evaluation
- **Event flow tests** (5): Subscribers reactive, event ordering
- **Integration tests** (10): Full workflow + context + policies
- **MCP tool tests** (5): Tool handlers, Result accuracy

**Target**: 34+ tests, 80%+ coverage

### Success Criteria

- ✅ FSM ↔ Context validation working (state gates freshness)
- ✅ Policies enforced at all transition points
- ✅ Compensation triggers on policy failure + rolls back correctly
- ✅ Context snapshots enable time-travel recovery
- ✅ All workflow events published + logged
- ✅ MCP tools provide transparent access to all layers

---

## Architecture Completeness

ADR-041-046 form a complete system:

| ADR | Component | Status |
| ----- | ----------- | -------- |
| **041** | 5-layer architecture | ✅ Proposed |
| **042** | Knowledge graph | ✅ Proposed |
| **043** | Hybrid search | ✅ Proposed |
| **044** | Lightweight routing | ✅ Proposed |
| **045** | Versioning & freshness | ✅ Proposed |
| **046** | Policy integration | ✅ Proposed (THIS) |

> **v0.3.0 Migration Note:** This ADR describes v0.4.0-v0.5.0 future work. The current v0.3.0 architecture uses 4 layers (domain → providers → infrastructure → server).

All layers connected. Ready for implementation (Phase 9).

---

**Series Complete**: ADR-041-046 provides production-grade integrated context system for MCB v0.4.0.
