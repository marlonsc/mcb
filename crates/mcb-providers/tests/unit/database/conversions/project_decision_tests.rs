//! Tests for `project_decision` conversion.

use mcb_domain::entities::ProjectDecision;
use mcb_providers::database::seaorm::entities::project_decision;
use rstest::rstest;

fn sample_project_decision() -> project_decision::Model {
    project_decision::Model {
        id: "project_decision_test_001".into(),
        project_id: "ref_project_id_001".into(),
        issue_id: Some("test_issue_id".into()),
        title: "test_title".into(),
        context: "test_context".into(),
        decision: "test_decision".into(),
        consequences: "test_consequences".into(),
        created_at: 1_700_000_000,
    }
}

#[rstest]
#[test]
fn round_trip_project_decision() {
    let model = sample_project_decision();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: ProjectDecision = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: project_decision::ActiveModel = domain.into();
}
