//! Project handler for project workflow operations.

use crate::args::{ProjectAction, ProjectArgs, ProjectResource};
use crate::formatter::ResponseFormatter;
use mcb_domain::ports::repositories::ProjectRepository;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

/// Handler for project workflow MCP tool operations.
#[derive(Clone)]
pub struct ProjectHandler {
    repository: Arc<dyn ProjectRepository>,
}

impl ProjectHandler {
    pub fn new(repository: Arc<dyn ProjectRepository>) -> Self {
        Self { repository }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        let result = match args.action {
            ProjectAction::Create => self.handle_create(&args).await,
            ProjectAction::Update => self.handle_update(&args).await,
            ProjectAction::List => self.handle_list(&args).await,
            ProjectAction::AddDependency => self.handle_add_dependency(&args).await,
        };

        match result {
            Ok(response) => ResponseFormatter::json_success(&response),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Project operation failed: {}",
                e
            ))])),
        }
    }

    async fn handle_create(&self, args: &ProjectArgs) -> Result<serde_json::Value, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;

        let project = mcb_domain::entities::project::Project {
            id: args.project_id.clone(),
            name: args.project_id.clone(),
            path: String::new(),
            created_at: now,
            updated_at: now,
        };

        self.repository
            .create(&project)
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "status": "success",
            "message": format!("Project '{}' created successfully", args.project_id),
            "project_id": args.project_id
        }))
    }

    async fn handle_update(&self, args: &ProjectArgs) -> Result<serde_json::Value, String> {
        match args.resource {
            ProjectResource::Phase => {
                let phase_id = args
                    .resource_id
                    .as_ref()
                    .ok_or("Phase ID required for update")?;
                let phase = self
                    .repository
                    .get_phase(phase_id)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Phase '{}' not found", phase_id))?;

                let name = args
                    .data
                    .as_ref()
                    .and_then(|d| d.get("name"))
                    .and_then(|n| n.as_str())
                    .ok_or("Phase name required")?;

                let mut updated_phase = phase;
                updated_phase.name = name.to_string();
                updated_phase.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_secs() as i64;

                self.repository
                    .update_phase(&updated_phase)
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(json!({
                    "status": "success",
                    "message": format!("Phase '{}' updated", phase_id)
                }))
            }
            ProjectResource::Issue => {
                let issue_id = args
                    .resource_id
                    .as_ref()
                    .ok_or("Issue ID required for update")?;
                let issue = self
                    .repository
                    .get_issue(issue_id)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Issue '{}' not found", issue_id))?;

                let mut updated_issue = issue;

                if let Some(status_str) = args
                    .data
                    .as_ref()
                    .and_then(|d| d.get("status"))
                    .and_then(|s| s.as_str())
                {
                    updated_issue.status = status_str
                        .parse()
                        .map_err(|_| format!("Invalid status: {}", status_str))?;
                }

                updated_issue.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_secs() as i64;

                self.repository
                    .update_issue(&updated_issue)
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(json!({
                    "status": "success",
                    "message": format!("Issue '{}' updated", issue_id)
                }))
            }
            ProjectResource::Decision => {
                let decision_id = args
                    .resource_id
                    .as_ref()
                    .ok_or("Decision ID required for update")?;
                let decision = self
                    .repository
                    .get_decision(decision_id)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Decision '{}' not found", decision_id))?;

                let mut updated_decision = decision;

                if let Some(title) = args
                    .data
                    .as_ref()
                    .and_then(|d| d.get("title"))
                    .and_then(|t| t.as_str())
                {
                    updated_decision.title = title.to_string();
                }

                self.repository
                    .create_decision(&updated_decision)
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(json!({
                    "status": "success",
                    "message": format!("Decision '{}' updated", decision_id)
                }))
            }
            ProjectResource::Dependency => {
                Err("Use add_dependency action for dependency updates".to_string())
            }
        }
    }

    async fn handle_list(&self, args: &ProjectArgs) -> Result<serde_json::Value, String> {
        let limit = args.limit.unwrap_or(100) as usize;

        match args.resource {
            ProjectResource::Phase => {
                let phases = self
                    .repository
                    .list_phases(&args.project_id)
                    .await
                    .map_err(|e| e.to_string())?;

                let items: Vec<_> = phases
                    .iter()
                    .take(limit)
                    .map(|p| {
                        json!({
                            "id": p.id,
                            "name": p.name,
                            "status": p.status.as_str()
                        })
                    })
                    .collect();

                Ok(json!({
                    "resource": "phases",
                    "count": items.len(),
                    "items": items
                }))
            }
            ProjectResource::Issue => {
                let issues = self
                    .repository
                    .list_issues(&args.project_id)
                    .await
                    .map_err(|e| e.to_string())?;

                let items: Vec<_> = issues
                    .iter()
                    .filter(|i| {
                        if let Some(ref phase) = args.phase_id {
                            i.phase_id.as_ref() == Some(phase)
                        } else {
                            true
                        }
                    })
                    .filter(|i| {
                        if let Some(ref s) = args.status {
                            i.status.as_str() == s
                        } else {
                            true
                        }
                    })
                    .filter(|i| {
                        if let Some(p) = args.priority {
                            i.priority == p
                        } else {
                            true
                        }
                    })
                    .take(limit)
                    .map(|i| {
                        json!({
                            "id": i.id,
                            "title": i.title,
                            "phase_id": i.phase_id,
                            "status": i.status.as_str(),
                            "priority": i.priority
                        })
                    })
                    .collect();

                Ok(json!({
                    "resource": "issues",
                    "count": items.len(),
                    "items": items
                }))
            }
            ProjectResource::Decision => {
                let decisions = self
                    .repository
                    .list_decisions(&args.project_id)
                    .await
                    .map_err(|e| e.to_string())?;

                let items: Vec<_> = decisions
                    .iter()
                    .take(limit)
                    .map(|d| {
                        json!({
                            "id": d.id,
                            "title": d.title,
                            "context": d.context
                        })
                    })
                    .collect();

                Ok(json!({
                    "resource": "decisions",
                    "count": items.len(),
                    "items": items
                }))
            }
            ProjectResource::Dependency => {
                let dependencies = self
                    .repository
                    .list_dependencies(&args.project_id)
                    .await
                    .map_err(|e| e.to_string())?;

                let items: Vec<_> = dependencies
                    .iter()
                    .take(limit)
                    .map(|d| {
                        json!({
                            "from_id": d.from_issue_id,
                            "to_id": d.to_issue_id,
                            "type": d.dependency_type.as_str()
                        })
                    })
                    .collect();

                Ok(json!({
                    "resource": "dependencies",
                    "count": items.len(),
                    "items": items
                }))
            }
        }
    }

    async fn handle_add_dependency(&self, args: &ProjectArgs) -> Result<serde_json::Value, String> {
        let from_id = args
            .resource_id
            .as_ref()
            .ok_or("from_id (resource_id) required for add_dependency")?;

        let to_id = args
            .data
            .as_ref()
            .and_then(|d| d.get("to_id"))
            .and_then(|t| t.as_str())
            .ok_or("to_id required in data for add_dependency")?;

        let dependency_type_str = args
            .data
            .as_ref()
            .and_then(|d| d.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("blocks");

        let dependency_type = dependency_type_str
            .parse()
            .map_err(|_| format!("Invalid dependency type: {}", dependency_type_str))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;

        let dependency = mcb_domain::entities::project::ProjectDependency {
            id: uuid::Uuid::new_v4().to_string(),
            from_issue_id: from_id.clone(),
            to_issue_id: to_id.to_string(),
            dependency_type,
            created_at: now,
        };

        self.repository
            .add_dependency(&dependency)
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "status": "success",
            "message": format!("Dependency added: {} -> {}", from_id, to_id)
        }))
    }
}
