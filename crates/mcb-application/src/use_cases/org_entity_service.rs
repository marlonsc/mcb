//! Org entity service implementation.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::{ApiKey, Organization, Team, TeamMember, User};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::repositories::OrgEntityRepository;
use mcb_domain::ports::services::OrgEntityServiceInterface;

/// Application-layer service for organization entity CRUD operations.
pub struct OrgEntityServiceImpl {
    repository: Arc<dyn OrgEntityRepository>,
}

impl OrgEntityServiceImpl {
    /// Create a new [`OrgEntityServiceImpl`] backed by the given repository.
    pub fn new(repository: Arc<dyn OrgEntityRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl OrgEntityServiceInterface for OrgEntityServiceImpl {
    async fn create_org(&self, org: &Organization) -> Result<()> {
        self.repository.create_org(org).await
    }

    async fn get_org(&self, id: &str) -> Result<Organization> {
        self.repository
            .get_org(id)
            .await?
            .ok_or_else(|| Error::not_found(format!("Organization {id}")))
    }

    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        self.repository.list_orgs().await
    }

    async fn update_org(&self, org: &Organization) -> Result<()> {
        self.repository.update_org(org).await
    }

    async fn delete_org(&self, id: &str) -> Result<()> {
        self.repository.delete_org(id).await
    }

    async fn create_user(&self, user: &User) -> Result<()> {
        self.repository.create_user(user).await
    }

    async fn get_user(&self, id: &str) -> Result<User> {
        self.repository
            .get_user(id)
            .await?
            .ok_or_else(|| Error::not_found(format!("User {id}")))
    }

    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User> {
        self.repository
            .get_user_by_email(org_id, email)
            .await?
            .ok_or_else(|| Error::not_found(format!("User {email}")))
    }

    async fn list_users(&self, org_id: &str) -> Result<Vec<User>> {
        self.repository.list_users(org_id).await
    }

    async fn update_user(&self, user: &User) -> Result<()> {
        self.repository.update_user(user).await
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        self.repository.delete_user(id).await
    }

    async fn create_team(&self, team: &Team) -> Result<()> {
        self.repository.create_team(team).await
    }

    async fn get_team(&self, id: &str) -> Result<Team> {
        self.repository
            .get_team(id)
            .await?
            .ok_or_else(|| Error::not_found(format!("Team {id}")))
    }

    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>> {
        self.repository.list_teams(org_id).await
    }

    async fn delete_team(&self, id: &str) -> Result<()> {
        self.repository.delete_team(id).await
    }

    async fn add_team_member(&self, member: &TeamMember) -> Result<()> {
        self.repository.add_team_member(member).await
    }

    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()> {
        self.repository.remove_team_member(team_id, user_id).await
    }

    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>> {
        self.repository.list_team_members(team_id).await
    }

    async fn create_api_key(&self, key: &ApiKey) -> Result<()> {
        self.repository.create_api_key(key).await
    }

    async fn get_api_key(&self, id: &str) -> Result<ApiKey> {
        self.repository
            .get_api_key(id)
            .await?
            .ok_or_else(|| Error::not_found(format!("ApiKey {id}")))
    }

    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>> {
        self.repository.list_api_keys(org_id).await
    }

    async fn revoke_api_key(&self, id: &str, revoked_at: i64) -> Result<()> {
        self.repository.revoke_api_key(id, revoked_at).await
    }

    async fn delete_api_key(&self, id: &str) -> Result<()> {
        self.repository.delete_api_key(id).await
    }
}
