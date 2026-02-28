//! Tests for observation conversion.

use mcb_domain::entities::Observation;
use mcb_providers::database::seaorm::entities::observation;

fn sample_observation() -> observation::Model {
    observation::Model {
        id: "observation_test_001".into(),
        project_id: "ref_project_id_001".into(),
        content: "test_content".into(),
        content_hash: "hash_observation_001".into(),
        tags: Some(r#"["tag1","tag2"]"#.into()),
        observation_type: Some("Context".into()),
        metadata: Some(r#"{"key":"val"}"#.into()),
        created_at: 1_700_000_000,
        embedding_id: Some("test_embedding_id".into()),
    }
}

#[test]
fn round_trip_observation() {
    let model = sample_observation();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: Observation = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: observation::ActiveModel = domain.into();
}
