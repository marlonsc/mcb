//! Tests for `tool_call` conversion.

use mcb_domain::entities::ToolCall;
use mcb_providers::database::seaorm::entities::tool_call;
use rstest::rstest;

fn sample_tool_call() -> tool_call::Model {
    tool_call::Model {
        id: "tool_call_test_001".into(),
        org_id: None,
        project_id: None,
        repo_id: None,
        session_id: "ref_session_id_001".into(),
        tool_name: "test_tool_name".into(),
        params_summary: Some("test_params_summary".into()),
        success: 1,
        error_message: Some("test_error_message".into()),
        duration_ms: Some(1500),
        created_at: 1_700_000_000,
    }
}

#[rstest]
#[test]
fn round_trip_tool_call() {
    let model = sample_tool_call();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: ToolCall = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: tool_call::ActiveModel = domain.into();
}
