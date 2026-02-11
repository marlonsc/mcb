use async_trait::async_trait;

use crate::entities::{ApiKey, Organization, Team, TeamMember, User};
use crate::error::Result;

#[async_trait]
pub trait OrgEntityRepository: Send + Sync {
    async fn create_org(&self, org: &Organization) -> Result<()>;
    async fn get_org(&self, id: &str) -> Result<Organization>;
    async fn list_orgs(&self) -> Result<Vec<Organization>>;
    async fn update_org(&self, org: &Organization) -> Result<()>;
    async fn delete_org(&self, id: &str) -> Result<()>;

    async fn create_user(&self, user: &User) -> Result<()>;
    async fn get_user(&self, id: &str) -> Result<User>;
    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User>;
    async fn list_users(&self, org_id: &str) -> Result<Vec<User>>;
    async fn update_user(&self, user: &User) -> Result<()>;
    async fn delete_user(&self, id: &str) -> Result<()>;

    async fn create_team(&self, team: &Team) -> Result<()>;
    async fn get_team(&self, id: &str) -> Result<Team>;
    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>>;
    async fn delete_team(&self, id: &str) -> Result<()>;

    async fn add_team_member(&self, member: &TeamMember) -> Result<()>;
    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()>;
    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>>;

    async fn create_api_key(&self, key: &ApiKey) -> Result<()>;
    async fn get_api_key(&self, id: &str) -> Result<ApiKey>;
    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>>;
    async fn revoke_api_key(&self, id: &str, revoked_at: i64) -> Result<()>;
    async fn delete_api_key(&self, id: &str) -> Result<()>;
}
