//! Tests for worktree conversion.

use mcb_domain::entities::Worktree;
use mcb_providers::database::seaorm::entities::worktree;

fn sample_worktree() -> worktree::Model {
    worktree::Model {
        id: "worktree_test_001".into(),
        org_id: None,
        project_id: None,
        repository_id: "ref_repository_id_001".into(),
        branch_id: "ref_branch_id_001".into(),
        path: "/tmp/test-path".into(),
        status: "Active".into(),
        assigned_agent_id: Some("test_assigned_agent_id".into()),
        origin_context: None,
        created_at: 1_700_000_000,
        updated_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_worktree() {
    let model = sample_worktree();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: Worktree = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: worktree::ActiveModel = domain.into();
}
