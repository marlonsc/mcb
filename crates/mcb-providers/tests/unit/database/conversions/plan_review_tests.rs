//! Tests for plan_review conversion.

use mcb_domain::entities::PlanReview;
use mcb_providers::database::seaorm::entities::plan_review;

fn sample_plan_review() -> plan_review::Model {
    plan_review::Model {
        id: "plan_review_test_001".into(),
        org_id: "ref_org_id_001".into(),
        plan_version_id: "ref_plan_version_id_001".into(),
        reviewer_id: "ref_reviewer_id_001".into(),
        verdict: "NeedsRevision".into(),
        feedback: "test_feedback".into(),
        created_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_plan_review() {
    let model = sample_plan_review();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: PlanReview = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: plan_review::ActiveModel = domain.into();
}
