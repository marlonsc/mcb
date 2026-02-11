use mcb_domain::entities::organization::{OrgStatus, Organization};

#[test]
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

#[test]
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

#[test]
fn org_status_as_str() {
    assert_eq!(OrgStatus::Active.as_str(), "active");
    assert_eq!(OrgStatus::Suspended.as_str(), "suspended");
    assert_eq!(OrgStatus::Archived.as_str(), "archived");
}

#[test]
fn org_status_from_str() {
    assert_eq!("active".parse::<OrgStatus>(), Ok(OrgStatus::Active));
    assert_eq!("suspended".parse::<OrgStatus>(), Ok(OrgStatus::Suspended));
    assert_eq!("archived".parse::<OrgStatus>(), Ok(OrgStatus::Archived));
    assert!("invalid".parse::<OrgStatus>().is_err());
}

#[test]
fn org_status_from_str_case_insensitive() {
    assert_eq!("ACTIVE".parse::<OrgStatus>(), Ok(OrgStatus::Active));
    assert_eq!("Suspended".parse::<OrgStatus>(), Ok(OrgStatus::Suspended));
}
