//! Unit tests for org context value object.

use mcb_domain::constants::keys::{DEFAULT_ORG_ID, DEFAULT_ORG_NAME};
use mcb_domain::value_objects::{OrgContext, OrgId};

#[test]
fn default_org_context_uses_bootstrap_id() {
    let ctx = OrgContext::default();
    assert_eq!(ctx.org_id.as_str(), DEFAULT_ORG_ID);
    assert_eq!(ctx.org_name, DEFAULT_ORG_NAME);
}

#[test]
fn custom_org_context() {
    let ctx = OrgContext::new(OrgId::new("org-123"), "acme".to_string());
    assert_eq!(ctx.org_id.as_str(), "org-123");
    assert_eq!(ctx.org_name, "acme");
}
