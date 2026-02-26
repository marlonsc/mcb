//! Organization entity — the root tenant for multi-tenant isolation.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

crate::define_entity! {
    /// An organization is the top-level tenant. Every user, team, project,
    /// and piece of data belongs to exactly one organization. Row-level
    /// isolation in the database is enforced via `org_id` foreign keys.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Organization { id, created_at, updated_at } {
        /// Readable name of the organization.
        pub name: String,
        /// URL-friendly identifier for the organization.
        pub slug: String,
        /// JSON-encoded settings for the organization.
        pub settings_json: String,
    }
}

/// Status of an organization in its lifecycle.
crate::define_string_enum! {
    /// Status of an organization in its lifecycle.
    pub enum OrgStatus [strum = "lowercase", schema] {
        /// Organization is active and operational.
        Active,
        /// Organization is suspended (e.g. billing issue).
        Suspended,
        /// Organization has been archived / soft-deleted.
        Archived,
    }
}
