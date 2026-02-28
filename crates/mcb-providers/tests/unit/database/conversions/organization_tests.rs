//! Tests for organization conversion.

use mcb_domain::entities::Organization;
use mcb_providers::database::seaorm::entities::organization;

fn sample_organization() -> organization::Model {
    organization::Model {
        id: "organization_test_001".into(),
        name: "test_name".into(),
        slug: "test-organization".into(),
        settings_json: "{}".into(),
        created_at: 1_700_000_000,
        updated_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_organization() {
    let model = sample_organization();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: Organization = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: organization::ActiveModel = domain.into();
}
