//! Tests for `project_issue` conversion.

use mcb_domain::entities::ProjectIssue;
use mcb_providers::database::seaorm::entities::project_issue;

fn sample_project_issue() -> project_issue::Model {
    project_issue::Model {
        id: "project_issue_test_001".into(),
        org_id: "ref_org_id_001".into(),
        project_id: "ref_project_id_001".into(),
        phase_id: Some("test_phase_id".into()),
        title: "test_title".into(),
        description: "test_description".into(),
        issue_type: "Task".into(),
        status: "Open".into(),
        priority: 2,
        assignee: Some("test_assignee".into()),
        labels: r#"["label-a"]"#.into(),
        created_at: 1_700_000_000,
        updated_at: 1_700_000_000,
        closed_at: Some(1_700_000_000),
        created_by: "test_created_by".into(),
        estimated_minutes: Some(1),
        actual_minutes: Some(1),
        notes: "test_notes".into(),
        design: "test_design".into(),
        parent_issue_id: Some("test_parent_issue_id".into()),
        closed_reason: "test_closed_reason".into(),
    }
}

#[test]
fn round_trip_project_issue() {
    let model = sample_project_issue();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: ProjectIssue = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: project_issue::ActiveModel = domain.into();
}
