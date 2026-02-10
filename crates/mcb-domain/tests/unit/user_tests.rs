use mcb_domain::entities::user::{User, UserRole};

#[test]
fn user_construction() {
    let user = User {
        id: "usr-001".to_string(),
        org_id: "org-001".to_string(),
        email: "alice@acme.com".to_string(),
        display_name: "Alice".to_string(),
        role: UserRole::Admin,
        api_key_hash: None,
        created_at: 1000,
        updated_at: 1000,
    };
    assert_eq!(user.id, "usr-001");
    assert_eq!(user.org_id, "org-001");
    assert_eq!(user.email, "alice@acme.com");
    assert_eq!(user.role, UserRole::Admin);
    assert!(user.api_key_hash.is_none());
}

#[test]
fn user_serialization_roundtrip() {
    let user = User {
        id: "usr-002".to_string(),
        org_id: "org-001".to_string(),
        email: "bob@acme.com".to_string(),
        display_name: "Bob".to_string(),
        role: UserRole::Member,
        api_key_hash: Some("$argon2id$...".to_string()),
        created_at: 2000,
        updated_at: 3000,
    };
    let json = serde_json::to_string(&user).expect("serialize");
    let deserialized: User = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "usr-002");
    assert_eq!(deserialized.role, UserRole::Member);
    assert!(deserialized.api_key_hash.is_some());
}

#[test]
fn user_role_as_str() {
    assert_eq!(UserRole::Admin.as_str(), "admin");
    assert_eq!(UserRole::Member.as_str(), "member");
    assert_eq!(UserRole::Viewer.as_str(), "viewer");
    assert_eq!(UserRole::Service.as_str(), "service");
}

#[test]
fn user_role_from_str() {
    assert_eq!("admin".parse::<UserRole>(), Ok(UserRole::Admin));
    assert_eq!("member".parse::<UserRole>(), Ok(UserRole::Member));
    assert_eq!("viewer".parse::<UserRole>(), Ok(UserRole::Viewer));
    assert_eq!("service".parse::<UserRole>(), Ok(UserRole::Service));
    assert!("invalid".parse::<UserRole>().is_err());
}

#[test]
fn user_role_from_str_case_insensitive() {
    assert_eq!("ADMIN".parse::<UserRole>(), Ok(UserRole::Admin));
    assert_eq!("Service".parse::<UserRole>(), Ok(UserRole::Service));
}
