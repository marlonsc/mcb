//! Unit tests for workflow FSM transitions.

use mcb_domain::entities::{TransitionTrigger, WorkflowSession, WorkflowState};
use mcb_providers::workflow::apply_transition;
use rstest::rstest;

#[rstest]
#[case("initializing", "ctx-1", "", "ready")]
#[case("ready", "ctx-1", "phase-1", "planning")]
fn transition_happy_paths(
    #[case] from_state: &str,
    #[case] context_id: &str,
    #[case] phase_id: &str,
    #[case] expected_state: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut session = WorkflowSession::new("s1".to_owned(), "p1".to_owned());
    if from_state == "ready" {
        session.current_state = WorkflowState::Ready {
            context_id: context_id.to_owned(),
        };
    }

    let trigger = if from_state == "initializing" {
        TransitionTrigger::ContextDiscovered {
            context_id: context_id.to_owned(),
        }
    } else {
        TransitionTrigger::StartPlanning {
            phase_id: phase_id.to_owned(),
        }
    };

    let new_state = apply_transition(&mut session, &trigger)?;

    if expected_state == "ready" {
        match new_state {
            WorkflowState::Ready { context_id: got } => assert_eq!(got, context_id),
            WorkflowState::Initializing
            | WorkflowState::Planning { .. }
            | WorkflowState::Executing { .. }
            | WorkflowState::Verifying { .. }
            | WorkflowState::PhaseComplete { .. }
            | WorkflowState::Completed
            | WorkflowState::Failed { .. } => panic!("Expected Ready state"),
        }
    } else {
        match new_state {
            WorkflowState::Planning { phase_id: got } => assert_eq!(got, phase_id),
            WorkflowState::Initializing
            | WorkflowState::Ready { .. }
            | WorkflowState::Executing { .. }
            | WorkflowState::Verifying { .. }
            | WorkflowState::PhaseComplete { .. }
            | WorkflowState::Completed
            | WorkflowState::Failed { .. } => panic!("Expected Planning state"),
        }
    }
    Ok(())
}

#[test]
fn terminal_state_no_transitions() {
    let mut session = WorkflowSession::new("s1".to_owned(), "p1".to_owned());
    session.current_state = WorkflowState::Completed;

    let trigger = TransitionTrigger::EndSession;
    let result = apply_transition(&mut session, &trigger);

    assert!(result.is_err());
}
