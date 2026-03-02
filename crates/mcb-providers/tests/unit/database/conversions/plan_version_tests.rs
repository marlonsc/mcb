//! Tests for `plan_version` conversion.

use mcb_domain::entities::PlanVersion;
use mcb_providers::database::seaorm::entities::plan_version;
use rstest::rstest;

fn sample_plan_version() -> plan_version::Model {
    plan_version::Model {
        id: "plan_version_test_001".into(),
        org_id: "ref_org_id_001".into(),
        plan_id: "ref_plan_id_001".into(),
        version_number: 1,
        content_json: "{}".into(),
        change_summary: "test_change_summary".into(),
        created_by: "test_created_by".into(),
        created_at: 1_700_000_000,
    }
}

#[rstest]
#[test]
fn round_trip_plan_version() {
    let model = sample_plan_version();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: PlanVersion = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: plan_version::ActiveModel = domain.into();
}
