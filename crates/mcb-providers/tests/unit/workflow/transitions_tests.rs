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

#[rstest]
fn terminal_state_no_transitions() {
    let mut session = WorkflowSession::new("s1".to_owned(), "p1".to_owned());
    session.current_state = WorkflowState::Completed;

    let trigger = TransitionTrigger::EndSession;
    let result = apply_transition(&mut session, &trigger);

    let err = result.expect_err("terminal state should not allow end-session transition");
    let err_msg = err.clone();
    assert!(
        err_msg.to_lowercase().contains("transition"),
        "error should mention transition, got: {err_msg}"
    );
}

/// Recovery re-enters execution WITHOUT inventing a phase identity: the
/// phase-less `Executing` state is the honest record until a phase-carrying
/// trigger arrives (mcb-v8a6; replaces the fabricated "unknown" phase id).
#[rstest]
fn recover_from_failed_yields_phaseless_executing() {
    let mut session = WorkflowSession::new("s1".to_owned(), "p1".to_owned());
    session.current_state = WorkflowState::Failed {
        error: "transient".to_owned(),
        recoverable: true,
    };

    let new_state =
        apply_transition(&mut session, &TransitionTrigger::Recover).expect("recovery is valid");

    match new_state {
        WorkflowState::Executing { phase_id, task_id } => {
            assert_eq!(phase_id, None, "recovery must not fabricate a phase id");
            assert_eq!(task_id, None);
        }
        WorkflowState::Initializing
        | WorkflowState::Ready { .. }
        | WorkflowState::Planning { .. }
        | WorkflowState::Verifying { .. }
        | WorkflowState::PhaseComplete { .. }
        | WorkflowState::Completed
        | WorkflowState::Failed { .. } => panic!("Expected Executing after Recover"),
    }
}

/// Verification needs a declared phase to carry into `Verifying`; a phase-less
/// execution must be rejected instead of inventing an identity.
#[rstest]
fn verification_from_phaseless_executing_is_rejected() {
    let mut session = WorkflowSession::new("s1".to_owned(), "p1".to_owned());
    session.current_state = WorkflowState::Executing {
        phase_id: None,
        task_id: None,
    };

    let err = apply_transition(&mut session, &TransitionTrigger::StartVerification)
        .expect_err("phase-less execution cannot enter verification");

    assert!(
        err.to_lowercase().contains("phase"),
        "error should mention the missing phase, got: {err}"
    );
}

/// A phase-carrying trigger still declares its phase in the executing state.
#[rstest]
fn start_execution_carries_declared_phase() {
    let mut session = WorkflowSession::new("s1".to_owned(), "p1".to_owned());
    session.current_state = WorkflowState::Planning {
        phase_id: "phase-1".to_owned(),
    };

    let new_state = apply_transition(
        &mut session,
        &TransitionTrigger::StartExecution {
            phase_id: "phase-1".to_owned(),
        },
    )
    .expect("planning to executing is valid");

    match new_state {
        WorkflowState::Executing { phase_id, task_id } => {
            assert_eq!(phase_id.as_deref(), Some("phase-1"));
            assert_eq!(task_id, None);
        }
        WorkflowState::Initializing
        | WorkflowState::Ready { .. }
        | WorkflowState::Planning { .. }
        | WorkflowState::Verifying { .. }
        | WorkflowState::PhaseComplete { .. }
        | WorkflowState::Completed
        | WorkflowState::Failed { .. } => panic!("Expected Executing after StartExecution"),
    }
}

/// Phase-less execution round-trips through serialization without fabricating
/// data — persisted recovery state stays honest.
#[rstest]
fn phaseless_executing_state_round_trips() {
    let state = WorkflowState::Executing {
        phase_id: None,
        task_id: None,
    };

    let json = serde_json::to_string(&state).expect("state serializes");
    let back: WorkflowState = serde_json::from_str(&json).expect("state deserializes");

    assert_eq!(back, state);
    assert!(!json.contains("unknown"), "no sentinel may appear: {json}");
}
