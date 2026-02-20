//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Project handler for workflow operations.

use std::sync::Arc;

use mcb_domain::ports::ProjectRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};
use tracing::info;

use crate::args::{ProjectAction, ProjectArgs, ProjectResource};
use crate::utils::mcp::{map_opaque_error, ok_json, resolve_org_id};

/// Handler for the consolidated `project` MCP tool.
pub struct ProjectHandler {
    repo: Arc<dyn ProjectRepository>,
}

impl ProjectHandler {
    /// Create a new project handler backed by a repository implementation.
    pub fn new(repo: Arc<dyn ProjectRepository>) -> Self {
        Self { repo }
    }

    /// Route an incoming `project` tool call to the appropriate operation.
    ///
    /// # Errors
    /// Returns an error when required identifiers are missing or action/resource is unsupported.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
    ) -> Result<CallToolResult, McpError> {
        let project_id = &args.project_id;
        let org_id = resolve_org_id(None);

        if project_id.trim().is_empty() && !matches!(args.action, ProjectAction::List) {
            return Err(McpError::invalid_params("project_id is required", None));
        }

        info!(
            action = ?args.action,
            resource = ?args.resource,
            project_id = %project_id,
            "project request"
        );

        match (args.action, args.resource) {
            (ProjectAction::Get, ProjectResource::Project) => ok_json(&map_opaque_error(
                self.repo.get_by_id(org_id.as_str(), project_id).await,
            )?),
            (ProjectAction::List, ProjectResource::Project) => {
                ok_json(&map_opaque_error(self.repo.list(org_id.as_str()).await)?)
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
