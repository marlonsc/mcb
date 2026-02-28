//! Tests for `repository` conversion.

use mcb_domain::entities::Repository;
use mcb_providers::database::seaorm::entities::repository;

fn sample_repository() -> repository::Model {
    repository::Model {
        id: "repository_test_001".into(),
        org_id: "ref_org_id_001".into(),
        project_id: "ref_project_id_001".into(),
        name: "test_name".into(),
        url: "https://example.com/repo".into(),
        local_path: "/tmp/test-path".into(),
        vcs_type: "Git".into(),
        origin_context: None,
        created_at: 1_700_000_000,
        updated_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_repository() {
    let model = sample_repository();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: Repository = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: repository::ActiveModel = domain.into();
}
