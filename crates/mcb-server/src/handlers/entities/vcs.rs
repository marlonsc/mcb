//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! VCS entity CRUD handler implementation.

use std::sync::Arc;

use mcb_domain::entities::repository::{Branch, Repository};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
use mcb_domain::ports::VcsEntityRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{VcsEntityAction, VcsEntityArgs, VcsEntityResource};
use crate::error_mapping::safe_internal_error;
use crate::utils::mcp::{
    map_opaque_error, ok_json, ok_text, require_data, require_id, require_resolved_identifier,
    resolve_org_id,
};

/// Handler for the consolidated `vcs_entity` MCP tool.
pub struct VcsEntityHandler {
    repo: Arc<dyn VcsEntityRepository>,
}

handler_new!(VcsEntityHandler {
    repo: Arc<dyn VcsEntityRepository>,
});

impl VcsEntityHandler {
    /// Route an incoming `vcs_entity` tool call to the appropriate CRUD operation.
    ///
    /// # Errors
    /// Returns an error when required identifiers or payload fields are missing.
    #[tracing::instrument(
        skip(self),
        fields(action = ?args.action, resource = ?args.resource, org_id = tracing::field::Empty)
    )]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<VcsEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        let org_id = resolve_org_id(args.org_id.as_deref());
        tracing::Span::current().record("org_id", org_id.as_str());

        crate::entity_crud_dispatch! {
            action = args.action,
            resource = args.resource,
            fallback = |action, resource| {
                mcb_domain::warn!(
                    "VcsEntity",
                    "unsupported action/resource combination",
                    &format!("action={action:?} resource={resource:?}")
                );
                Err(McpError::invalid_params(
                    "unsupported action/resource combination",
                    None,
                ))
            },
            {
            // -- Repository --
            (VcsEntityAction::Create, VcsEntityResource::Repository) => {
                let project_id =
                    require_arg!(args.project_id, "project_id required for repository create");
                let mut repo: Repository = require_data(args.data, "data required for create")?;
                repo.project_id = require_resolved_identifier(
                    "project_id",
                    Some(project_id),
                    Some(repo.project_id.as_str()),
                    "project_id required for repository create",
                )?;
                repo.org_id = org_id.clone();
                map_opaque_error(self.repo.create_repository(&repo).await)?;
                ok_json(&repo)
            }
            (VcsEntityAction::Get, VcsEntityResource::Repository) => {
                let id = require_id(&args.id)?;
                let repository = map_opaque_error(self.repo.get_repository(&org_id, &id).await)?;
                if let Some(project_id) = args.project_id.as_deref()
                    && repository.project_id != project_id
                {
                    return Err(McpError::invalid_params(
                        format!(
                            "conflicting project_id: args='{project_id}', repository='{}'",
                            repository.project_id
                        ),
                        None,
                    ));
                }
                ok_json(&repository)
            }
            (VcsEntityAction::List, VcsEntityResource::Repository) => {
                let project_id = require_arg!(args.project_id, "project_id required for list");
                ok_json(&map_opaque_error(self.repo.list_repositories(&org_id, project_id).await)?)
            }
            (VcsEntityAction::Update, VcsEntityResource::Repository) => {
                let project_id =
                    require_arg!(args.project_id, "project_id required for repository update");
                let mut repo: Repository = require_data(args.data, "data required for update")?;
                repo.project_id = require_resolved_identifier(
                    "project_id",
                    Some(project_id),
                    Some(repo.project_id.as_str()),
                    "project_id required for repository update",
                )?;
                let existing = map_opaque_error(self.repo.get_repository(&org_id, &repo.id).await)?;
                if existing.project_id != repo.project_id {
                    return Err(McpError::invalid_params(
                        format!(
                            "conflicting project_id: payload='{}', repository='{}'",
                            repo.project_id, existing.project_id
                        ),
                        None,
                    ));
                }
                repo.org_id = org_id.clone();
                map_opaque_error(self.repo.update_repository(&repo).await)?;
                ok_text("updated")
            }
            (VcsEntityAction::Delete, VcsEntityResource::Repository) => {
                let id = require_id(&args.id)?;
                let project_id =
                    require_arg!(args.project_id, "project_id required for repository delete");
                let existing = map_opaque_error(self.repo.get_repository(&org_id, &id).await)?;
                if existing.project_id != project_id {
                    return Err(McpError::invalid_params(
                        format!(
                            "conflicting project_id: args='{project_id}', repository='{}'",
                            existing.project_id
                        ),
                        None,
                    ));
                }
                map_opaque_error(self.repo.delete_repository(&org_id, &id).await)?;
                ok_text("deleted")
            }

            // -- Branch --
            (VcsEntityAction::Create, VcsEntityResource::Branch) => {
                let branch: Branch = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.create_branch(&branch).await)?;
                ok_json(&branch)
            }
            (VcsEntityAction::Get, VcsEntityResource::Branch) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_branch(&id).await)?)
            }
            (VcsEntityAction::List, VcsEntityResource::Branch) => {
                let repo_id = require_arg!(args.repository_id, "repository_id required");
                ok_json(&map_opaque_error(self.repo.list_branches(repo_id).await)?)
            }
            (VcsEntityAction::Update, VcsEntityResource::Branch) => {
                let branch: Branch = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.update_branch(&branch).await)?;
                ok_text("updated")
            }
            (VcsEntityAction::Delete, VcsEntityResource::Branch) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_branch(&id).await)?;
                ok_text("deleted")
            }

            // -- Worktree --
            (VcsEntityAction::Create, VcsEntityResource::Worktree) => {
                let wt: Worktree = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.create_worktree(&wt).await)?;
                ok_json(&wt)
            }
            (VcsEntityAction::Get, VcsEntityResource::Worktree) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_worktree(&id).await)?)
            }
            (VcsEntityAction::List, VcsEntityResource::Worktree) => {
                let repo_id = require_arg!(args.repository_id, "repository_id required");
                ok_json(&map_opaque_error(self.repo.list_worktrees(repo_id).await)?)
            }
            (VcsEntityAction::Update, VcsEntityResource::Worktree) => {
                let wt: Worktree = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.update_worktree(&wt).await)?;
                ok_text("updated")
            }
            (VcsEntityAction::Delete, VcsEntityResource::Worktree) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_worktree(&id).await)?;
                ok_text("deleted")
            }

            // -- Assignment --
            (VcsEntityAction::Create, VcsEntityResource::Assignment) => {
                let asgn: AgentWorktreeAssignment =
                    require_data(args.data, "data required")?;
                map_opaque_error(self.repo.create_assignment(&asgn).await)?;
                ok_json(&asgn)
            }
            (VcsEntityAction::Get, VcsEntityResource::Assignment) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_assignment(&id).await)?)
            }
            (VcsEntityAction::List, VcsEntityResource::Assignment) => {
                let wt_id = require_arg!(args.worktree_id, "worktree_id required");
                ok_json(&map_opaque_error(self.repo.list_assignments_by_worktree(wt_id).await)?)
            }
            (VcsEntityAction::Release, VcsEntityResource::Assignment) => {
                let id = require_id(&args.id)?;
                let now = mcb_domain::utils::time::epoch_secs_i64()
                    .map_err(|e| safe_internal_error("resolve timestamp", &e))?;
                map_opaque_error(self.repo.release_assignment(&id, now).await)?;
                ok_text("released")
            }

            }
        }
    }
}
