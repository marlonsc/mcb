//! Unit tests for org context value object.

use mcb_domain::constants::keys::{DEFAULT_ORG_ID, DEFAULT_ORG_NAME};
use mcb_domain::utils::id;
use mcb_domain::value_objects::{OrgContext, OrgId};
use rstest::rstest;

#[rstest]
fn default_org_context_uses_bootstrap_id() {
    let ctx = OrgContext::default();
    // Verify it is a valid UUID and matches deterministic expectation
    assert_eq!(
        ctx.org_id,
        OrgId::from_uuid(id::deterministic("org", DEFAULT_ORG_ID))
    );
    assert_eq!(ctx.org_name, DEFAULT_ORG_NAME);
}

#[rstest]
fn custom_org_context() {
    let org_uuid = id::deterministic("org", "org-123");
    let ctx = OrgContext::new(OrgId::from_uuid(org_uuid), "acme".to_owned());
    assert_eq!(ctx.org_id.to_string(), org_uuid.to_string());
    assert_eq!(ctx.org_name, "acme");
}
