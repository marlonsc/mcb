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
    pub fn new(org_id: OrgId, org_name: String) -> Self {
        Self { org_id, org_name }
    }

    /// Performs the current operation.
    pub fn current() -> Self {
        Self::default()
    }

    /// Performs the id str operation.
    pub fn id_str(&self) -> &str {
        self.org_id.as_str()
    }
}

impl Default for OrgContext {
    fn default() -> Self {
        Self {
            org_id: OrgId::new(DEFAULT_ORG_ID),
            org_name: DEFAULT_ORG_NAME.to_string(),
        }
    }
}
