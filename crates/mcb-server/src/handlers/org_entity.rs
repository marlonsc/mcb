use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::entities::{ApiKey, Organization, Team, TeamMember, User};
use mcb_domain::ports::repositories::OrgEntityRepository;
use mcb_domain::value_objects::OrgContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
use crate::error_mapping::to_opaque_mcp_error;
use crate::handler_helpers::{ok_json, ok_text, require_id};

/// Handler for the consolidated `org_entity` MCP tool.
pub struct OrgEntityHandler {
    repo: Arc<dyn OrgEntityRepository>,
}

impl OrgEntityHandler {
    /// Create a new handler backed by the given repository.
    pub fn new(repo: Arc<dyn OrgEntityRepository>) -> Self {
        Self { repo }
    }

    /// Route an incoming `org_entity` tool call to the appropriate CRUD operation.
    pub async fn handle(
        &self,
        Parameters(args): Parameters<OrgEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        let org_ctx = OrgContext::default();
        let org_id = args.org_id.as_deref().unwrap_or(org_ctx.org_id.as_str());

        match (args.action, args.resource) {
            (OrgEntityAction::Create, OrgEntityResource::Org) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for create", None))?;
                let org: Organization = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                self.repo
                    .create_org(&org)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&org)
            }
            (OrgEntityAction::Get, OrgEntityResource::Org) => {
                let id = require_id(&args.id)?;
                let org = self.repo.get_org(&id).await.map_err(to_opaque_mcp_error)?;
                ok_json(&org)
            }
            (OrgEntityAction::List, OrgEntityResource::Org) => {
                let orgs = self.repo.list_orgs().await.map_err(to_opaque_mcp_error)?;
                ok_json(&orgs)
            }
            (OrgEntityAction::Update, OrgEntityResource::Org) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for update", None))?;
                let org: Organization = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                self.repo
                    .update_org(&org)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("updated")
            }
            (OrgEntityAction::Delete, OrgEntityResource::Org) => {
                let id = require_id(&args.id)?;
                self.repo
                    .delete_org(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("deleted")
            }
            (OrgEntityAction::Create, OrgEntityResource::User) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for create", None))?;
                let mut user: User = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                user.org_id = org_id.to_string();
                self.repo
                    .create_user(&user)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&user)
            }
            (OrgEntityAction::Get, OrgEntityResource::User) => {
                let user = if let Some(id) = args.id.as_deref() {
                    self.repo.get_user(id).await
                } else if let Some(email) = args.email.as_deref() {
                    self.repo.get_user_by_email(org_id, email).await
                } else {
                    return Err(McpError::invalid_params(
                        "id or email required for user get",
                        None,
                    ));
                }
                .map_err(to_opaque_mcp_error)?;
                ok_json(&user)
            }
            (OrgEntityAction::List, OrgEntityResource::User) => {
                let users = self
                    .repo
                    .list_users(org_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&users)
            }
            (OrgEntityAction::Update, OrgEntityResource::User) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for update", None))?;
                let mut user: User = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                user.org_id = org_id.to_string();
                self.repo
                    .update_user(&user)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("updated")
            }
            (OrgEntityAction::Delete, OrgEntityResource::User) => {
                let id = require_id(&args.id)?;
                self.repo
                    .delete_user(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("deleted")
            }
            (OrgEntityAction::Create, OrgEntityResource::Team) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for create", None))?;
                let mut team: Team = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                team.org_id = org_id.to_string();
                self.repo
                    .create_team(&team)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&team)
            }
            (OrgEntityAction::Get, OrgEntityResource::Team) => {
                let id = require_id(&args.id)?;
                let team = self.repo.get_team(&id).await.map_err(to_opaque_mcp_error)?;
                ok_json(&team)
            }
            (OrgEntityAction::List, OrgEntityResource::Team) => {
                let teams = self
                    .repo
                    .list_teams(org_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&teams)
            }
            (OrgEntityAction::Delete, OrgEntityResource::Team) => {
                let id = require_id(&args.id)?;
                self.repo
                    .delete_team(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("deleted")
            }
            (OrgEntityAction::Create, OrgEntityResource::TeamMember) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for create", None))?;
                let member: TeamMember = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                self.repo
                    .add_team_member(&member)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&member)
            }
            (OrgEntityAction::List, OrgEntityResource::TeamMember) => {
                let team_id = args
                    .team_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("team_id required for list", None))?;
                let members = self
                    .repo
                    .list_team_members(team_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&members)
            }
            (OrgEntityAction::Delete, OrgEntityResource::TeamMember) => {
                let team_id = args
                    .team_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("team_id required for delete", None))?;
                let user_id = args
                    .user_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("user_id required for delete", None))?;
                self.repo
                    .remove_team_member(team_id, user_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("deleted")
            }
            (OrgEntityAction::Create, OrgEntityResource::ApiKey) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for create", None))?;
                let mut key: ApiKey = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                key.org_id = org_id.to_string();
                self.repo
                    .create_api_key(&key)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&key)
            }
            (OrgEntityAction::Get, OrgEntityResource::ApiKey) => {
                let id = require_id(&args.id)?;
                let key = self
                    .repo
                    .get_api_key(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&key)
            }
            (OrgEntityAction::List, OrgEntityResource::ApiKey) => {
                let keys = self
                    .repo
                    .list_api_keys(org_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&keys)
            }
            (OrgEntityAction::Update, OrgEntityResource::ApiKey) => {
                let id = require_id(&args.id)?;
                let revoked_at = extract_revoked_at(args.data.as_ref());
                self.repo
                    .revoke_api_key(&id, revoked_at)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("updated")
            }
            (OrgEntityAction::Delete, OrgEntityResource::ApiKey) => {
                let id = require_id(&args.id)?;
                self.repo
                    .delete_api_key(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("deleted")
            }
            _ => Err(McpError::invalid_params(
                format!(
                    "Unsupported action {:?} for resource {:?}",
                    args.action, args.resource
                ),
                None,
            )),
        }
    }
}

fn extract_revoked_at(data: Option<&serde_json::Value>) -> i64 {
    data.and_then(|value| value.get("revoked_at").and_then(serde_json::Value::as_i64))
        .unwrap_or_else(current_timestamp)
}

fn current_timestamp() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mcb_domain::error::Result;
    use std::sync::Mutex;

    struct MockOrgEntityService {
        orgs: Mutex<Vec<Organization>>,
    }

    impl MockOrgEntityService {
        fn new() -> Self {
            Self {
                orgs: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl OrgEntityRepository for MockOrgEntityService {
        async fn create_org(&self, org: &Organization) -> Result<()> {
            self.orgs.lock().expect("lock orgs").push(org.clone());
            Ok(())
        }
        async fn get_org(&self, id: &str) -> Result<Organization> {
            self.orgs
                .lock()
                .expect("lock orgs")
                .iter()
                .find(|o| o.id == id)
                .cloned()
                .ok_or_else(|| mcb_domain::error::Error::not_found(format!("Organization {id}")))
        }
        async fn list_orgs(&self) -> Result<Vec<Organization>> {
            Ok(self.orgs.lock().expect("lock orgs").clone())
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
            Err(mcb_domain::error::Error::not_found("user"))
        }
        async fn get_user_by_email(&self, _org_id: &str, _email: &str) -> Result<User> {
            Err(mcb_domain::error::Error::not_found("user"))
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
            Err(mcb_domain::error::Error::not_found("team"))
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
            Err(mcb_domain::error::Error::not_found("api key"))
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

    #[tokio::test]
    async fn test_list_orgs() {
        let service = Arc::new(MockOrgEntityService::new());
        {
            service.orgs.lock().expect("lock orgs").push(Organization {
                id: "o1".into(),
                name: "Acme".into(),
                slug: "acme".into(),
                settings_json: "{}".into(),
                created_at: 0,
                updated_at: 0,
            });
        }
        let handler = OrgEntityHandler::new(service);
        let args = OrgEntityArgs {
            action: OrgEntityAction::List,
            resource: OrgEntityResource::Org,
            id: None,
            org_id: None,
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        };
        let result = handler.handle(Parameters(args)).await.expect("handle ok");
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_get_org() {
        let service = Arc::new(MockOrgEntityService::new());
        {
            service.orgs.lock().expect("lock orgs").push(Organization {
                id: "o1".into(),
                name: "Acme".into(),
                slug: "acme".into(),
                settings_json: "{}".into(),
                created_at: 0,
                updated_at: 0,
            });
        }
        let handler = OrgEntityHandler::new(service);
        let args = OrgEntityArgs {
            action: OrgEntityAction::Get,
            resource: OrgEntityResource::Org,
            id: Some("o1".into()),
            org_id: None,
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        };
        let result = handler.handle(Parameters(args)).await.expect("handle ok");
        assert!(!result.content.is_empty());
    }
}
