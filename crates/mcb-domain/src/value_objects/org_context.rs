use serde::{Deserialize, Serialize};

use super::ids::OrgId;
use crate::constants::keys::{DEFAULT_ORG_ID, DEFAULT_ORG_NAME};

/// Tenant context for row-level isolation.
///
/// Carried through the service layer so every repository query
/// is scoped to a single organization. Phase 0 bootstraps with
/// [`DEFAULT_ORG_ID`]; Phase 1 adds real org management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgContext {
    /// Stores the org id value.
    pub org_id: OrgId,
    /// Stores the org name value.
    pub org_name: String,
}

impl OrgContext {
    /// Creates a new instance.
    #[must_use]
    pub fn new(org_id: OrgId, org_name: String) -> Self {
        Self { org_id, org_name }
    }

    /// Returns the org id as a string.
    #[must_use]
    pub fn id_str(&self) -> String {
        self.org_id.to_string()
    }
}

impl Default for OrgContext {
    fn default() -> Self {
        Self {
            org_id: OrgId::from_uuid(crate::utils::id::deterministic("org", DEFAULT_ORG_ID)),
            org_name: DEFAULT_ORG_NAME.to_owned(),
        }
    }
}
