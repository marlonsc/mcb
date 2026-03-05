use mcb_domain::entities::organization::{OrgStatus, Organization};
use rstest::{fixture, rstest};

#[fixture]
fn default_org() -> Organization {
    Organization {
        id: "org-001".to_owned(),
        name: "Acme Corp".to_owned(),
        slug: "acme-corp".to_owned(),
        settings_json: "{}".to_owned(),
        created_at: 1000,
        updated_at: 1000,
    }
}

#[rstest]
fn test_organization_construction(default_org: Organization) {
    assert_eq!(default_org.id, "org-001");
    assert_eq!(default_org.name, "Acme Corp");
}

#[rstest]
fn test_organization_serialization_roundtrip(mut default_org: Organization) {
    default_org.settings_json = r#"{"max_projects":10}"#.to_owned();
    let json = serde_json::to_string(&default_org).expect("serialize");
    let deserialized: Organization = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, default_org.id);
    assert!(deserialized.settings_json.contains("10"));
}

#[rstest]
#[case(OrgStatus::Active, "active")]
#[case(OrgStatus::Suspended, "suspended")]
#[case(OrgStatus::Archived, "archived")]
fn test_org_status_as_str(#[case] status: OrgStatus, #[case] expected: &str) {
    assert_eq!(status.as_str(), expected);
}

#[rstest]
#[case("active", Ok(OrgStatus::Active))]
#[case("suspended", Ok(OrgStatus::Suspended))]
#[case("archived", Ok(OrgStatus::Archived))]
#[case("ACTIVE", Ok(OrgStatus::Active))]
#[case("Suspended", Ok(OrgStatus::Suspended))]
#[case("invalid", Err(()))]
fn test_org_status_from_str(#[case] input: &str, #[case] expected: Result<OrgStatus, ()>) {
    match expected {
        Ok(status) => assert_eq!(input.parse::<OrgStatus>(), Ok(status)),
        Err(()) => assert!(input.parse::<OrgStatus>().is_err()),
    }
}
