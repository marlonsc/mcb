use mcb_domain::entities::api_key::ApiKey;
use rstest::{fixture, rstest};

#[fixture]
fn default_api_key() -> ApiKey {
    ApiKey {
        id: "key-001".to_owned(),
        user_id: "usr-001".to_owned(),
        org_id: "org-001".to_owned(),
        key_hash: "$argon2id$v=19$m=65536...".to_owned(),
        name: "CI pipeline".to_owned(),
        scopes_json: r#"["read:code","write:memory"]"#.to_owned(),
        expires_at: Some(1800000000),
        created_at: 1000,
        revoked_at: None,
    }
}

#[rstest]
fn test_api_key_construction(default_api_key: ApiKey) {
    assert_eq!(default_api_key.id, "key-001");
    assert_eq!(default_api_key.name, "CI pipeline");
}

#[rstest]
fn test_api_key_serialization_roundtrip(mut default_api_key: ApiKey) {
    default_api_key.name = "dev laptop".to_owned();
    default_api_key.expires_at = None;

    let json = serde_json::to_string(&default_api_key).expect("serialize");
    let deserialized: ApiKey = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, default_api_key.id);
    assert_eq!(deserialized.name, "dev laptop");
    assert!(deserialized.expires_at.is_none());
}

#[rstest]
fn test_api_key_revocation(mut default_api_key: ApiKey) {
    default_api_key.revoked_at = Some(2000);
    assert!(default_api_key.revoked_at.is_some());
    assert_eq!(default_api_key.revoked_at.unwrap(), 2000);
}
