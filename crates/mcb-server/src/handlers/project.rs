//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Project handler for workflow operations.

use std::sync::Arc;

use mcb_domain::entities::project::Project;
use mcb_domain::info;
use mcb_domain::ports::ProjectRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{ProjectAction, ProjectArgs, ProjectResource};
use crate::error_mapping::safe_internal_error;
use crate::utils::mcp::{map_opaque_error, ok_json, resolve_org_id};

/// Handler for the consolidated `project` MCP tool.
pub struct ProjectHandler {
    repo: Arc<dyn ProjectRepository>,
}

handler_new!(ProjectHandler {
    repo: Arc<dyn ProjectRepository>,
});

impl ProjectHandler {
    /// Route an incoming `project` tool call to the appropriate operation.
    ///
    /// # Errors
    /// Returns an error when required identifiers are missing or action/resource is unsupported.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
    ) -> Result<CallToolResult, McpError> {
        let project_id = args.project_id.as_deref().unwrap_or("");
        let org_id = resolve_org_id(None);

        if project_id.trim().is_empty() && !matches!(args.action, ProjectAction::List) {
            return Err(McpError::invalid_params(
                "project_id is required (not resolved from context)",
                None,
            ));
        }

        info!(
            "ProjectHandler",
            "project request",
            &format!(
                "action={:?} resource={:?} project_id={project_id}",
                args.action, args.resource
            )
        );

        match (args.action, args.resource) {
            (ProjectAction::Get, ProjectResource::Project) => ok_json(&map_opaque_error(
                self.repo.get_by_id(org_id.as_str(), project_id).await,
            )?),
            (ProjectAction::List, ProjectResource::Project) => {
                ok_json(&map_opaque_error(self.repo.list(org_id.as_str()).await)?)
            }
            (ProjectAction::Create, ProjectResource::Project) => {
                self.create_project(&org_id, project_id, &args).await
            }
            (ProjectAction::Update, ProjectResource::Project) => {
                self.update_project(&org_id, project_id, &args).await
            }
            (ProjectAction::Delete, ProjectResource::Project) => {
                self.delete_project(&org_id, project_id).await
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

    /// Creates a new project from the provided MCP arguments.
    async fn create_project(
        &self,
        org_id: &str,
        project_id: &str,
        args: &ProjectArgs,
    ) -> Result<CallToolResult, McpError> {
        let data = args
            .data
            .as_ref()
            .ok_or_else(|| McpError::invalid_params("data is required for create", None))?;
        let name = data
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("name is required in data", None))?;
        let path = data
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("path is required in data", None))?;
        let now = mcb_domain::utils::time::epoch_secs_i64()
            .map_err(|e| safe_internal_error("resolve timestamp", &e))?;
        let project = Project {
            id: project_id.to_owned(),
            org_id: org_id.to_owned(),
            name: name.to_owned(),
            path: path.to_owned(),
            created_at: now,
            updated_at: now,
        };
        map_opaque_error(self.repo.create(&project).await)?;
        ok_json(&project)
    }

    /// Updates an existing project with optional field changes from data payload.
    async fn update_project(
        &self,
        org_id: &str,
        project_id: &str,
        args: &ProjectArgs,
    ) -> Result<CallToolResult, McpError> {
        let mut project = map_opaque_error(self.repo.get_by_id(org_id, project_id).await)?;
        if let Some(data) = &args.data {
            if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
                project.name = name.to_owned();
            }
            if let Some(path) = data.get("path").and_then(|v| v.as_str()) {
                project.path = path.to_owned();
            }
        }
        let now = mcb_domain::utils::time::epoch_secs_i64()
            .map_err(|e| safe_internal_error("resolve timestamp", &e))?;
        project.updated_at = now;
        map_opaque_error(self.repo.update(&project).await)?;
        ok_json(&project)
    }

    /// Deletes a project by ID.
    async fn delete_project(
        &self,
        org_id: &str,
        project_id: &str,
    ) -> Result<CallToolResult, McpError> {
        map_opaque_error(self.repo.delete(org_id, project_id).await)?;
        ok_json(&serde_json::json!({
            "deleted": true,
            "id": project_id,
        }))
    }
}
