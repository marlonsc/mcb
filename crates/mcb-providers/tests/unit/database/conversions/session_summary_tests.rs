//! Tests for `session_summary` conversion.

use mcb_domain::entities::memory::SessionSummary;
use mcb_providers::database::seaorm::entities::session_summary;
use rstest::rstest;

fn sample_session_summary() -> session_summary::Model {
    session_summary::Model {
        id: "session_summary_test_001".into(),
        org_id: Some("test_unwrap".into()),
        project_id: "ref_project_id_001".into(),
        repo_id: None,
        session_id: "ref_session_id_001".into(),
        topics: Some(r#"["tag1","tag2"]"#.into()),
        decisions: Some(r#"["tag1","tag2"]"#.into()),
        next_steps: Some(r#"["tag1","tag2"]"#.into()),
        key_files: Some(r#"["tag1","tag2"]"#.into()),
        origin_context: Some(r#"{"key":"val"}"#.into()),
        created_at: 1_700_000_000,
    }
}

#[rstest]
#[test]
fn round_trip_session_summary() {
    let model = sample_session_summary();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: SessionSummary = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: session_summary::ActiveModel = domain.into();
}
