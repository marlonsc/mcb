//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

crate::define_entity_org_audited! {
    /// Represents a user within the system.
    ///
    /// Users are associated with an organization and have specific roles that
    /// determine their permissions.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct User {
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

/// Role a user holds within an organization.
#[derive(
    Debug,
    Clone,
    Default,
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
pub enum UserRole {
    /// Full administrative access.
    #[default]
    Admin,
    /// Standard member with read/write access.
    Member,
    /// Read-only viewer.
    Viewer,
    /// Service account (API-only, used by agents).
    Service,
}

crate::impl_as_str_from_as_ref!(UserRole);
