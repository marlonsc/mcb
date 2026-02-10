use async_trait::async_trait;
use mcb_domain::entities::{ApiKey, Organization, Team, TeamMember, User};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::services::OrgEntityServiceInterface;

#[allow(dead_code)]
pub struct MockOrgEntityService;

#[allow(dead_code)]
impl MockOrgEntityService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockOrgEntityService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OrgEntityServiceInterface for MockOrgEntityService {
    async fn create_org(&self, _org: &Organization) -> Result<()> {
        Ok(())
    }

    async fn get_org(&self, _id: &str) -> Result<Organization> {
        Err(Error::not_found("not found"))
    }

    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        Ok(vec![])
    }

    async fn update_org(&self, _org: &Organization) -> Result<()> {
        Ok(())
    }

    async fn delete_org(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn create_user(&self, _user: &User) -> Result<()> {
        Ok(())
    }

    async fn get_user(&self, _id: &str) -> Result<User> {
        Err(Error::not_found("not found"))
    }

    async fn get_user_by_email(&self, _org_id: &str, _email: &str) -> Result<User> {
        Err(Error::not_found("not found"))
    }

    async fn list_users(&self, _org_id: &str) -> Result<Vec<User>> {
        Ok(vec![])
    }

    async fn update_user(&self, _user: &User) -> Result<()> {
        Ok(())
    }

    async fn delete_user(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn create_team(&self, _team: &Team) -> Result<()> {
        Ok(())
    }

    async fn get_team(&self, _id: &str) -> Result<Team> {
        Err(Error::not_found("not found"))
    }

    async fn list_teams(&self, _org_id: &str) -> Result<Vec<Team>> {
        Ok(vec![])
    }

    async fn delete_team(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn add_team_member(&self, _member: &TeamMember) -> Result<()> {
        Ok(())
    }

    async fn remove_team_member(&self, _team_id: &str, _user_id: &str) -> Result<()> {
        Ok(())
    }

    async fn list_team_members(&self, _team_id: &str) -> Result<Vec<TeamMember>> {
        Ok(vec![])
    }

    async fn create_api_key(&self, _key: &ApiKey) -> Result<()> {
        Ok(())
    }

    async fn get_api_key(&self, _id: &str) -> Result<ApiKey> {
        Err(Error::not_found("not found"))
    }

    async fn list_api_keys(&self, _org_id: &str) -> Result<Vec<ApiKey>> {
        Ok(vec![])
    }

    async fn revoke_api_key(&self, _id: &str, _revoked_at: i64) -> Result<()> {
        Ok(())
    }

    async fn delete_api_key(&self, _id: &str) -> Result<()> {
        Ok(())
    }
}
