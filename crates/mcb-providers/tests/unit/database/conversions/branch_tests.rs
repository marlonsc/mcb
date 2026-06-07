//! Tests for `branch` conversion.

use mcb_domain::entities::Branch;
use mcb_providers::database::seaorm::entities::branch;
use rstest::rstest;

fn sample_branch() -> branch::Model {
    branch::Model {
        id: "branch_test_001".into(),
        org_id: "ref_org_id_001".into(),
        project_id: None,
        repository_id: "ref_repository_id_001".into(),
        name: "test_name".into(),
        is_default: 1,
        head_commit: "test_head_commit".into(),
        upstream: Some("test_upstream".into()),
        origin_context: None,
        created_at: 1_700_000_000,
    }
}

#[rstest]
#[test]
fn round_trip_branch() {
    let model = sample_branch();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: Branch = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: branch::ActiveModel = domain.into();
}
