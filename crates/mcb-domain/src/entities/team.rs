//! Team and TeamMember entities — groups of users within an organization.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A team groups users within an organization for access control and
/// project assignment. Teams are used in the GitHub-like RBAC model:
/// Organization → Teams → Projects.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Team {
    /// Unique identifier (UUID).
    pub id: String,
    /// Organization this team belongs to (tenant isolation).
    pub org_id: String,
    /// Human-readable team name (unique within an org).
    pub name: String,
    /// Timestamp when the team was created (Unix epoch).
    pub created_at: i64,
}

/// A membership link between a user and a team, with a role describing
/// the user's authority within that team.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
// TODO(architecture): Add id: Uuid or similar identity field to entity.
// Current implementation relies on composite key (team_id, user_id).
// TODO(CA004): Entity TeamMember missing id/uuid field - Add id: Uuid or similar identity field to entity
pub struct TeamMember {
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

impl TeamMemberRole {
    /// Returns the string representation of the team member role.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
