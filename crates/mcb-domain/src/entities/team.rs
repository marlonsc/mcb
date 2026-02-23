//! Team and TeamMember entities — groups of users within an organization.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

crate::define_entity! {
    /// A team groups users within an organization for access control and
    /// project assignment. Teams are used in the GitHub-like RBAC model:
    /// Organization → Teams → Projects.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Team { id, org_id, created_at } {
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

crate::impl_table_schema!(Team, "teams",
    columns: [
        ("id", Text, pk),
        ("org_id", Text),
        ("name", Text),
        ("created_at", Integer),
    ],
    indexes: [
        "idx_teams_org" => ["org_id"],
    ],
    foreign_keys: [
        ("org_id", "organizations", "id"),
    ],
    unique_constraints: [
        ["org_id", "name"],
    ],
);

crate::impl_table_schema!(TeamMember, "team_members",
    columns: [
        ("team_id", Text, pk),
        ("user_id", Text, pk),
        ("role", Text),
        ("joined_at", Integer),
    ],
    indexes: [
        "idx_team_members_team" => ["team_id"],
        "idx_team_members_user" => ["user_id"],
    ],
    foreign_keys: [
        ("team_id", "teams", "id"),
        ("user_id", "users", "id"),
    ],
    unique_constraints: [
        ["team_id", "user_id"],
    ],
);
