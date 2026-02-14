//! Organization entity CRUD handler implementation.

use std::sync::Arc;

use mcb_domain::entities::{ApiKey, Organization, Team, TeamMember, User};
use mcb_domain::ports::repositories::OrgEntityRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
use crate::handlers::helpers::{
    current_timestamp, map_opaque_error, ok_json, ok_text, require_data, require_id, resolve_org_id,
};

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
    /// # Architecture Violation (KISS005)
    /// Function length (131 lines) exceeds the 50-line limit.
    ///
    // TODO(KISS005): Break 'handle' into smaller, focused functions.
    pub async fn handle(
        &self,
        Parameters(args): Parameters<OrgEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        let org_id = resolve_org_id(args.org_id.as_deref());

        crate::entity_crud_dispatch! {
            action = args.action,
            resource = args.resource,
            {
            (OrgEntityAction::Create, OrgEntityResource::Org) => {
                let org: Organization = require_data(args.data, "data required for create")?;
                map_opaque_error(self.repo.create_org(&org).await)?;
                ok_json(&org)
            }
            (OrgEntityAction::Get, OrgEntityResource::Org) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_org(&id).await)?)
            }
            (OrgEntityAction::List, OrgEntityResource::Org) => {
                ok_json(&map_opaque_error(self.repo.list_orgs().await)?)
            }
            (OrgEntityAction::Update, OrgEntityResource::Org) => {
                let org: Organization = require_data(args.data, "data required for update")?;
                map_opaque_error(self.repo.update_org(&org).await)?;
                ok_text("updated")
            }
            (OrgEntityAction::Delete, OrgEntityResource::Org) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_org(&id).await)?;
                ok_text("deleted")
            }
            (OrgEntityAction::Create, OrgEntityResource::User) => {
                let mut user: User = require_data(args.data, "data required for create")?;
                user.org_id = org_id.to_string();
                map_opaque_error(self.repo.create_user(&user).await)?;
                ok_json(&user)
            }
            (OrgEntityAction::Get, OrgEntityResource::User) => {
                let user = if let Some(id) = args.id.as_deref() {
                    self.repo.get_user(id).await
                } else if let Some(email) = args.email.as_deref() {
                    self.repo.get_user_by_email(org_id.as_str(), email).await
                } else {
                    return Err(McpError::invalid_params(
                        "id or email required for user get",
                        None,
                    ));
                };
                ok_json(&map_opaque_error(user)?)
            }
            (OrgEntityAction::List, OrgEntityResource::User) => {
                ok_json(&map_opaque_error(self.repo.list_users(org_id.as_str()).await)?)
            }
            (OrgEntityAction::Update, OrgEntityResource::User) => {
                let mut user: User = require_data(args.data, "data required for update")?;
                user.org_id = org_id.to_string();
                map_opaque_error(self.repo.update_user(&user).await)?;
                ok_text("updated")
            }
            (OrgEntityAction::Delete, OrgEntityResource::User) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_user(&id).await)?;
                ok_text("deleted")
            }
            (OrgEntityAction::Create, OrgEntityResource::Team) => {
                let mut team: Team = require_data(args.data, "data required for create")?;
                team.org_id = org_id.to_string();
                map_opaque_error(self.repo.create_team(&team).await)?;
                ok_json(&team)
            }
            (OrgEntityAction::Get, OrgEntityResource::Team) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_team(&id).await)?)
            }
            (OrgEntityAction::List, OrgEntityResource::Team) => {
                ok_json(&map_opaque_error(self.repo.list_teams(org_id.as_str()).await)?)
            }
            (OrgEntityAction::Delete, OrgEntityResource::Team) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_team(&id).await)?;
                ok_text("deleted")
            }
            (OrgEntityAction::Create, OrgEntityResource::TeamMember) => {
                let member: TeamMember = require_data(args.data, "data required for create")?;
                map_opaque_error(self.repo.add_team_member(&member).await)?;
                ok_json(&member)
            }
            (OrgEntityAction::List, OrgEntityResource::TeamMember) => {
                let team_id = args
                    .team_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("team_id required for list", None))?;
                ok_json(&map_opaque_error(self.repo.list_team_members(team_id).await)?)
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
                map_opaque_error(self.repo.remove_team_member(team_id, user_id).await)?;
                ok_text("deleted")
            }
            (OrgEntityAction::Create, OrgEntityResource::ApiKey) => {
                let mut key: ApiKey = require_data(args.data, "data required for create")?;
                key.org_id = org_id.to_string();
                map_opaque_error(self.repo.create_api_key(&key).await)?;
                ok_json(&key)
            }
            (OrgEntityAction::Get, OrgEntityResource::ApiKey) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_api_key(&id).await)?)
            }
            (OrgEntityAction::List, OrgEntityResource::ApiKey) => {
                ok_json(&map_opaque_error(self.repo.list_api_keys(org_id.as_str()).await)?)
            }
            (OrgEntityAction::Update, OrgEntityResource::ApiKey) => {
                let id = require_id(&args.id).map_err(|e| {
                    McpError::invalid_params(
                        format!("failed to parse api key id from request: {e}"),
                        None,
                    )
                })?;
                let revoked_at = extract_revoked_at(args.data.as_ref());
                map_opaque_error(self.repo.revoke_api_key(&id, revoked_at).await).map_err(|e| {
                    McpError::internal_error(format!("failed to revoke api key '{id}': {e}"), None)
                })?;
                ok_text("updated")
            }
            (OrgEntityAction::Delete, OrgEntityResource::ApiKey) => {
                let id = require_id(&args.id).map_err(|e| {
                    McpError::invalid_params(
                        format!("failed to parse api key id from request: {e}"),
                        None,
                    )
                })?;
                map_opaque_error(self.repo.delete_api_key(&id).await).map_err(|e| {
                    McpError::internal_error(format!("failed to delete api key '{id}': {e}"), None)
                })?;
                ok_text("deleted")
            }
            }
        }
    }
}

fn extract_revoked_at(data: Option<&serde_json::Value>) -> i64 {
    data.and_then(|value| value.get("revoked_at").and_then(serde_json::Value::as_i64))
        .unwrap_or_else(current_timestamp)
}
