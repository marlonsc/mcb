//! Team and TeamMember entities — groups of users within an organization.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

crate::define_entity_org_created! {
    /// A team groups users within an organization for access control and
    /// project assignment. Teams are used in the GitHub-like RBAC model:
    /// Organization → Teams → Projects.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Team {
        /// Display name of the team.
        pub name: String,
    }
}

use crate::value_objects::ids::TeamMemberId;

/// A membership link between a user and a team, with a role describing
/// the user's authority within that team.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TeamMember {
    /// Unique identifier for the membership (composite of `team_id:user_id`).
    #[serde(default)]
    pub id: TeamMemberId,
    /// Team the user belongs to.
    pub team_id: String,
    /// User who is a member.
    pub user_id: String,
    /// Role the user holds within the team.
    pub role: TeamMemberRole,
    /// Timestamp when the user joined the team (Unix epoch).
    pub joined_at: i64,
}

/// Role a user holds within a specific team.
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
pub enum TeamMemberRole {
    /// Team lead with management capabilities.
    Lead,
    /// Regular team member.
    Member,
}

crate::impl_as_str_from_as_ref!(TeamMemberRole);
