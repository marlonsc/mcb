# MCB Minimal FSM Specification (v0.3.0)

This document defines the minimal viable Finite State Machine (FSM) for MCB workflow sessions. It simplifies the complex 12-state production model from ADR-034 into a focused 4-state core for initial implementation.

## 1. State Diagram

```text
          +----------+
          |   Idle   |
          +----------+
               |
            [Start]
               v
          +----------+    [Suspend]    +----------+
          |  Active  | --------------> |  Paused  |
          |          | <-------------- |          |
          +----------+     [Resume]    +----------+
               |
            [Finish]
               v
          +----------+
          | Complete |
          +----------+
```

## 2. State Definitions

| State | Description |
|-------|-------------|
| **Idle** | Initial state. Session created, no active work performing. |
| **Active** | Primary execution state. Operations are being performed. |
| **Paused** | Temporarily suspended. State is preserved but no work occurs. |
| **Complete** | Terminal success state. All goals reached. |

## 3. Transition Rules & Guards

Transitions are triggered by `WorkflowEvent` and must pass associated `Policy` guards.

| From | Trigger | To | Guards / Policies |
|------|---------|----|-------------------|
| Idle | `Start` | Active | **FreshnessPolicy**: Context must be < 5s old. |
| Active | `Suspend`| Paused | None (always allowed). |
| Paused | `Resume` | Active | **FreshnessPolicy**: Re-validate context. |
| Active | `Finish` | Complete | **ValidationPolicy**: All tasks must be closed. |

## 4. Policy Definitions

Policies are implemented as traits that evaluate the current `ProjectContext` against a transition.

### Freshness Policy
Ensures the agent is operating on up-to-date information.
- **Rule**: `context.last_updated < now - 5 seconds` → REJECT.

### Validation Policy (Minimal)
Ensures project integrity before marking as complete.
- **Rule**: `context.open_issues_count == 0` → ALLOW.

## 5. Rust Code Examples

### Workflow Entities

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowState {
    Idle,
    Active { started_at: chrono::DateTime<chrono::Utc> },
    Paused { reason: String },
    Complete,
}

pub enum WorkflowTrigger {
    Start,
    Suspend { reason: String },
    Resume,
    Finish,
}
```

### Workflow FSM

```rust
pub struct WorkflowFSM {
    state: WorkflowState,
}

impl WorkflowFSM {
    pub fn new() -> Self {
        Self { state: WorkflowState::Idle }
    }

    pub fn transition(&mut self, trigger: WorkflowTrigger) -> Result<(), String> {
        self.state = match (&self.state, trigger) {
            (WorkflowState::Idle, WorkflowTrigger::Start) => {
                WorkflowState::Active { started_at: chrono::Utc::now() }
            }
            (WorkflowState::Active { .. }, WorkflowTrigger::Suspend { reason }) => {
                WorkflowState::Paused { reason }
            }
            (WorkflowState::Paused { .. }, WorkflowTrigger::Resume) => {
                WorkflowState::Active { started_at: chrono::Utc::now() }
            }
            (WorkflowState::Active { .. }, WorkflowTrigger::Finish) => {
                WorkflowState::Complete
            }
            (s, t) => return Err(format!("Invalid transition from {:?} via {:?}", s, t)),
        };
        Ok(())
    }
}
```

### Policy Trait

```rust
#[async_trait::async_trait]
pub trait Policy: Send + Sync {
    fn name(&self) -> &str;
    async fn evaluate(&self, context: &ProjectContext) -> PolicyResult;
}

pub struct FreshnessPolicy {
    pub max_age_secs: u64,
}

#[async_trait::async_trait]
impl Policy for FreshnessPolicy {
    fn name(&self) -> &str { "freshness" }
    
    async fn evaluate(&self, context: &ProjectContext) -> PolicyResult {
        let age = chrono::Utc::now() - context.last_sync;
        if age.num_seconds() > self.max_age_secs as i64 {
            PolicyResult::deny("Context is stale", "Run 'mcb sync' to refresh")
        } else {
            PolicyResult::pass()
        }
    }
}
```
