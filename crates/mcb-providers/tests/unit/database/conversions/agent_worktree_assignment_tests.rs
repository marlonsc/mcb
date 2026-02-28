//! Tests for `agent_worktree_assignment` conversion.

use mcb_domain::entities::AgentWorktreeAssignment;
use mcb_providers::database::seaorm::entities::agent_worktree_assignment;

fn sample_agent_worktree_assignment() -> agent_worktree_assignment::Model {
    agent_worktree_assignment::Model {
        id: "agent_worktree_assignment_test_001".into(),
        agent_session_id: "ref_agent_session_id_001".into(),
        worktree_id: "ref_worktree_id_001".into(),
        assigned_at: 1_700_000_000,
        released_at: Some(1_700_000_000),
        origin_context: None,
    }
}

#[test]
fn round_trip_agent_worktree_assignment() {
    let model = sample_agent_worktree_assignment();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: AgentWorktreeAssignment = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: agent_worktree_assignment::ActiveModel = domain.into();
}
