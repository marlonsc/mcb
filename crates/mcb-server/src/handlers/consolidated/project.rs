//! Project handler for project workflow operations.

use crate::args::{ProjectAction, ProjectArgs, ProjectResource};
use crate::formatter::ResponseFormatter;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use validator::Validate;

/// In-memory project storage for workflow operations.
/// In a production system, this would be backed by a persistent database.
#[derive(Clone)]
struct ProjectStore {
    projects: Arc<RwLock<HashMap<String, ProjectData>>>,
}

#[derive(Clone, Debug)]
struct ProjectData {
    id: String,
    name: String,
    description: Option<String>,
    phases: Vec<PhaseData>,
    issues: Vec<IssueData>,
    dependencies: Vec<DependencyData>,
    decisions: Vec<DecisionData>,
}

#[derive(Clone, Debug)]
struct PhaseData {
    id: String,
    name: String,
    status: String,
}

#[derive(Clone, Debug)]
struct IssueData {
    id: String,
    title: String,
    phase_id: Option<String>,
    status: String,
    priority: i32,
}

#[derive(Clone, Debug)]
struct DependencyData {
    from_id: String,
    to_id: String,
    dependency_type: String,
}

#[derive(Clone, Debug)]
struct DecisionData {
    id: String,
    title: String,
    rationale: String,
}

impl ProjectStore {
    fn new() -> Self {
        Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn create_project(&self, project_id: String, data: Option<Value>) -> Result<Value, String> {
        let mut projects = self.projects.write().map_err(|e| e.to_string())?;

        if projects.contains_key(&project_id) {
            return Err(format!("Project '{}' already exists", project_id));
        }

        let name = data
            .as_ref()
            .and_then(|d| d.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or(&project_id)
            .to_string();

        let description = data
            .as_ref()
            .and_then(|d| d.get("description"))
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        let project = ProjectData {
            id: project_id.clone(),
            name,
            description,
            phases: Vec::new(),
            issues: Vec::new(),
            dependencies: Vec::new(),
            decisions: Vec::new(),
        };

        projects.insert(project_id.clone(), project);

        Ok(json!({
            "status": "success",
            "message": format!("Project '{}' created successfully", project_id),
            "project_id": project_id
        }))
    }

    fn update_project(
        &self,
        project_id: String,
        resource: ProjectResource,
        resource_id: Option<String>,
        data: Option<Value>,
    ) -> Result<Value, String> {
        let mut projects = self.projects.write().map_err(|e| e.to_string())?;

        let project = projects
            .get_mut(&project_id)
            .ok_or_else(|| format!("Project '{}' not found", project_id))?;

        match resource {
            ProjectResource::Phase => {
                let phase_id = resource_id.ok_or("Phase ID required for update")?;
                let name = data
                    .as_ref()
                    .and_then(|d| d.get("name"))
                    .and_then(|n| n.as_str())
                    .ok_or("Phase name required")?
                    .to_string();

                if let Some(phase) = project.phases.iter_mut().find(|p| p.id == phase_id) {
                    phase.name = name;
                    Ok(json!({
                        "status": "success",
                        "message": format!("Phase '{}' updated", phase_id)
                    }))
                } else {
                    Err(format!("Phase '{}' not found", phase_id))
                }
            }
            ProjectResource::Issue => {
                let issue_id = resource_id.ok_or("Issue ID required for update")?;
                let status = data
                    .as_ref()
                    .and_then(|d| d.get("status"))
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());

                if let Some(issue) = project.issues.iter_mut().find(|i| i.id == issue_id) {
                    if let Some(new_status) = status {
                        issue.status = new_status;
                    }
                    Ok(json!({
                        "status": "success",
                        "message": format!("Issue '{}' updated", issue_id)
                    }))
                } else {
                    Err(format!("Issue '{}' not found", issue_id))
                }
            }
            ProjectResource::Decision => {
                let decision_id = resource_id.ok_or("Decision ID required for update")?;
                let title = data
                    .as_ref()
                    .and_then(|d| d.get("title"))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string());

                if let Some(decision) = project.decisions.iter_mut().find(|d| d.id == decision_id) {
                    if let Some(new_title) = title {
                        decision.title = new_title;
                    }
                    Ok(json!({
                        "status": "success",
                        "message": format!("Decision '{}' updated", decision_id)
                    }))
                } else {
                    Err(format!("Decision '{}' not found", decision_id))
                }
            }
            ProjectResource::Dependency => {
                Err("Use add_dependency action for dependency updates".to_string())
            }
        }
    }

    fn list_resources(
        &self,
        project_id: String,
        resource: ProjectResource,
        phase_id: Option<String>,
        status: Option<String>,
        priority: Option<i32>,
        limit: Option<u32>,
    ) -> Result<Value, String> {
        let projects = self.projects.read().map_err(|e| e.to_string())?;

        let project = projects
            .get(&project_id)
            .ok_or_else(|| format!("Project '{}' not found", project_id))?;

        let limit = limit.unwrap_or(100) as usize;

        match resource {
            ProjectResource::Phase => {
                let phases: Vec<_> = project
                    .phases
                    .iter()
                    .take(limit)
                    .map(|p| {
                        json!({
                            "id": p.id,
                            "name": p.name,
                            "status": p.status
                        })
                    })
                    .collect();

                Ok(json!({
                    "resource": "phases",
                    "count": phases.len(),
                    "items": phases
                }))
            }
            ProjectResource::Issue => {
                let issues: Vec<_> = project
                    .issues
                    .iter()
                    .filter(|i| {
                        if let Some(ref phase) = phase_id {
                            i.phase_id.as_ref() == Some(phase)
                        } else {
                            true
                        }
                    })
                    .filter(|i| {
                        if let Some(ref s) = status {
                            &i.status == s
                        } else {
                            true
                        }
                    })
                    .filter(|i| {
                        if let Some(p) = priority {
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
                            "status": i.status,
                            "priority": i.priority
                        })
                    })
                    .collect();

                Ok(json!({
                    "resource": "issues",
                    "count": issues.len(),
                    "items": issues
                }))
            }
            ProjectResource::Decision => {
                let decisions: Vec<_> = project
                    .decisions
                    .iter()
                    .take(limit)
                    .map(|d| {
                        json!({
                            "id": d.id,
                            "title": d.title,
                            "rationale": d.rationale
                        })
                    })
                    .collect();

                Ok(json!({
                    "resource": "decisions",
                    "count": decisions.len(),
                    "items": decisions
                }))
            }
            ProjectResource::Dependency => {
                let dependencies: Vec<_> = project
                    .dependencies
                    .iter()
                    .take(limit)
                    .map(|d| {
                        json!({
                            "from_id": d.from_id,
                            "to_id": d.to_id,
                            "type": d.dependency_type
                        })
                    })
                    .collect();

                Ok(json!({
                    "resource": "dependencies",
                    "count": dependencies.len(),
                    "items": dependencies
                }))
            }
        }
    }

    fn add_dependency(
        &self,
        project_id: String,
        from_id: String,
        to_id: String,
        data: Option<Value>,
    ) -> Result<Value, String> {
        let mut projects = self.projects.write().map_err(|e| e.to_string())?;

        let project = projects
            .get_mut(&project_id)
            .ok_or_else(|| format!("Project '{}' not found", project_id))?;

        let dependency_type = data
            .as_ref()
            .and_then(|d| d.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("depends_on")
            .to_string();

        let dependency = DependencyData {
            from_id: from_id.clone(),
            to_id: to_id.clone(),
            dependency_type,
        };

        project.dependencies.push(dependency);

        Ok(json!({
            "status": "success",
            "message": format!("Dependency added: {} -> {}", from_id, to_id)
        }))
    }
}

/// Handler for project workflow MCP tool operations.
#[derive(Clone)]
pub struct ProjectHandler {
    store: ProjectStore,
}

impl ProjectHandler {
    pub fn new() -> Self {
        Self {
            store: ProjectStore::new(),
        }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        let result = match args.action {
            ProjectAction::Create => self
                .store
                .create_project(args.project_id, args.data)
                .map_err(|e| e.to_string()),
            ProjectAction::Update => self
                .store
                .update_project(args.project_id, args.resource, args.resource_id, args.data)
                .map_err(|e| e.to_string()),
            ProjectAction::List => self
                .store
                .list_resources(
                    args.project_id,
                    args.resource,
                    args.phase_id,
                    args.status,
                    args.priority,
                    args.limit,
                )
                .map_err(|e| e.to_string()),
            ProjectAction::AddDependency => {
                let from_id = match args.resource_id {
                    Some(id) => id,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "from_id (resource_id) required for add_dependency".to_string(),
                        )]));
                    }
                };
                let to_id = match args
                    .data
                    .as_ref()
                    .and_then(|d| d.get("to_id"))
                    .and_then(|t| t.as_str())
                {
                    Some(id) => id.to_string(),
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "to_id required in data for add_dependency".to_string(),
                        )]));
                    }
                };

                self.store
                    .add_dependency(args.project_id, from_id, to_id, args.data)
                    .map_err(|e| e.to_string())
            }
        };

        match result {
            Ok(response) => ResponseFormatter::json_success(&response),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Project operation failed: {}",
                e
            ))])),
        }
    }
}

impl Default for ProjectHandler {
    fn default() -> Self {
        Self::new()
    }
}
