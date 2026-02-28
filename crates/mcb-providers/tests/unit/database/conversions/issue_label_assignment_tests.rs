//! Tests for issue_label_assignment conversion.

use mcb_domain::entities::IssueLabelAssignment;
use mcb_providers::database::seaorm::entities::issue_label_assignment;

fn sample_issue_label_assignment() -> issue_label_assignment::Model {
    issue_label_assignment::Model {
        issue_id: "ref_issue_id_001".into(),
        label_id: "ref_label_id_001".into(),
        created_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_issue_label_assignment() {
    let model = sample_issue_label_assignment();
    let model_val = model.issue_id.clone();

    // Model → Domain
    let domain: IssueLabelAssignment = model.into();
    assert_eq!(domain.issue_id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: issue_label_assignment::ActiveModel = domain.into();
}
