//! Tests for user conversion.

use mcb_domain::entities::User;
use mcb_providers::database::seaorm::entities::user;

fn sample_user() -> user::Model {
    user::Model {
        id: "user_test_001".into(),
        org_id: "ref_org_id_001".into(),
        email: "test@example.com".into(),
        display_name: "test_display_name".into(),
        role: "Default".into(),
        api_key_hash: Some("test_api_key_hash".into()),
        created_at: 1_700_000_000,
        updated_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_user() {
    let model = sample_user();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: User = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: user::ActiveModel = domain.into();
}
