use mcb_domain::entities::user::{User, UserRole};
use rstest::rstest;

#[rstest]
fn user_construction() {
    let user = User {
        id: "usr-001".to_owned(),
        org_id: "org-001".to_owned(),
        email: "alice@acme.com".to_owned(),
        display_name: "Alice".to_owned(),
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

#[rstest]
fn user_serialization_roundtrip() {
    let user = User {
        id: "usr-002".to_owned(),
        org_id: "org-001".to_owned(),
        email: "bob@acme.com".to_owned(),
        display_name: "Bob".to_owned(),
        role: UserRole::Member,
        api_key_hash: Some("$argon2id$...".to_owned()),
        created_at: 2000,
        updated_at: 3000,
    };
    let json = serde_json::to_string(&user).expect("serialize");
    let deserialized: User = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "usr-002");
    assert_eq!(deserialized.role, UserRole::Member);
    assert!(deserialized.api_key_hash.is_some());
}

#[rstest]
#[case(UserRole::Admin, "admin")]
#[case(UserRole::Member, "member")]
#[case(UserRole::Viewer, "viewer")]
#[case(UserRole::Service, "service")]
fn user_role_as_str(#[case] role: UserRole, #[case] expected: &str) {
    assert_eq!(role.as_str(), expected);
}

#[rstest]
#[case("admin", Ok(UserRole::Admin))]
#[case("member", Ok(UserRole::Member))]
#[case("viewer", Ok(UserRole::Viewer))]
#[case("service", Ok(UserRole::Service))]
#[case("ADMIN", Ok(UserRole::Admin))]
#[case("Service", Ok(UserRole::Service))]
#[case("invalid", Err(()))]
fn user_role_from_str(#[case] input: &str, #[case] expected: Result<UserRole, ()>) {
    match expected {
        Ok(role) => assert_eq!(input.parse::<UserRole>(), Ok(role)),
        Err(()) => assert!(input.parse::<UserRole>().is_err()),
    }
}
