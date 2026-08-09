//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Project handler for workflow operations.

use std::sync::Arc;

use mcb_domain::entities::project::{
    IssueFilter, Project, ProjectDecision, ProjectDependency, ProjectIssue, ProjectPhase,
};
use mcb_domain::info;
use mcb_domain::ports::ProjectRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{ProjectAction, ProjectArgs, ProjectResource};
use crate::error_mapping::safe_internal_error;
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::{map_opaque_error, ok_text, require_data, require_id, resolve_org_id};

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
        let project_id = args.project_id.clone().unwrap_or_default();
        let org_id = resolve_org_id(None);

        Self::ensure_project_id_present(&project_id, args.action, args.resource)?;

        info!(
            "ProjectHandler",
            "project request",
            &format!(
                "action={:?} resource={:?} project_id={project_id}",
                args.action, args.resource
            )
        );

        match args.resource {
            ProjectResource::Project => self.handle_project(&org_id, &project_id, args).await,
            ProjectResource::Phase => self.handle_phase(&project_id, args).await,
            ProjectResource::Issue => self.handle_issue(&org_id, &project_id, args).await,
            ProjectResource::Dependency => self.handle_dependency(args).await,
            ProjectResource::Decision => self.handle_decision(&project_id, args).await,
        }
    }

    /// Reject requests that omit `project_id` for operations that require it.
    fn ensure_project_id_present(
        project_id: &str,
        action: ProjectAction,
        resource: ProjectResource,
    ) -> Result<(), McpError> {
        if !project_id.trim().is_empty() {
            return Ok(());
        }
        let exempt = matches!(
            (action, resource),
            (ProjectAction::List, ProjectResource::Project)
        ) || matches!(
            (action, resource),
            (
                ProjectAction::Get | ProjectAction::Delete,
                ProjectResource::Phase | ProjectResource::Decision | ProjectResource::Dependency
            )
        );
        if exempt {
            Ok(())
        } else {
            Err(McpError::invalid_params(
                "project_id is required (not resolved from context)",
                None,
            ))
        }
    }

    async fn handle_project(
        &self,
        org_id: &str,
        project_id: &str,
        args: ProjectArgs,
    ) -> Result<CallToolResult, McpError> {
        match args.action {
            ProjectAction::Get => ResponseFormatter::json_success(&map_opaque_error(
                self.repo.get_by_id(org_id, project_id).await,
            )?),
            ProjectAction::List => {
                ResponseFormatter::json_success(&map_opaque_error(self.repo.list(org_id).await)?)
            }
            ProjectAction::Create => self.create_project(org_id, project_id, &args).await,
            ProjectAction::Update => self.update_project(org_id, project_id, &args).await,
            ProjectAction::Delete => self.delete_project(org_id, project_id).await,
        }
    }

    async fn handle_phase(
        &self,
        project_id: &str,
        args: ProjectArgs,
    ) -> Result<CallToolResult, McpError> {
        match args.action {
            ProjectAction::Create => {
                let mut phase: ProjectPhase = require_data(args.data, "data required")?;
                if phase.project_id.is_empty() {
                    phase.project_id = project_id.to_owned();
                }
                map_opaque_error(self.repo.create_phase(&phase).await)?;
                ResponseFormatter::json_success(&phase)
            }
            ProjectAction::Get => {
                let id = require_id(&args.id)?;
                ResponseFormatter::json_success(&map_opaque_error(self.repo.get_phase(&id).await)?)
            }
            ProjectAction::List => ResponseFormatter::json_success(&map_opaque_error(
                self.repo.list_phases(project_id).await,
            )?),
            ProjectAction::Update => {
                let phase: ProjectPhase = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.update_phase(&phase).await)?;
                ok_text("updated")
            }
            ProjectAction::Delete => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_phase(&id).await)?;
                ok_text("deleted")
            }
        }
    }

    async fn handle_issue(
        &self,
        org_id: &str,
        project_id: &str,
        args: ProjectArgs,
    ) -> Result<CallToolResult, McpError> {
        match args.action {
            ProjectAction::Create => {
                let mut issue: ProjectIssue = require_data(args.data, "data required")?;
                if issue.project_id.is_empty() {
                    issue.project_id = project_id.to_owned();
                }
                issue.org_id = org_id.to_owned();
                map_opaque_error(self.repo.create_issue(&issue).await)?;
                ResponseFormatter::json_success(&issue)
            }
            ProjectAction::Get => {
                let id = require_id(&args.id)?;
                ResponseFormatter::json_success(&map_opaque_error(
                    self.repo.get_issue(org_id, &id).await,
                )?)
            }
            ProjectAction::List => {
                if let Some(filters) = &args.filters {
                    let filter: IssueFilter = serde_json::from_value(filters.clone())
                        .map_err(|e| McpError::invalid_params(format!("bad filters: {e}"), None))?;
                    ResponseFormatter::json_success(&map_opaque_error(
                        self.repo.list_issues_filtered(org_id, &filter).await,
                    )?)
                } else {
                    ResponseFormatter::json_success(&map_opaque_error(
                        self.repo.list_issues(org_id, project_id).await,
                    )?)
                }
            }
            ProjectAction::Update => {
                let mut issue: ProjectIssue = require_data(args.data, "data required")?;
                issue.org_id = org_id.to_owned();
                map_opaque_error(self.repo.update_issue(&issue).await)?;
                ok_text("updated")
            }
            ProjectAction::Delete => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_issue(org_id, &id).await)?;
                ok_text("deleted")
            }
        }
    }

    async fn handle_dependency(&self, args: ProjectArgs) -> Result<CallToolResult, McpError> {
        match args.action {
            ProjectAction::Create => {
                let dep: ProjectDependency = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.create_dependency(&dep).await)?;
                ResponseFormatter::json_success(&dep)
            }
            ProjectAction::List => {
                let issue_id = args.issue_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("issue_id required for dependency list", None)
                })?;
                ResponseFormatter::json_success(&map_opaque_error(
                    self.repo.list_dependencies(issue_id).await,
                )?)
            }
            ProjectAction::Delete => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_dependency(&id).await)?;
                ok_text("deleted")
            }
            action @ (ProjectAction::Get | ProjectAction::Update) => Err(McpError::invalid_params(
                format!("Unsupported action {action:?} for resource Dependency"),
                None,
            )),
        }
    }

    async fn handle_decision(
        &self,
        project_id: &str,
        args: ProjectArgs,
    ) -> Result<CallToolResult, McpError> {
        match args.action {
            ProjectAction::Create => {
                let mut decision: ProjectDecision = require_data(args.data, "data required")?;
                if decision.project_id.is_empty() {
                    decision.project_id = project_id.to_owned();
                }
                map_opaque_error(self.repo.create_decision(&decision).await)?;
                ResponseFormatter::json_success(&decision)
            }
            ProjectAction::Get => {
                let id = require_id(&args.id)?;
                ResponseFormatter::json_success(&map_opaque_error(
                    self.repo.get_decision(&id).await,
                )?)
            }
            ProjectAction::List => ResponseFormatter::json_success(&map_opaque_error(
                self.repo.list_decisions(project_id).await,
            )?),
            ProjectAction::Update => {
                let decision: ProjectDecision = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.update_decision(&decision).await)?;
                ok_text("updated")
            }
            ProjectAction::Delete => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_decision(&id).await)?;
                ok_text("deleted")
            }
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
        let now = mcb_utils::utils::time::epoch_secs_i64()
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
        ResponseFormatter::json_success(&project)
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
        let now = mcb_utils::utils::time::epoch_secs_i64()
            .map_err(|e| safe_internal_error("resolve timestamp", &e))?;
        project.updated_at = now;
        map_opaque_error(self.repo.update(&project).await)?;
        ResponseFormatter::json_success(&project)
    }

    /// Deletes a project by ID.
    async fn delete_project(
        &self,
        org_id: &str,
        project_id: &str,
    ) -> Result<CallToolResult, McpError> {
        map_opaque_error(self.repo.delete(org_id, project_id).await)?;
        ResponseFormatter::json_success(&serde_json::json!({
            "deleted": true,
            "id": project_id,
        }))
    }
}
