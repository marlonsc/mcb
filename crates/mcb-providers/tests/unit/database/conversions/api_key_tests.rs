//! Tests for `api_key` conversion.

use mcb_domain::entities::ApiKey;
use mcb_providers::database::seaorm::entities::api_key;

fn sample_api_key() -> api_key::Model {
    api_key::Model {
        id: "api_key_test_001".into(),
        user_id: "ref_user_id_001".into(),
        org_id: "ref_org_id_001".into(),
        key_hash: "hash_api_key_001".into(),
        name: "test_name".into(),
        scopes_json: "{}".into(),
        expires_at: Some(1_700_000_000),
        created_at: 1_700_000_000,
        revoked_at: Some(1_700_000_000),
    }
}

#[test]
fn round_trip_api_key() {
    let model = sample_api_key();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: ApiKey = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: api_key::ActiveModel = domain.into();
}
