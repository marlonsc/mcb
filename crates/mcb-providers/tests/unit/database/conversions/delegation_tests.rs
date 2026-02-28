//! Tests for delegation conversion.

use mcb_domain::entities::Delegation;
use mcb_providers::database::seaorm::entities::delegation;

fn sample_delegation() -> delegation::Model {
    delegation::Model {
        id: "delegation_test_001".into(),
        parent_session_id: "ref_parent_session_id_001".into(),
        child_session_id: "ref_child_session_id_001".into(),
        prompt: "test_prompt".into(),
        prompt_embedding_id: Some("test_prompt_embedding_id".into()),
        result: Some("test_result".into()),
        success: 1,
        created_at: 1_700_000_000,
        completed_at: Some(1_700_000_000),
        duration_ms: Some(1500),
    }
}

#[test]
fn round_trip_delegation() {
    let model = sample_delegation();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: Delegation = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: delegation::ActiveModel = domain.into();
}
