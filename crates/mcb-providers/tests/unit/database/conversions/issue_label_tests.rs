//! Tests for `issue_label` conversion.

use mcb_domain::entities::IssueLabel;
use mcb_providers::database::seaorm::entities::issue_label;

fn sample_issue_label() -> issue_label::Model {
    issue_label::Model {
        id: "issue_label_test_001".into(),
        org_id: "ref_org_id_001".into(),
        project_id: "ref_project_id_001".into(),
        name: "test_name".into(),
        color: "test_color".into(),
        created_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_issue_label() {
    let model = sample_issue_label();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: IssueLabel = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: issue_label::ActiveModel = domain.into();
}
