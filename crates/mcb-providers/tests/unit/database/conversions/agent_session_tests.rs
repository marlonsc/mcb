//! Tests for `agent_session` conversion.

use mcb_domain::entities::AgentSession;
use mcb_providers::database::seaorm::entities::agent_session;

fn sample_agent_session() -> agent_session::Model {
    agent_session::Model {
        id: "agent_session_test_001".into(),
        project_id: Some("test_project_id".into()),
        worktree_id: Some("test_worktree_id".into()),
        session_summary_id: "ref_session_summary_id_001".into(),
        agent_type: "Sisyphus".into(),
        model: "test_model".into(),
        parent_session_id: Some("test_parent_session_id".into()),
        started_at: 1_700_000_000,
        ended_at: Some(1_700_000_000),
        duration_ms: Some(1500),
        status: "Active".into(),
        prompt_summary: Some("test_prompt_summary".into()),
        result_summary: Some("test_result_summary".into()),
        token_count: Some(1),
        tool_calls_count: Some(1),
        delegations_count: Some(1),
    }
}

#[test]
fn round_trip_agent_session() {
    let model = sample_agent_session();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: AgentSession = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: agent_session::ActiveModel = domain.into();
}
