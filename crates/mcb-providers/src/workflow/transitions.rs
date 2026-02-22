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
    let new_state = match (&session.current_state, trigger) {
        // Initializing → Ready (must have context)
        (WorkflowState::Initializing, TransitionTrigger::ContextDiscovered { context_id }) => {
            WorkflowState::Ready {
                context_id: context_id.clone(),
            }
        }

        // Ready | PhaseComplete → Planning
        (WorkflowState::Ready { .. }, TransitionTrigger::StartPlanning { phase_id })
        | (WorkflowState::PhaseComplete { .. }, TransitionTrigger::StartPlanning { phase_id }) => {
            WorkflowState::Planning {
                phase_id: phase_id.clone(),
            }
        }

        // Ready → Executing (skip planning) | Planning → Executing |
        // Executing → Executing (complete task) | Verifying → Executing (verification failed)
        (WorkflowState::Ready { .. }, TransitionTrigger::StartExecution { phase_id })
        | (WorkflowState::Planning { phase_id }, TransitionTrigger::StartExecution { .. })
        | (WorkflowState::Executing { phase_id, .. }, TransitionTrigger::CompleteTask { .. })
        | (WorkflowState::Verifying { phase_id }, TransitionTrigger::VerificationFailed { .. }) => {
            WorkflowState::Executing {
                phase_id: phase_id.clone(),
                task_id: None,
            }
        }

        // Executing → Executing (claim task)
        (WorkflowState::Executing { phase_id, .. }, TransitionTrigger::ClaimTask { task_id }) => {
            WorkflowState::Executing {
                phase_id: phase_id.clone(),
                task_id: Some(task_id.clone()),
            }
        }

        // Executing → Verifying
        (WorkflowState::Executing { phase_id, .. }, TransitionTrigger::StartVerification) => {
            WorkflowState::Verifying {
                phase_id: phase_id.clone(),
            }
        }

        // Verifying → PhaseComplete (verification passed)
        (WorkflowState::Verifying { phase_id }, TransitionTrigger::VerificationPassed) => {
            WorkflowState::PhaseComplete {
                phase_id: phase_id.clone(),
            }
        }

        // PhaseComplete → Completed
        (WorkflowState::PhaseComplete { .. }, TransitionTrigger::EndSession) => {
            WorkflowState::Completed
        }

        // Any state → Failed (error trigger)
        (_, TransitionTrigger::Error { message }) => WorkflowState::Failed {
            error: message.clone(),
            recoverable: true,
        },

        // Failed → Executing (recovery)
        (WorkflowState::Failed { .. }, TransitionTrigger::Recover) => {
            // Return to last known good executing state
            WorkflowState::Executing {
                phase_id: "unknown".to_owned(),
                task_id: None,
            }
        }

        // Failed → Completed (give up)
        (WorkflowState::Failed { .. }, TransitionTrigger::EndSession) => WorkflowState::Completed,

        // Completed → (no transitions allowed)
        (WorkflowState::Completed, _) => {
            return Err("Cannot transition from terminal state Completed".to_owned());
        }

        // Invalid transition
        (from, trigger) => {
            return Err(format!(
                "Invalid FSM transition: {from} + {trigger} not allowed"
            ));
        }
    };

    Ok(new_state)
}
