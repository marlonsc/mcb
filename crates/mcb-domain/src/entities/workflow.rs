//! Workflow FSM entities for session state management and transitions.
//!
//! Implements the workflow finite state machine defined in ADR-034.
//! Supports 8 states with typed state data and 11 transition triggers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Workflow session states. Each variant carries context-specific data.
///
/// FSM Transition Graph:
/// ```text
/// Initializing
///     ├─ContextDiscovered─→ Ready
/// Ready
///     ├─StartPlanning──────→ Planning
///     └─StartExecution────→ Executing
/// Planning
///     └─StartExecution────→ Executing
/// Executing
///     ├─ClaimTask─────────→ Executing (same state, different task)
///     ├─CompleteTask──────→ Executing (same state, task cleared)
///     ├─StartVerification→ Verifying
///     └─Error─────────────→ Failed
/// Verifying
///     ├─VerificationPassed─→ PhaseComplete
///     └─VerificationFailed→ Executing (retry)
/// PhaseComplete
///     ├─StartPlanning─────→ Planning (next phase)
///     └─EndSession────────→ Completed
/// Completed
///     └─(terminal state)
/// Failed
///     ├─Recover──────────→ Executing
///     └─EndSession────────→ Completed
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", content = "data")]
pub enum WorkflowState {
    /// Session created, awaiting context discovery.
    Initializing,
    /// Context discovered, ready to plan or execute.
    Ready { context_snapshot_id: String },
    /// Planning phase in progress.
    Planning { phase_id: String },
    /// Executing tasks within a phase.
    Executing {
        phase_id: String,
        task_id: Option<String>,
    },
    /// Verifying phase completion.
    Verifying { phase_id: String },
    /// Phase completed, ready for next phase.
    PhaseComplete { phase_id: String },
    /// Session ended normally.
    Completed,
    /// Error state with recovery information.
    Failed { error: String, recoverable: bool },
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
        }
    }
}

impl WorkflowState {
    /// Get the state name as a string (useful for UI).
    pub fn name(&self) -> &str {
        match self {
            Self::Initializing => "Initializing",
            Self::Ready { .. } => "Ready",
            Self::Planning { .. } => "Planning",
            Self::Executing { .. } => "Executing",
            Self::Verifying { .. } => "Verifying",
            Self::PhaseComplete { .. } => "Phase Complete",
            Self::Completed => "Completed",
            Self::Failed { .. } => "Failed",
        }
    }

    /// Check if this state is terminal (no further transitions possible).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Check if this state is an error state.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

/// Events that trigger state transitions.
///
/// Each trigger maps to a specific transition rule validated by the FSM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "trigger", content = "data")]
pub enum TransitionTrigger {
    /// Context (project, memory, git) has been discovered.
    ContextDiscovered { context_snapshot_id: String },
    /// Start planning phase.
    StartPlanning { phase_id: String },
    /// Start executing phase (skip planning).
    StartExecution { phase_id: String },
    /// Claim a specific task within a phase.
    ClaimTask { task_id: String },
    /// Mark a task as complete.
    CompleteTask { task_id: String },
    /// Begin verification of phase results.
    StartVerification,
    /// Verification passed - phase is complete.
    VerificationPassed,
    /// Verification failed - need to retry execution.
    VerificationFailed { reason: String },
    /// Mark phase as complete and transition to next phase.
    CompletePhase,
    /// End workflow session.
    EndSession,
    /// Unrecoverable error occurred.
    Error { message: String },
    /// Attempt to recover from an error state.
    Recover,
}

impl std::fmt::Display for TransitionTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContextDiscovered { .. } => write!(f, "ContextDiscovered"),
            Self::StartPlanning { .. } => write!(f, "StartPlanning"),
            Self::StartExecution { .. } => write!(f, "StartExecution"),
            Self::ClaimTask { .. } => write!(f, "ClaimTask"),
            Self::CompleteTask { .. } => write!(f, "CompleteTask"),
            Self::StartVerification => write!(f, "StartVerification"),
            Self::VerificationPassed => write!(f, "VerificationPassed"),
            Self::VerificationFailed { .. } => write!(f, "VerificationFailed"),
            Self::CompletePhase => write!(f, "CompletePhase"),
            Self::EndSession => write!(f, "EndSession"),
            Self::Error { .. } => write!(f, "Error"),
            Self::Recover => write!(f, "Recover"),
        }
    }
}

/// Recorded transition with full audit context.
///
/// Immutable record of a state transition for audit trail and time-travel debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Unique transition ID.
    pub id: String,
    /// Session this transition belongs to.
    pub session_id: String,
    /// State before transition.
    pub from_state: WorkflowState,
    /// State after transition.
    pub to_state: WorkflowState,
    /// Trigger that caused this transition.
    pub trigger: TransitionTrigger,
    /// Optional result from guard evaluation (if any guards were checked).
    pub guard_result: Option<String>,
    /// When this transition occurred.
    pub timestamp: DateTime<Utc>,
}

impl Transition {
    /// Create a new transition record.
    pub fn new(
        id: String,
        session_id: String,
        from_state: WorkflowState,
        to_state: WorkflowState,
        trigger: TransitionTrigger,
        guard_result: Option<String>,
    ) -> Self {
        Self {
            id,
            session_id,
            from_state,
            to_state,
            trigger,
            guard_result,
            timestamp: Utc::now(),
        }
    }
}

/// Workflow session entity.
///
/// Represents a single workflow session with its current state and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSession {
    /// Unique session ID.
    pub id: String,
    /// Project this session belongs to.
    pub project_id: String,
    /// Current workflow state.
    pub current_state: WorkflowState,
    /// Session creation time.
    pub created_at: DateTime<Utc>,
    /// Last state update time.
    pub updated_at: DateTime<Utc>,
    /// Version for optimistic concurrency detection.
    pub version: u32,
}

impl WorkflowSession {
    /// Create a new workflow session in Initializing state.
    pub fn new(id: String, project_id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            project_id,
            current_state: WorkflowState::Initializing,
            created_at: now,
            updated_at: now,
            version: 0,
        }
    }

    /// Check if session is in a terminal state (no further transitions possible).
    pub fn is_complete(&self) -> bool {
        self.current_state.is_terminal()
    }

    /// Check if session is in an error state.
    pub fn is_error(&self) -> bool {
        self.current_state.is_error()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_state_display() {
        assert_eq!(WorkflowState::Initializing.to_string(), "initializing");
        assert_eq!(
            WorkflowState::Ready {
                context_snapshot_id: "ctx-1".to_string()
            }
            .to_string(),
            "ready"
        );
        assert_eq!(
            WorkflowState::Planning {
                phase_id: "phase-1".to_string()
            }
            .to_string(),
            "planning"
        );
    }

    #[test]
    fn workflow_state_is_terminal() {
        assert!(!WorkflowState::Initializing.is_terminal());
        assert!(WorkflowState::Completed.is_terminal());
    }

    #[test]
    fn workflow_state_is_error() {
        assert!(!WorkflowState::Initializing.is_error());
        assert!(
            WorkflowState::Failed {
                error: "test".to_string(),
                recoverable: true
            }
            .is_error()
        );
    }

    #[test]
    fn workflow_session_new() {
        let session = WorkflowSession::new("sess-1".to_string(), "proj-1".to_string());
        assert_eq!(session.id, "sess-1");
        assert_eq!(session.project_id, "proj-1");
        assert_eq!(session.current_state, WorkflowState::Initializing);
        assert_eq!(session.version, 0);
        assert!(!session.is_complete());
    }

    #[test]
    fn transition_serialization() {
        let transition = Transition::new(
            "trans-1".to_string(),
            "sess-1".to_string(),
            WorkflowState::Initializing,
            WorkflowState::Ready {
                context_snapshot_id: "ctx-1".to_string(),
            },
            TransitionTrigger::ContextDiscovered {
                context_snapshot_id: "ctx-1".to_string(),
            },
            None,
        );

        let json = serde_json::to_string(&transition).expect("serialization failed");
        let deserialized: Transition = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(transition.id, deserialized.id);
        assert_eq!(transition.from_state, deserialized.from_state);
        assert_eq!(transition.to_state, deserialized.to_state);
    }
}
