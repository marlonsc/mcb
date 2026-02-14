//! Organization Entity Repository Port
//!
//! # Overview
//! Defines the interface for persisting organization-related entities including
//! organizations, users, teams, and API keys.
use async_trait::async_trait;

use crate::entities::{ApiKey, Organization, Team, TeamMember, User};
use crate::error::Result;

#[async_trait]
/// Defines behavior for OrgEntityRepository.
// TODO(architecture): Consider splitting into smaller interfaces (ISP).
// Current interface combines Org, User, Team, and ApiKey management.
// TODO(PORT003): Port OrgEntityRepository has 23 methods (>10) - Consider splitting into smaller interfaces (ISP)
#[async_trait]
/// Registry for organizations.
pub trait OrgRegistry: Send + Sync {
    /// Performs the create org operation.
    async fn create_org(&self, org: &Organization) -> Result<()>;
    /// Performs the get org operation.
    async fn get_org(&self, id: &str) -> Result<Organization>;
    /// Performs the list orgs operation.
    async fn list_orgs(&self) -> Result<Vec<Organization>>;
    /// Performs the update org operation.
    async fn update_org(&self, org: &Organization) -> Result<()>;
    /// Performs the delete org operation.
    async fn delete_org(&self, id: &str) -> Result<()>;
}

#[async_trait]
/// Registry for users.
pub trait UserRegistry: Send + Sync {
    /// Performs the create user operation.
    async fn create_user(&self, user: &User) -> Result<()>;
    /// Performs the get user operation.
    async fn get_user(&self, id: &str) -> Result<User>;
    /// Performs the get user by email operation.
    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User>;
    /// Performs the list users operation.
    async fn list_users(&self, org_id: &str) -> Result<Vec<User>>;
    /// Performs the update user operation.
    async fn update_user(&self, user: &User) -> Result<()>;
    /// Performs the delete user operation.
    async fn delete_user(&self, id: &str) -> Result<()>;
}

#[async_trait]
/// Registry for teams.
pub trait TeamRegistry: Send + Sync {
    /// Performs the create team operation.
    async fn create_team(&self, team: &Team) -> Result<()>;
    /// Performs the get team operation.
    async fn get_team(&self, id: &str) -> Result<Team>;
    /// Performs the list teams operation.
    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>>;
    /// Performs the delete team operation.
    async fn delete_team(&self, id: &str) -> Result<()>;
}

#[async_trait]
/// Manager for team members.
pub trait TeamMemberManager: Send + Sync {
    /// Performs the add team member operation.
    async fn add_team_member(&self, member: &TeamMember) -> Result<()>;
    /// Performs the remove team member operation.
    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()>;
    /// Performs the list team members operation.
    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>>;
}

#[async_trait]
/// Registry for API keys.
pub trait ApiKeyRegistry: Send + Sync {
    /// Performs the create api key operation.
    async fn create_api_key(&self, key: &ApiKey) -> Result<()>;
    /// Performs the get api key operation.
    async fn get_api_key(&self, id: &str) -> Result<ApiKey>;
    /// Performs the list api keys operation.
    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>>;
    /// Performs the revoke api key operation.
    async fn revoke_api_key(&self, id: &str, revoked_at: i64) -> Result<()>;
    /// Performs the delete api key operation.
    async fn delete_api_key(&self, id: &str) -> Result<()>;
}

/// Aggregate trait for org entity management.
pub trait OrgEntityRepository:
    OrgRegistry + UserRegistry + TeamRegistry + TeamMemberManager + ApiKeyRegistry + Send + Sync
{
}

impl<T> OrgEntityRepository for T where
    T: OrgRegistry + UserRegistry + TeamRegistry + TeamMemberManager + ApiKeyRegistry + Send + Sync
{
}
