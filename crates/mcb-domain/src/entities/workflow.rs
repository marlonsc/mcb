//! Workflow FSM entities for session state management and transitions.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!
//!
//! This module implements the finite state machine (FSM) for workflow orchestration.
//! It defines the various states, transition triggers, and audit records required
//! to manage the lifecycle of an agent-led workflow session.

use chrono::Utc;
use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Workflow session states. Each variant carries context-specific data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
#[serde(tag = "state", content = "data")]
pub enum WorkflowState {
    /// Initial state when a workflow session is created.
    #[display("initializing")]
    Initializing,
    /// State when the project context is loaded and ready.
    #[display("ready")]
    Ready {
        /// Identifier of the loaded context.
        context_id: String,
    },
    /// State when planning a specific phase.
    #[display("planning")]
    Planning {
        /// Identifier of the phase being planned.
        phase_id: String,
    },
    /// State when executing tasks within a phase.
    #[display("executing")]
    Executing {
        /// Identifier of the phase being executed.
        phase_id: String,
        /// Optional identifier of the task currently being worked on.
        task_id: Option<String>,
    },
    /// State when verifying the results of a phase.
    #[display("verifying")]
    Verifying {
        /// Identifier of the phase being verified.
        phase_id: String,
    },
    /// State when a phase has been successfully completed.
    #[display("phase_complete")]
    PhaseComplete {
        /// Identifier of the completed phase.
        phase_id: String,
    },
    /// Terminal state indicating the workflow session finished successfully.
    #[display("completed")]
    Completed,
    /// Terminal state indicating the workflow failed.
    #[display("failed")]
    Failed {
        /// Error message describing the failure.
        error: String,
        /// Whether the error can be recovered from.
        recoverable: bool,
    },
}

impl WorkflowState {
    /// Returns the human-readable name of the current state.
    #[must_use]
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
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Checks if the state represents an error condition.
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

/// Events that trigger state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "trigger")]
pub enum TransitionTrigger {
    /// Event when project context is successfully discovered.
    #[display("ContextDiscovered")]
    ContextDiscovered {
        /// Identifier of the discovered context.
        context_id: String,
    },
    /// Event to begin planning for a phase.
    #[display("StartPlanning")]
    StartPlanning {
        /// Identifier of the phase to plan.
        phase_id: String,
    },
    /// Event to start executing tasks in a phase.
    #[display("StartExecution")]
    StartExecution {
        /// Identifier of the phase to execute.
        phase_id: String,
    },
    /// Event when a task is claimed for work.
    #[display("ClaimTask")]
    ClaimTask {
        /// Identifier of the claimed task.
        task_id: String,
    },
    /// Event when a task is completed.
    #[display("CompleteTask")]
    CompleteTask {
        /// Identifier of the completed task.
        task_id: String,
    },
    /// Event to start the verification process.
    #[display("StartVerification")]
    StartVerification,
    /// Event when verification succeeds.
    #[display("VerificationPassed")]
    VerificationPassed,
    /// Event when verification fails.
    #[display("VerificationFailed")]
    VerificationFailed {
        /// Reason why verification failed.
        reason: String,
    },
    /// Event to mark the entire phase as complete.
    #[display("CompletePhase")]
    CompletePhase,
    /// Event to end the workflow session.
    #[display("EndSession")]
    EndSession,
    /// Event indicating an error occurred.
    #[display("Error")]
    Error {
        /// Error message for the transition.
        message: String,
    },
    /// Event to attempt recovery from an error state.
    #[display("Recover")]
    Recover,
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
    /// Timestamp when the transition occurred (Unix epoch seconds).
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Input payload for creating a transition record.
pub struct TransitionInput {
    /// Unique identifier for the transition record.
    pub id: String,
    /// Session identifier where the transition happened.
    pub session_id: String,
    /// Previous workflow state.
    pub from_state: WorkflowState,
    /// Next workflow state.
    pub to_state: WorkflowState,
    /// Trigger that caused the transition.
    pub trigger: TransitionTrigger,
    /// Optional guard evaluation result.
    pub guard_result: Option<String>,
}

impl Transition {
    /// Creates a new transition audit record from a structured input.
    #[must_use]
    pub fn new(input: TransitionInput) -> Self {
        Self {
            id: input.id,
            session_id: input.session_id,
            from_state: input.from_state,
            to_state: input.to_state,
            trigger: input.trigger,
            guard_result: input.guard_result,
            timestamp: Utc::now().timestamp(),
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
    /// Stores the created at value.
    pub created_at: i64,
    /// Stores the updated at value.
    pub updated_at: i64,
    /// Version number for optimistic concurrency control.
    pub version: u32,
}

impl WorkflowSession {
    /// Creates a new workflow session in Initializing state.
    #[must_use]
    pub fn new(id: String, project_id: String) -> Self {
        let now = Utc::now().timestamp();
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
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current_state.is_terminal()
    }

    /// Checks if the session is in an error state.
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.current_state.is_error()
    }
}
