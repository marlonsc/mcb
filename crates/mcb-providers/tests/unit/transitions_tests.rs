//! Unit tests for workflow FSM transitions.

use mcb_domain::entities::{TransitionTrigger, WorkflowSession, WorkflowState};
use mcb_providers::workflow::apply_transition;

#[test]
fn test_initializing_to_ready() {
    let mut session = WorkflowSession::new("s1".to_string(), "p1".to_string());
    let trigger = TransitionTrigger::ContextDiscovered {
        context_id: "ctx-1".to_string(),
    };

    let new_state = apply_transition(&mut session, trigger).expect("transition failed");

    match new_state {
        WorkflowState::Ready { context_id } => assert_eq!(context_id, "ctx-1"),
        _ => panic!("Expected Ready state"),
    }
}

#[test]
fn test_ready_to_planning() {
    let mut session = WorkflowSession::new("s1".to_string(), "p1".to_string());
    session.current_state = WorkflowState::Ready {
        context_id: "ctx-1".to_string(),
    };

    let trigger = TransitionTrigger::StartPlanning {
        phase_id: "phase-1".to_string(),
    };
    let new_state = apply_transition(&mut session, trigger).expect("transition failed");

    match new_state {
        WorkflowState::Planning { phase_id } => assert_eq!(phase_id, "phase-1"),
        _ => panic!("Expected Planning state"),
    }
}

#[test]
fn test_terminal_state_no_transitions() {
    let mut session = WorkflowSession::new("s1".to_string(), "p1".to_string());
    session.current_state = WorkflowState::Completed;

    let trigger = TransitionTrigger::EndSession;
    let result = apply_transition(&mut session, trigger);

    assert!(result.is_err());
}
