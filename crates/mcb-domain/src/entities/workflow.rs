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
    /// Initial state when a workflow session is created.
    Initializing,
    /// State when the project context is loaded and ready.
    Ready {
        /// Identifier of the loaded context.
        context_id: String,
    },
    /// State when planning a specific phase.
    Planning {
        /// Identifier of the phase being planned.
        phase_id: String,
    },
    /// State when executing tasks within a phase.
    Executing {
        /// Identifier of the phase being executed.
        phase_id: String,
        /// Optional identifier of the task currently being worked on.
        task_id: Option<String>,
    },
    /// State when verifying the results of a phase.
    Verifying {
        /// Identifier of the phase being verified.
        phase_id: String,
    },
    /// State when a phase has been successfully completed.
    PhaseComplete {
        /// Identifier of the completed phase.
        phase_id: String,
    },
    /// Terminal state indicating the workflow session finished successfully.
    Completed,
    /// Terminal state indicating the workflow failed.
    Failed {
        /// Error message describing the failure.
        error: String,
        /// Whether the error can be recovered from.
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
    /// Returns the human-readable name of the current state.
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

    /// Checks if the state is a terminal state (Completed).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Checks if the state represents an error condition.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

/// Events that trigger state transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "trigger")]
pub enum TransitionTrigger {
    /// Event when project context is successfully discovered.
    ContextDiscovered { context_id: String },
    /// Event to begin planning for a phase.
    StartPlanning { phase_id: String },
    /// Event to start executing tasks in a phase.
    StartExecution { phase_id: String },
    /// Event when a task is claimed for work.
    ClaimTask { task_id: String },
    /// Event when a task is completed.
    CompleteTask { task_id: String },
    /// Event to start the verification process.
    StartVerification,
    /// Event when verification succeeds.
    VerificationPassed,
    /// Event when verification fails.
    VerificationFailed { reason: String },
    /// Event to mark the entire phase as complete.
    CompletePhase,
    /// Event to end the workflow session.
    EndSession,
    /// Event indicating an error occurred.
    Error { message: String },
    /// Event to attempt recovery from an error state.
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
    /// Unique identifier for the transition record.
    pub id: String,
    /// Identifier of the session where this transition occurred.
    pub session_id: String,
    /// State before the transition.
    pub from_state: WorkflowState,
    /// State after the transition.
    pub to_state: WorkflowState,
    /// The event that triggered the transition.
    pub trigger: TransitionTrigger,
    /// Result of any guard condition check (optional).
    pub guard_result: Option<String>,
    /// Timestamp when the transition occurred.
    pub timestamp: DateTime<Utc>,
}

impl Transition {
    /// Creates a new transition record.
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
    /// Unique identifier for the session.
    pub id: String,
    /// Identifier of the project this session belongs to.
    pub project_id: String,
    /// Current state of the workflow.
    pub current_state: WorkflowState,
    /// Timestamp when the session was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the session was last updated.
    pub updated_at: DateTime<Utc>,
    /// Version number for optimistic concurrency control.
    pub version: u32,
}

impl WorkflowSession {
    /// Creates a new workflow session in Initializing state.
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

    /// Checks if the session is in a terminal state.
    pub fn is_complete(&self) -> bool {
        self.current_state.is_terminal()
    }

    /// Checks if the session is in an error state.
    pub fn is_error(&self) -> bool {
        self.current_state.is_error()
    }
}
