//! Tests for project_phase conversion.

use mcb_domain::entities::ProjectPhase;
use mcb_providers::database::seaorm::entities::project_phase;

fn sample_project_phase() -> project_phase::Model {
    project_phase::Model {
        id: "project_phase_test_001".into(),
        project_id: "ref_project_id_001".into(),
        name: "test_name".into(),
        description: "test_description".into(),
        sequence: 2,
        status: "Planned".into(),
        started_at: Some(1_700_000_000),
        completed_at: Some(1_700_000_000),
        created_at: 1_700_000_000,
        updated_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_project_phase() {
    let model = sample_project_phase();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: ProjectPhase = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: project_phase::ActiveModel = domain.into();
}
