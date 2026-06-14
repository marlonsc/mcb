//! Organization repository ports.

use async_trait::async_trait;

use crate::entities::{ApiKey, Organization, Team, TeamMember, User};
use crate::error::Result;

define_crud_port! {
    /// Registry for organizations.
    pub trait OrgRegistry {
        entity: Organization,
        create: create_org,
        get: get_org(id),
        list: list_orgs(),
        update: update_org,
        delete: delete_org(id),
    }
}

/// Registry for users.
#[async_trait]
pub trait UserRegistry: Send + Sync {
    /// Create a user.
    async fn create_user(&self, user: &User) -> Result<()>;
    /// Get a user by ID within an organization.
    async fn get_user(&self, org_id: &str, id: &str) -> Result<User>;
    /// Get a user by email within an organization.
    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User>;
    /// List users in an organization.
    async fn list_users(&self, org_id: &str) -> Result<Vec<User>>;
    /// Update a user.
    async fn update_user(&self, user: &User) -> Result<()>;
    /// Delete a user by ID.
    async fn delete_user(&self, id: &str) -> Result<()>;
}

define_crud_port! {
    /// Registry for teams.
    pub trait TeamRegistry {
        entity: Team,
        create: create_team,
        get: get_team(id),
        list: list_teams(org_id),
        delete: delete_team(id),
    }
}

/// Manager for team members.
#[async_trait]
pub trait TeamMemberManager: Send + Sync {
    /// Add a team member.
    async fn add_team_member(&self, member: &TeamMember) -> Result<()>;
    /// Remove a team member.
    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()>;
    /// List team members.
    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>>;
}

/// Registry for API keys.
#[async_trait]
pub trait ApiKeyRegistry: Send + Sync {
    /// Create an API key.
    async fn create_api_key(&self, key: &ApiKey) -> Result<()>;
    /// Get an API key by ID.
    async fn get_api_key(&self, id: &str) -> Result<ApiKey>;
    /// List API keys in an organization.
    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>>;
    /// Revoke an API key.
    async fn revoke_api_key(&self, id: &str, revoked_at: i64) -> Result<()>;
    /// Delete an API key by ID.
    async fn delete_api_key(&self, id: &str) -> Result<()>;
}

define_aggregate! {
    /// Aggregate trait for org entity management.
    #[async_trait]
    pub trait OrgEntityRepository = OrgRegistry + UserRegistry + TeamRegistry + TeamMemberManager + ApiKeyRegistry;
}
