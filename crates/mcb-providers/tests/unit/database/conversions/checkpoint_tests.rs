//! Tests for `checkpoint` conversion.

use mcb_domain::entities::Checkpoint;
use mcb_providers::database::seaorm::entities::checkpoint;
use rstest::rstest;

fn sample_checkpoint() -> checkpoint::Model {
    checkpoint::Model {
        id: "checkpoint_test_001".into(),
        session_id: "ref_session_id_001".into(),
        checkpoint_type: "File".into(),
        description: "test_description".into(),
        snapshot_data: r#"{"test":true}"#.into(),
        created_at: mcb_utils::constants::testing::TEST_TIMESTAMP,
        restored_at: Some(mcb_utils::constants::testing::TEST_TIMESTAMP),
        expired: Some(1),
    }
}

#[rstest]
fn round_trip_checkpoint() {
    let model = sample_checkpoint();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: Checkpoint = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: checkpoint::ActiveModel = domain.into();
}
