//! Workflow FSM entities for session state management and transitions.
//!
//! Implements the workflow finite state machine defined in ADR-034.
//! Supports 8 states with typed state data and 11 transition triggers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Workflow session states. Each variant carries context-specific data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", content = "data")]
pub enum WorkflowState {
    Initializing,
    Ready {
        context_id: String,
    },
    Planning {
        phase_id: String,
    },
    Executing {
        phase_id: String,
        task_id: Option<String>,
    },
    Verifying {
        phase_id: String,
    },
    PhaseComplete {
        phase_id: String,
    },
    Completed,
    Failed {
        error: String,
        recoverable: bool,
    },
}

impl fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    pub fn name(&self) -> &'static str {
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

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

/// Events that trigger state transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "trigger")]
pub enum TransitionTrigger {
    ContextDiscovered { context_id: String },
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
}

impl fmt::Display for TransitionTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl Transition {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSession {
    pub id: String,
    pub project_id: String,
    pub current_state: WorkflowState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

impl WorkflowSession {
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

    pub fn is_complete(&self) -> bool {
        self.current_state.is_terminal()
    }

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
                context_id: "ctx-1".to_string()
            }
            .to_string(),
            "ready"
        );
    }

    #[test]
    fn workflow_state_is_terminal() {
        assert!(!WorkflowState::Initializing.is_terminal());
        assert!(WorkflowState::Completed.is_terminal());
    }

    #[test]
    fn workflow_session_new() {
        let session = WorkflowSession::new("sess-1".to_string(), "proj-1".to_string());
        assert_eq!(session.id, "sess-1");
        assert_eq!(session.current_state, WorkflowState::Initializing);
        assert!(!session.is_complete());
    }
}
