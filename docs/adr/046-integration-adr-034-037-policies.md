# ADR-046: Integration with ADR-034-037 & Policies

**Status**: Proposed  
**Date**: 2026-02-05  
**Deciders**: MCB Architecture Team  
**Related**: ADR-034-037 (Workflow series), ADR-041-045 (Context system)  
**Series Finale**: Completes ADR-041-046 (v0.4.0 architecture)

## Context

ADR-041-045 define the integrated context system layers. ADR-046 **bridges** this to ADR-034-037 (workflow FSM, policies, compensation) and specifies **how they interact**.

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

```rust
pub struct CompensationHandler {
    context_store: Arc<dyn ContextRepository>,
    vcs_provider: Arc<dyn VcsProvider>,
}

pub enum CompensationAction {
    DeleteBranch { branch_name: String },
    ResetCommit { target_commit: String },
    RestoreSnapshot { snapshot_id: ContextId },
    RollbackChanges { file_paths: Vec<String> },
    ManualReview { reason: String },
}

impl CompensationHandler {
    pub async fn execute(&self, action: &CompensationAction) -> Result<()> {
        match action {
            CompensationAction::RestoreSnapshot { snapshot_id } => {
                // Time-travel to previous context
                let snapshot = self.context_store.snapshot(snapshot_id).await?;
                
                // Revert code to match snapshot
                self.vcs_provider.reset_to_commit(
                    &snapshot.vcs_state.commit_hash
                ).await?;
                
                // Revalidate policies on restored context
                self.revalidate_policies(&snapshot).await?;
                
                Ok(())
            },
            CompensationAction::DeleteBranch { branch_name } => {
                self.vcs_provider.delete_branch(branch_name).await?;
                Ok(())
            },
            _ => Err(Error::UnsupportedCompensation),
        }
    }
    
    async fn revalidate_policies(&self, snapshot: &ContextSnapshot) -> Result<()> {
        // Re-check all policies after compensation
        let results = policies.evaluate_all(snapshot).await?;
        
        if results.iter().any(|r| !r.passed) {
            return Err(CompensationError::StillFailing {
                violations: results.iter()
                    .filter(|r| !r.passed)
                    .collect(),
            });
        }
        
        Ok(())
    }
}
```

### 4. Event-Driven Orchestration

```rust
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

pub struct WorkflowEventBus {
    channel: tokio::sync::mpsc::UnboundedSender<WorkflowEvent>,
}

// Usage: Every significant event published
pub async fn on_state_transition(from: &ExecutionState, to: &ExecutionState) {
    event_bus.publish(WorkflowEvent::StateTransitioned {
        from: from.clone(),
        to: to.clone(),
        timestamp: SystemTime::now(),
    }).await;
}

// Subscribers can react:
pub struct CompensationSubscriber {
    handler: Arc<CompensationHandler>,
}

impl WorkflowEventSubscriber for CompensationSubscriber {
    async fn on_event(&self, event: &WorkflowEvent) {
        match event {
            WorkflowEvent::PolicyViolation { policy_id, severity } => {
                if *severity == Severity::Critical {
                    self.handler.trigger_compensation(policy_id).await;
                }
            },
            _ => {},
        }
    }
}
```

### 5. MCP Tools: Unified Interface

```rust
// mcb-server/handlers/context_handlers.rs

pub enum ContextTool {
    // Search
    ContextSearch {
        query: String,
        task_id: String,  // Links to Beads task
        filters: SearchFilters,
    },
    
    // Snapshots
    ContextSnapshot {
        id: String,
        timestamp: SystemTime,
    },
    
    // Timeline
    ContextTimeline {
        task_id: String,
        start: SystemTime,
        end: SystemTime,
    },
    
    // Validation
    ContextValidate {
        snapshot_id: String,
        policies: Vec<PolicyId>,
    },
}

pub struct ContextToolHandler {
    search_engine: Arc<HybridSearchEngine>,
    context_store: Arc<dyn ContextRepository>,
    policy_guard: Arc<PolicyGuard>,
}

impl ContextToolHandler {
    pub async fn handle(&self, tool: &ContextTool) -> Result<ToolResult> {
        match tool {
            ContextTool::ContextSearch { query, task_id, filters } => {
                // 1. Load task context from Beads
                let task = self.beads_client.get_task(task_id).await?;
                
                // 2. Get current context snapshot
                let context = self.context_store.get_current().await?;
                
                // 3. Validate freshness for task
                if context.freshness == ContextFreshness::StaleWithRisk {
                    // Trigger context refresh
                    let new_context = self.refresh_context(&task).await?;
                    context = new_context;
                }
                
                // 4. Route search by task type
                let routed_results = self.route_search(&context, &task, query).await?;
                
                // 5. Return results with provenance
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

## Integration Checklist

- ✅ FSM state determines context freshness requirements
- ✅ Policies enforce scope boundaries (ScopeLevel)
- ✅ Compensation uses context snapshots for rollback
- ✅ Events published for all major state changes
- ✅ MCP tools provide unified query interface
- ✅ Beads task context flows through all layers

## Testing

- **State transition tests** (8): Freshness checks, policy validation
- **Compensation tests** (6): Rollback correctness, policy re-evaluation
- **Event flow tests** (5): Subscribers reactive, event ordering
- **Integration tests** (10): Full workflow + context + policies
- **MCP tool tests** (5): Tool handlers, result accuracy

**Target**: 34+ tests, 80%+ coverage

## Success Criteria

- ✅ FSM ↔ Context validation working (state gates freshness)
- ✅ Policies enforced at all transition points
- ✅ Compensation triggers on policy failure + rolls back correctly
- ✅ Context snapshots enable time-travel recovery
- ✅ All workflow events published + logged
- ✅ MCP tools provide transparent access to all layers

---

## Architecture Completeness

**ADR-041-046 form a complete system:**

| ADR | Component | Status |
|-----|-----------|--------|
| **041** | 5-layer architecture | ✅ Proposed |
| **042** | Knowledge graph | ✅ Proposed |
| **043** | Hybrid search | ✅ Proposed |
| **044** | Lightweight routing | ✅ Proposed |
| **045** | Versioning & freshness | ✅ Proposed |
| **046** | Policy integration | ✅ Proposed (THIS) |

**All layers connected. Ready for implementation (Phase 9).**

---

**Series Complete**: ADR-041-046 provides production-grade integrated context system for MCB v0.4.0.
