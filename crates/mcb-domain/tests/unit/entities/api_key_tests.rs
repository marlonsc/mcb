use mcb_domain::entities::api_key::ApiKey;
use rstest::rstest;

#[rstest]
fn api_key_construction() {
    let key = ApiKey {
        id: "key-001".to_string(),
        user_id: "usr-001".to_string(),
        org_id: "org-001".to_string(),
        key_hash: "$argon2id$v=19$m=65536...".to_string(),
        name: "CI pipeline".to_string(),
        scopes_json: r#"["read:code","write:memory"]"#.to_string(),
        expires_at: Some(1800000000),
        created_at: 1000,
        revoked_at: None,
    };
    assert_eq!(key.id, "key-001");
    assert_eq!(key.user_id, "usr-001");
    assert_eq!(key.org_id, "org-001");
    assert_eq!(key.name, "CI pipeline");
    assert!(key.expires_at.is_some());
    assert!(key.revoked_at.is_none());
}

#[rstest]
fn api_key_serialization_roundtrip() {
    let key = ApiKey {
        id: "key-002".to_string(),
        user_id: "usr-002".to_string(),
        org_id: "org-001".to_string(),
        key_hash: "hash-value".to_string(),
        name: "dev laptop".to_string(),
        scopes_json: "[]".to_string(),
        expires_at: None,
        created_at: 2000,
        revoked_at: None,
    };
    let json = serde_json::to_string(&key).expect("serialize");
    let deserialized: ApiKey = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "key-002");
    assert_eq!(deserialized.name, "dev laptop");
    assert!(deserialized.expires_at.is_none());
}

#[rstest]
fn api_key_revoked() {
    let key = ApiKey {
        id: "key-003".to_string(),
        user_id: "usr-001".to_string(),
        org_id: "org-001".to_string(),
        key_hash: "hash-value".to_string(),
        name: "old key".to_string(),
        scopes_json: "[]".to_string(),
        expires_at: None,
        created_at: 1000,
        revoked_at: Some(2000),
    };
    assert!(key.revoked_at.is_some());
    assert_eq!(key.revoked_at.unwrap(), 2000);
}
