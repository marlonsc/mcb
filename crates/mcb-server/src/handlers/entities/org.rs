//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Organization entity CRUD handler implementation.

use std::sync::Arc;

use mcb_domain::entities::{ApiKey, Organization, Team, TeamMember, User};
use mcb_domain::ports::OrgEntityRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
use crate::utils::mcp::{
    map_opaque_error, ok_json, ok_text, require_data, require_id, resolve_identifier_precedence,
    resolve_org_id,
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
    ///
    /// # Errors
    /// Returns an error when required identifiers or payload fields are missing.
    #[tracing::instrument(skip_all)]
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
                let payload_org_id = args
                    .data
                    .as_ref()
                    .and_then(|value| value.get("org_id"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                let mut org: Organization = require_data(args.data, "data required for create")?;
                if let Some(resolved) = resolve_identifier_precedence(
                    "org_id",
                    args.org_id.as_deref(),
                    payload_org_id.as_deref(),
                )? {
                    org.id = resolved;
                }
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
                user.org_id = org_id.clone();
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
                user.org_id = org_id.clone();
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
                team.org_id = org_id.clone();
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
                let team_id = require_arg!(args.team_id, "team_id required for list");
                ok_json(&map_opaque_error(self.repo.list_team_members(team_id).await)?)
            }
            (OrgEntityAction::Delete, OrgEntityResource::TeamMember) => {
                let team_id = require_arg!(args.team_id, "team_id required for delete");
                let user_id = require_arg!(args.user_id, "user_id required for delete");
                map_opaque_error(self.repo.remove_team_member(team_id, user_id).await)?;
                ok_text("deleted")
            }
            (OrgEntityAction::Create, OrgEntityResource::ApiKey) => {
                let mut key: ApiKey = require_data(args.data, "data required for create")?;
                key.org_id = org_id.clone();
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
                let id = require_id(&args.id)?;
                let revoked_at = extract_revoked_at(args.data.as_ref());
                map_opaque_error(self.repo.revoke_api_key(&id, revoked_at).await)?;
                ok_text("updated")
            }
            (OrgEntityAction::Delete, OrgEntityResource::ApiKey) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_api_key(&id).await)?;
                ok_text("deleted")
            }
            }
        }
    }
}

fn extract_revoked_at(data: Option<&serde_json::Value>) -> i64 {
    data.and_then(|value| value.get("revoked_at").and_then(serde_json::Value::as_i64))
        .unwrap_or_else(|| mcb_domain::utils::time::epoch_secs_i64().unwrap_or(0))
}
