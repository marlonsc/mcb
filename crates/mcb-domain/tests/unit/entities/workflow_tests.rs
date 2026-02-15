//! Unit tests for workflow entities (`WorkflowState`, `WorkflowSession`).

use mcb_domain::entities::{WorkflowSession, WorkflowState};
use rstest::rstest;

#[rstest]
fn test_workflow_state_display() {
    assert_eq!(WorkflowState::Initializing.to_string(), "initializing");
    assert_eq!(
        WorkflowState::Ready {
            context_id: "ctx-1".to_owned()
        }
        .to_string(),
        "ready"
    );
}

#[rstest]
fn test_workflow_state_is_terminal() {
    assert!(!WorkflowState::Initializing.is_terminal());
    assert!(WorkflowState::Completed.is_terminal());
}

#[rstest]
fn test_workflow_session_new() {
    let session = WorkflowSession::new("sess-1".to_owned(), "proj-1".to_owned());
    assert_eq!(session.id, "sess-1");
    assert_eq!(session.current_state, WorkflowState::Initializing);
    assert!(!session.is_complete());
}
