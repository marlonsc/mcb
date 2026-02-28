//! Tests for plan conversion.

use mcb_domain::entities::Plan;
use mcb_providers::database::seaorm::entities::plan;

fn sample_plan() -> plan::Model {
    plan::Model {
        id: "plan_test_001".into(),
        org_id: "ref_org_id_001".into(),
        project_id: "ref_project_id_001".into(),
        title: "test_title".into(),
        description: "test_description".into(),
        status: "Draft".into(),
        created_by: "test_created_by".into(),
        created_at: 1_700_000_000,
        updated_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_plan() {
    let model = sample_plan();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: Plan = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: plan::ActiveModel = domain.into();
}
