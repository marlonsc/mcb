//! Tests for project_dependency conversion.

use mcb_domain::entities::ProjectDependency;
use mcb_providers::database::seaorm::entities::project_dependency;

fn sample_project_dependency() -> project_dependency::Model {
    project_dependency::Model {
        id: "project_dependency_test_001".into(),
        from_issue_id: "ref_from_issue_id_001".into(),
        to_issue_id: "ref_to_issue_id_001".into(),
        dependency_type: "RelatesTo".into(),
        created_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_project_dependency() {
    let model = sample_project_dependency();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: ProjectDependency = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: project_dependency::ActiveModel = domain.into();
}
