//! VCS entity CRUD handler implementation.

use std::sync::Arc;

use mcb_domain::entities::repository::{Branch, Repository};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
use mcb_domain::ports::repositories::VcsEntityRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{VcsEntityAction, VcsEntityArgs, VcsEntityResource};
use crate::handlers::helpers::{
    current_timestamp, map_opaque_error, ok_json, ok_text, require_data, require_id,
    resolve_identifier_precedence, resolve_org_id,
};

/// Handler for the consolidated `vcs_entity` MCP tool.
pub struct VcsEntityHandler {
    repo: Arc<dyn VcsEntityRepository>,
}

impl VcsEntityHandler {
    /// Create a new VCS entity handler backed by a repository implementation.
    pub fn new(repo: Arc<dyn VcsEntityRepository>) -> Self {
        Self { repo }
    }

    /// Route an incoming `vcs_entity` tool call to the appropriate CRUD operation.
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
                tracing::warn!(
                    ?action,
                    ?resource,
                    "unsupported action/resource combination"
                );
                Err(McpError::invalid_params(
                    "unsupported action/resource combination",
                    None,
                ))
            },
            {
            // -- Repository --
            (VcsEntityAction::Create, VcsEntityResource::Repository) => {
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for repository create", None)
                })?;
                let mut repo: Repository = require_data(args.data, "data required for create")?;
                repo.project_id = resolve_identifier_precedence(
                    "project_id",
                    Some(project_id),
                    Some(repo.project_id.as_str()),
                )?
                .ok_or_else(|| {
                    McpError::invalid_params("project_id required for repository create", None)
                })?;
                repo.org_id = org_id.to_string();
                map_opaque_error(self.repo.create_repository(&repo).await).map_err(|e| {
                    McpError::internal_error(format!("failed to create repository: {e}"), None)
                })?;
                ok_json(&repo)
            }
            (VcsEntityAction::Get, VcsEntityResource::Repository) => {
                let id = require_id(&args.id).map_err(|e| {
                    McpError::invalid_params(
                        format!("failed to parse repository id from request: {e}"),
                        None,
                    )
                })?;
                let repository = map_opaque_error(self.repo.get_repository(&org_id, &id).await)
                    .map_err(|e| {
                        McpError::internal_error(
                            format!("failed to get repository '{id}' for org '{org_id}': {e}"),
                            None,
                        )
                    })?;
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
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for list", None)
                })?;
                ok_json(&map_opaque_error(self.repo.list_repositories(&org_id, project_id).await)?)
            }
            (VcsEntityAction::Update, VcsEntityResource::Repository) => {
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for repository update", None)
                })?;
                let mut repo: Repository = require_data(args.data, "data required for update")?;
                repo.project_id = resolve_identifier_precedence(
                    "project_id",
                    Some(project_id),
                    Some(repo.project_id.as_str()),
                )?
                .ok_or_else(|| {
                    McpError::invalid_params("project_id required for repository update", None)
                })?;
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
                repo.org_id = org_id.to_string();
                map_opaque_error(self.repo.update_repository(&repo).await)?;
                ok_text("updated")
            }
            (VcsEntityAction::Delete, VcsEntityResource::Repository) => {
                let id = require_id(&args.id)?;
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for repository delete", None)
                })?;
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
                let branch: Branch = require_data(args.data, "data required").map_err(|e| {
                    McpError::invalid_params(
                        format!("failed to parse branch payload from request: {e}"),
                        None,
                    )
                })?;
                map_opaque_error(self.repo.create_branch(&branch).await).map_err(|e| {
                    McpError::internal_error(format!("failed to create branch: {e}"), None)
                })?;
                ok_json(&branch)
            }
            (VcsEntityAction::Get, VcsEntityResource::Branch) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_branch(&id).await)?)
            }
            (VcsEntityAction::List, VcsEntityResource::Branch) => {
                let repo_id = args
                    .repository_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("repository_id required", None))?;
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
                let repo_id = args
                    .repository_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("repository_id required", None))?;
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
                    require_data(args.data, "data required").map_err(|e| {
                        McpError::invalid_params(
                            format!("failed to parse assignment payload from request: {e}"),
                            None,
                        )
                    })?;
                map_opaque_error(self.repo.create_assignment(&asgn).await).map_err(|e| {
                    McpError::internal_error(format!("failed to create worktree assignment: {e}"), None)
                })?;
                ok_json(&asgn)
            }
            (VcsEntityAction::Get, VcsEntityResource::Assignment) => {
                let id = require_id(&args.id).map_err(|e| {
                    McpError::invalid_params(
                        format!("failed to parse assignment id from request: {e}"),
                        None,
                    )
                })?;
                ok_json(&map_opaque_error(self.repo.get_assignment(&id).await)?)
            }
            (VcsEntityAction::List, VcsEntityResource::Assignment) => {
                let wt_id = args
                    .worktree_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("worktree_id required", None))?;
                ok_json(&map_opaque_error(self.repo.list_assignments_by_worktree(wt_id).await)?)
            }
            (VcsEntityAction::Release, VcsEntityResource::Assignment) => {
                let id = require_id(&args.id).map_err(|e| {
                    McpError::invalid_params(
                        format!("failed to parse assignment id from request: {e}"),
                        None,
                    )
                })?;
                map_opaque_error(self.repo.release_assignment(&id, current_timestamp()).await)
                    .map_err(|e| {
                        McpError::internal_error(
                            format!("failed to release assignment '{id}': {e}"),
                            None,
                        )
                    })?;
                ok_text("released")
            }

            }
        }
    }
}
