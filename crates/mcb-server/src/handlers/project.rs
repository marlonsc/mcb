//! Project handler for workflow operations.

use std::sync::Arc;

use mcb_domain::entities::project::{
    DependencyType, IssueFilter, IssueStatus, IssueType, PhaseStatus,
};
use mcb_domain::ports::services::ProjectService;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};
use serde_json::Value;

use crate::args::{ProjectAction, ProjectArgs, ProjectResource};

/// Handler for project workflow operations.
pub struct ProjectHandler {
    service: Arc<dyn ProjectService>,
}

impl ProjectHandler {
    /// Create a new ProjectHandler with dependencies.
    pub fn new(service: Arc<dyn ProjectService>) -> Self {
        Self { service }
    }

    /// Handle a project tool request.
    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
    ) -> Result<CallToolResult, McpError> {
        let project_id = &args.project_id;
        let data = args.data.unwrap_or(Value::Null);

        match (args.action, args.resource) {
            // Phase Operations
            (ProjectAction::Create, ProjectResource::Phase) => {
                let name = get_string(&data, "name")?;
                let description = get_string(&data, "description")?;
                let id = self
                    .service
                    .create_phase(project_id, name, description)
                    .await
                    .map_err(to_mcp_error)?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("Created phase: {}", id),
                )]))
            }
            (ProjectAction::Update, ProjectResource::Phase) => {
                let id = get_string(&data, "id")?;
                let name = get_opt_string(&data, "name");
                let description = get_opt_string(&data, "description");
                let status = if let Some(s) = get_opt_string(&data, "status") {
                    Some(
                        s.parse::<PhaseStatus>()
                            .map_err(|e| McpError::invalid_params(e, None))?,
                    )
                } else {
                    None
                };

                self.service
                    .update_phase(&id, name, description, status)
                    .await
                    .map_err(to_mcp_error)?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("Updated phase: {}", id),
                )]))
            }
            (ProjectAction::List, ProjectResource::Phase) => {
                let phases = self
                    .service
                    .list_phases(project_id)
                    .await
                    .map_err(to_mcp_error)?;
                let json = serde_json::to_string_pretty(&phases)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    json,
                )]))
            }
            (ProjectAction::Delete, ProjectResource::Phase) => {
                let id = get_string(&data, "id")?;
                self.service.delete_phase(&id).await.map_err(to_mcp_error)?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("Deleted phase: {}", id),
                )]))
            }

            // Issue Operations
            (ProjectAction::Create, ProjectResource::Issue) => {
                let title = get_string(&data, "title")?;
                let description = get_string(&data, "description")?;
                let issue_type = get_string(&data, "issue_type")?
                    .parse::<IssueType>()
                    .map_err(|e| McpError::invalid_params(e, None))?;
                let priority = get_i32(&data, "priority")?;
                let phase_id = get_opt_string(&data, "phase_id");
                let assignee = get_opt_string(&data, "assignee");
                let labels = get_vec_string(&data, "labels").unwrap_or_default();

                let id = self
                    .service
                    .create_issue(
                        project_id,
                        title,
                        description,
                        issue_type,
                        priority,
                        phase_id,
                        assignee,
                        labels,
                    )
                    .await
                    .map_err(to_mcp_error)?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("Created issue: {}", id),
                )]))
            }
            (ProjectAction::Update, ProjectResource::Issue) => {
                let id = get_string(&data, "id")?;
                let title = get_opt_string(&data, "title");
                let description = get_opt_string(&data, "description");
                let status = if let Some(s) = get_opt_string(&data, "status") {
                    Some(
                        s.parse::<IssueStatus>()
                            .map_err(|e| McpError::invalid_params(e, None))?,
                    )
                } else {
                    None
                };
                let priority = get_opt_i32(&data, "priority");
                let assignee = get_opt_string(&data, "assignee");
                let labels = get_vec_string(&data, "labels");

                self.service
                    .update_issue(&id, title, description, status, priority, assignee, labels)
                    .await
                    .map_err(to_mcp_error)?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("Updated issue: {}", id),
                )]))
            }
            (ProjectAction::List, ProjectResource::Issue) => {
                let filter: Option<IssueFilter> = if let Some(f) = args.filters {
                    Some(serde_json::from_value(f).map_err(|e| {
                        McpError::invalid_params(format!("Invalid filter: {}", e), None)
                    })?)
                } else {
                    None
                };

                let issues = self
                    .service
                    .list_issues(project_id, filter)
                    .await
                    .map_err(to_mcp_error)?;
                let json = serde_json::to_string_pretty(&issues)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    json,
                )]))
            }
            (ProjectAction::Delete, ProjectResource::Issue) => {
                let id = get_string(&data, "id")?;
                self.service.delete_issue(&id).await.map_err(to_mcp_error)?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("Deleted issue: {}", id),
                )]))
            }

            // Dependency Operations
            (ProjectAction::Create, ProjectResource::Dependency) => {
                let from = get_string(&data, "from_issue_id")?;
                let to = get_string(&data, "to_issue_id")?;
                let type_str = get_string(&data, "dependency_type")?;
                let dep_type = type_str
                    .parse::<DependencyType>()
                    .map_err(|e| McpError::invalid_params(e, None))?;

                let id = self
                    .service
                    .add_dependency(from, to, dep_type)
                    .await
                    .map_err(to_mcp_error)?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("Added dependency: {}", id),
                )]))
            }
            (ProjectAction::Delete, ProjectResource::Dependency) => {
                let id = get_string(&data, "id")?;
                self.service
                    .remove_dependency(&id)
                    .await
                    .map_err(to_mcp_error)?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("Removed dependency: {}", id),
                )]))
            }
            (ProjectAction::List, ProjectResource::Dependency) => {
                let deps = self
                    .service
                    .list_dependencies(project_id)
                    .await
                    .map_err(to_mcp_error)?;
                let json = serde_json::to_string_pretty(&deps)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    json,
                )]))
            }
            (ProjectAction::Update, ProjectResource::Dependency) => Err(McpError::invalid_params(
                "Update not supported for dependency".to_string(),
                None,
            )),

            // Decision Operations
            (ProjectAction::Create, ProjectResource::Decision) => {
                let title = get_string(&data, "title")?;
                let context = get_string(&data, "context")?;
                let decision = get_string(&data, "decision")?;
                let consequences = get_string(&data, "consequences")?;
                let issue_id = get_opt_string(&data, "issue_id");

                let id = self
                    .service
                    .create_decision(project_id, title, context, decision, consequences, issue_id)
                    .await
                    .map_err(to_mcp_error)?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("Recorded decision: {}", id),
                )]))
            }
            (ProjectAction::List, ProjectResource::Decision) => {
                let decisions = self
                    .service
                    .list_decisions(project_id)
                    .await
                    .map_err(to_mcp_error)?;
                let json = serde_json::to_string_pretty(&decisions)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    json,
                )]))
            }
            (ProjectAction::Delete, ProjectResource::Decision) => {
                let id = get_string(&data, "id")?;
                self.service
                    .delete_decision(&id)
                    .await
                    .map_err(to_mcp_error)?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("Deleted decision: {}", id),
                )]))
            }
            (ProjectAction::Update, ProjectResource::Decision) => Err(McpError::invalid_params(
                "Update not supported for decision (immutable)".to_string(),
                None,
            )),
        }
    }
}

// Helper functions for parameter extraction

fn get_string(data: &Value, key: &str) -> Result<String, McpError> {
    data.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| McpError::invalid_params(format!("Missing or invalid field: {}", key), None))
}

fn get_opt_string(data: &Value, key: &str) -> Option<String> {
    data.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn get_i32(data: &Value, key: &str) -> Result<i32, McpError> {
    data.get(key)
        .and_then(|v| v.as_i64())
        .map(|i| i as i32)
        .ok_or_else(|| McpError::invalid_params(format!("Missing or invalid field: {}", key), None))
}

fn get_opt_i32(data: &Value, key: &str) -> Option<i32> {
    data.get(key).and_then(|v| v.as_i64()).map(|i| i as i32)
}

fn get_vec_string(data: &Value, key: &str) -> Option<Vec<String>> {
    data.get(key).and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|val| val.as_str().map(|s| s.to_string()))
                .collect()
        })
    })
}

fn to_mcp_error(e: mcb_domain::error::Error) -> McpError {
    match e {
        mcb_domain::error::Error::NotFound { .. } => McpError::invalid_params(e.to_string(), None),
        mcb_domain::error::Error::InvalidArgument { .. } => {
            McpError::invalid_params(e.to_string(), None)
        }
        _ => McpError::internal_error(e.to_string(), None),
    }
}
