//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! FSM transition logic and state validation rules for ADR-034 Workflow.
//!
//! Implements the state machine rules that govern valid transitions
//! and side effects for the 8-state workflow model.

use mcb_domain::entities::{TransitionTrigger, WorkflowSession, WorkflowState};

type Result<T> = std::result::Result<T, String>;

/// Apply a transition trigger to a workflow session, validating FSM rules.
///
/// Returns the target state if transition is valid, error otherwise.
///
/// # Errors
///
/// Returns an error string if the transition is not valid from the current state.
pub fn apply_transition(
    session: &mut WorkflowSession,
    trigger: &TransitionTrigger,
) -> Result<WorkflowState> {
    // Terminal state — no transitions allowed
    if matches!(session.current_state, WorkflowState::Completed) {
        return Err("Cannot transition from terminal state Completed".to_owned());
    }

    // Global error transition: any state → Failed
    if let TransitionTrigger::Error { message } = trigger {
        return Ok(WorkflowState::Failed {
            error: message.clone(),
            recoverable: true,
        });
    }

    resolve_transition(&session.current_state, trigger)
}

/// Resolve a non-error, non-terminal transition.
fn resolve_transition(state: &WorkflowState, trigger: &TransitionTrigger) -> Result<WorkflowState> {
    match (state, trigger) {
        // Initializing → Ready
        (WorkflowState::Initializing, TransitionTrigger::ContextDiscovered { context_id }) => {
            Ok(WorkflowState::Ready {
                context_id: context_id.clone(),
            })
        }

        // Ready | PhaseComplete → Planning
        (
            WorkflowState::Ready { .. } | WorkflowState::PhaseComplete { .. },
            TransitionTrigger::StartPlanning { phase_id },
        ) => Ok(WorkflowState::Planning {
            phase_id: phase_id.clone(),
        }),

        // → Executing (from Ready, Planning, Executing, Verifying)
        (state, trigger) if is_executing_transition(state, trigger) => {
            Ok(resolve_executing_state(state, trigger))
        }

        // Executing → Verifying
        (WorkflowState::Executing { phase_id, .. }, TransitionTrigger::StartVerification) => {
            Ok(WorkflowState::Verifying {
                phase_id: phase_id.clone(),
            })
        }

        // Verifying → PhaseComplete
        (WorkflowState::Verifying { phase_id }, TransitionTrigger::VerificationPassed) => {
            Ok(WorkflowState::PhaseComplete {
                phase_id: phase_id.clone(),
            })
        }

        // PhaseComplete | Failed → Completed
        (
            WorkflowState::PhaseComplete { .. } | WorkflowState::Failed { .. },
            TransitionTrigger::EndSession,
        ) => Ok(WorkflowState::Completed),

        // Failed → Executing (recovery)
        (WorkflowState::Failed { .. }, TransitionTrigger::Recover) => {
            Ok(WorkflowState::Executing {
                phase_id: "unknown".to_owned(),
                task_id: None,
            })
        }

        // Invalid transition
        (from, trigger) => Err(format!(
            "Invalid FSM transition: {from} + {trigger} not allowed"
        )),
    }
}

/// Check if this is a transition that results in `Executing` state.
fn is_executing_transition(state: &WorkflowState, trigger: &TransitionTrigger) -> bool {
    matches!(
        (state, trigger),
        (
            WorkflowState::Ready { .. },
            TransitionTrigger::StartExecution { .. }
        ) | (
            WorkflowState::Planning { .. },
            TransitionTrigger::StartExecution { .. }
        ) | (
            WorkflowState::Executing { .. },
            TransitionTrigger::CompleteTask { .. }
        ) | (
            WorkflowState::Executing { .. },
            TransitionTrigger::ClaimTask { .. }
        ) | (
            WorkflowState::Verifying { .. },
            TransitionTrigger::VerificationFailed { .. }
        )
    )
}

/// Resolve the specific `Executing` variant based on the trigger.
fn resolve_executing_state(state: &WorkflowState, trigger: &TransitionTrigger) -> WorkflowState {
    let phase_id = match (state, trigger) {
        (_, TransitionTrigger::StartExecution { phase_id })
        | (
            WorkflowState::Executing { phase_id, .. }
            | WorkflowState::Planning { phase_id }
            | WorkflowState::Verifying { phase_id },
            _,
        ) => phase_id.clone(),
        _ => "unknown".to_owned(),
    };

    let task_id = if let TransitionTrigger::ClaimTask { task_id } = trigger {
        Some(task_id.clone())
    } else {
        None
    };

    WorkflowState::Executing { phase_id, task_id }
}
