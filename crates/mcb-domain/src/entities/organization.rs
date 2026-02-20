//! Organization entity — the root tenant for multi-tenant isolation.
//!
//! **Documentation**: [`docs/modules/domain.md#core-entities-entities`](../../../../docs/modules/domain.md#core-entities-entities)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// An organization is the top-level tenant. Every user, team, project,
/// and piece of data belongs to exactly one organization. Row-level
/// isolation in the database is enforced via `org_id` foreign keys.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Organization {
    /// Unique identifier (UUID).
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// URL-safe slug for routing and display (e.g. "acme-corp").
    pub slug: String,
    /// Arbitrary JSON settings (quotas, feature flags, etc.).
    pub settings_json: String,
    /// Timestamp when the organization was created (Unix epoch).
    pub created_at: i64,
    /// Timestamp when the organization was last updated (Unix epoch).
    pub updated_at: i64,
}

/// Status of an organization in its lifecycle.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    strum_macros::AsRefStr,
    strum_macros::Display,
    strum_macros::EnumString,
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum OrgStatus {
    /// Organization is active and operational.
    Active,
    /// Organization is suspended (e.g. billing issue).
    Suspended,
    /// Organization has been archived / soft-deleted.
    Archived,
}

impl OrgStatus {
    /// Returns the string representation of the organization status.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
