//! Tests for `project` conversion.

use mcb_domain::entities::Project;
use mcb_providers::database::seaorm::entities::project;
use rstest::rstest;

fn sample_project() -> project::Model {
    project::Model {
        id: "project_test_001".into(),
        org_id: "ref_org_id_001".into(),
        name: "test_name".into(),
        path: "/tmp/test-path".into(),
        created_at: 1_700_000_000,
        updated_at: 1_700_000_000,
    }
}

#[rstest]
#[test]
fn round_trip_project() {
    let model = sample_project();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: Project = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: project::ActiveModel = domain.into();
}
