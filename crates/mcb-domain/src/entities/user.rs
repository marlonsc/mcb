//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

crate::define_entity! {
    /// Represents a user within the system.
    ///
    /// Users are associated with an organization and have specific roles that
    /// determine their permissions.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct User { id, org_id, created_at, updated_at } {
        /// Email address of the user.
        pub email: String,
        /// Name to be displayed for the user.
        pub display_name: String,
        /// Role assigned to the user within the organization.
        pub role: UserRole,
        /// Hashed API key for the user, if applicable.
        pub api_key_hash: Option<String>,
    }
}

crate::define_string_enum! {
    /// Role a user holds within an organization.
    pub enum UserRole [strum = "lowercase", schema] {
        /// Full administrative access.
        Admin,
        /// Standard member with read/write access.
        Member,
        /// Read-only viewer.
        Viewer,
        /// Service account (API-only, used by agents).
        Service,
    }
}

// Manual impl because `define_string_enum!` does not derive Default for all enums.
#[allow(clippy::derivable_impls)]
impl Default for UserRole {
    fn default() -> Self {
        Self::Admin
    }
}
