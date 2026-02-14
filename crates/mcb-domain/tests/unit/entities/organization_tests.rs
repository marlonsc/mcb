use mcb_domain::entities::organization::{OrgStatus, Organization};
use rstest::rstest;

#[rstest]
fn organization_construction() {
    let org = Organization {
        id: "org-001".to_string(),
        name: "Acme Corp".to_string(),
        slug: "acme-corp".to_string(),
        settings_json: "{}".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    assert_eq!(org.id, "org-001");
    assert_eq!(org.name, "Acme Corp");
    assert_eq!(org.slug, "acme-corp");
    assert_eq!(org.settings_json, "{}");
}

#[rstest]
fn organization_serialization_roundtrip() {
    let org = Organization {
        id: "org-002".to_string(),
        name: "Test Org".to_string(),
        slug: "test-org".to_string(),
        settings_json: r#"{"max_projects":10}"#.to_string(),
        created_at: 2000,
        updated_at: 3000,
    };
    let json = serde_json::to_string(&org).expect("serialize");
    let deserialized: Organization = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "org-002");
    assert_eq!(deserialized.slug, "test-org");
}

#[rstest]
#[case(OrgStatus::Active, "active")]
#[case(OrgStatus::Suspended, "suspended")]
#[case(OrgStatus::Archived, "archived")]
fn org_status_as_str(#[case] status: OrgStatus, #[case] expected: &str) {
    assert_eq!(status.as_str(), expected);
}

#[rstest]
#[case("active", Ok(OrgStatus::Active))]
#[case("suspended", Ok(OrgStatus::Suspended))]
#[case("archived", Ok(OrgStatus::Archived))]
#[case("ACTIVE", Ok(OrgStatus::Active))]
#[case("Suspended", Ok(OrgStatus::Suspended))]
#[case("invalid", Err(()))]
fn org_status_from_str(#[case] input: &str, #[case] expected: Result<OrgStatus, ()>) {
    match expected {
        Ok(status) => assert_eq!(input.parse::<OrgStatus>(), Ok(status)),
        Err(()) => assert!(input.parse::<OrgStatus>().is_err()),
    }
}
