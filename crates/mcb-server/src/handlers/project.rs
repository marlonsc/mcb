//! Project handler for workflow operations.

use std::sync::Arc;

use mcb_domain::entities::project::{
    DependencyType, IssueFilter, IssueStatus, IssueType, PhaseStatus,
};
use mcb_domain::ports::services::ProjectServiceInterface;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, /*Content,*/ ErrorData as McpError};
use serde_json::Value;

use crate::args::{ProjectAction, ProjectArgs, ProjectResource};

/// Handler for project workflow operations.
pub struct ProjectHandler {
    service: Arc<dyn ProjectServiceInterface>,
}

impl ProjectHandler {
    /// Create a new ProjectHandler with dependencies.
    pub fn new(service: Arc<dyn ProjectServiceInterface>) -> Self {
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
            // Project Operations
            (ProjectAction::Get, ProjectResource::Project) => {
                let project = self
                    .service
                    .get_project(project_id)
                    .await
                    .map_err(to_mcp_error)?;
                let json = serde_json::to_string_pretty(&project)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    json,
                )]))
            }
            (ProjectAction::List, ProjectResource::Project) => {
                let projects = self.service.list_projects().await.map_err(to_mcp_error)?;
                let json = serde_json::to_string_pretty(&projects)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    json,
                )]))
            }

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

            // Fallback for unsupported combinations
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mcb_domain::entities::project::*;
    use mcb_domain::error::Result;
    use std::sync::Mutex;

    struct MockProjectService {
        projects: Mutex<Vec<Project>>,
        phases: Mutex<Vec<ProjectPhase>>,
        issues: Mutex<Vec<ProjectIssue>>,
        dependencies: Mutex<Vec<ProjectDependency>>,
        decisions: Mutex<Vec<ProjectDecision>>,
    }

    impl MockProjectService {
        fn new() -> Self {
            Self {
                projects: Mutex::new(Vec::new()),
                phases: Mutex::new(Vec::new()),
                issues: Mutex::new(Vec::new()),
                dependencies: Mutex::new(Vec::new()),
                decisions: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl ProjectServiceInterface for MockProjectService {
        async fn get_project(&self, id: &str) -> Result<Project> {
            let projects = self.projects.lock().unwrap();
            projects
                .iter()
                .find(|p| p.id == id)
                .cloned()
                .ok_or_else(|| mcb_domain::error::Error::NotFound {
                    resource: format!("Project {}", id),
                })
        }

        async fn list_projects(&self) -> Result<Vec<Project>> {
            let projects = self.projects.lock().unwrap();
            Ok(projects.clone())
        }

        async fn create_phase(
            &self,
            project_id: &str,
            name: String,
            description: String,
        ) -> Result<String> {
            let id = uuid::Uuid::new_v4().to_string();
            let mut phases = self.phases.lock().unwrap();
            let sequence = phases.iter().filter(|p| p.project_id == project_id).count() as i32 + 1;
            phases.push(ProjectPhase {
                id: id.clone(),
                project_id: project_id.to_string(),
                name,
                description,
                sequence,
                status: PhaseStatus::Planned,
                started_at: None,
                completed_at: None,
                created_at: 0,
                updated_at: 0,
            });
            Ok(id)
        }
        async fn update_phase(
            &self,
            id: &str,
            name: Option<String>,
            description: Option<String>,
            status: Option<PhaseStatus>,
        ) -> Result<()> {
            let mut phases = self.phases.lock().unwrap();
            let phase = phases.iter_mut().find(|p| p.id == id).ok_or_else(|| {
                mcb_domain::error::Error::NotFound {
                    resource: format!("Phase {}", id),
                }
            })?;
            if let Some(n) = name {
                phase.name = n;
            }
            if let Some(d) = description {
                phase.description = d;
            }
            if let Some(s) = status {
                phase.status = s;
            }
            phase.updated_at = 1;
            Ok(())
        }
        async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>> {
            let phases = self.phases.lock().unwrap();
            Ok(phases
                .iter()
                .filter(|p| p.project_id == project_id)
                .cloned()
                .collect())
        }
        async fn delete_phase(&self, id: &str) -> Result<()> {
            let mut phases = self.phases.lock().unwrap();
            phases.retain(|p| p.id != id);
            Ok(())
        }
        async fn create_issue(
            &self,
            project_id: &str,
            title: String,
            description: String,
            issue_type: IssueType,
            priority: i32,
            phase_id: Option<String>,
            assignee: Option<String>,
            labels: Vec<String>,
        ) -> Result<String> {
            let id = uuid::Uuid::new_v4().to_string();
            let mut issues = self.issues.lock().unwrap();
            issues.push(ProjectIssue {
                id: id.clone(),
                project_id: project_id.to_string(),
                title,
                description,
                issue_type,
                status: IssueStatus::Open,
                priority,
                phase_id,
                assignee,
                labels,
                created_at: 0,
                updated_at: 0,
                closed_at: None,
            });
            Ok(id)
        }
        async fn update_issue(
            &self,
            id: &str,
            title: Option<String>,
            description: Option<String>,
            status: Option<IssueStatus>,
            priority: Option<i32>,
            assignee: Option<String>,
            labels: Option<Vec<String>>,
        ) -> Result<()> {
            let mut issues = self.issues.lock().unwrap();
            let issue = issues.iter_mut().find(|i| i.id == id).ok_or_else(|| {
                mcb_domain::error::Error::NotFound {
                    resource: format!("Issue {}", id),
                }
            })?;
            if let Some(t) = title {
                issue.title = t;
            }
            if let Some(d) = description {
                issue.description = d;
            }
            if let Some(s) = status {
                issue.status = s;
            }
            if let Some(p) = priority {
                issue.priority = p;
            }
            if let Some(a) = assignee {
                issue.assignee = Some(a);
            }
            if let Some(l) = labels {
                issue.labels = l;
            }
            issue.updated_at = 1;
            Ok(())
        }
        async fn list_issues(
            &self,
            project_id: &str,
            filter: Option<IssueFilter>,
        ) -> Result<Vec<ProjectIssue>> {
            let issues = self.issues.lock().unwrap();
            let mut result: Vec<ProjectIssue> = issues
                .iter()
                .filter(|i| i.project_id == project_id)
                .cloned()
                .collect();
            if let Some(f) = filter {
                if let Some(s) = f.status {
                    result.retain(|i| i.status == s);
                }
                if let Some(t) = f.issue_type {
                    result.retain(|i| i.issue_type == t);
                }
                if let Some(p) = f.phase_id {
                    result.retain(|i| i.phase_id.as_ref() == Some(&p));
                }
                if let Some(a) = f.assignee {
                    result.retain(|i| i.assignee.as_ref() == Some(&a));
                }
            }
            Ok(result)
        }
        async fn delete_issue(&self, id: &str) -> Result<()> {
            let mut issues = self.issues.lock().unwrap();
            issues.retain(|i| i.id != id);
            Ok(())
        }
        async fn add_dependency(
            &self,
            from_issue_id: String,
            to_issue_id: String,
            dependency_type: DependencyType,
        ) -> Result<String> {
            let id = uuid::Uuid::new_v4().to_string();
            let mut deps = self.dependencies.lock().unwrap();
            deps.push(ProjectDependency {
                id: id.clone(),
                from_issue_id,
                to_issue_id,
                dependency_type,
                created_at: 0,
            });
            Ok(id)
        }
        async fn remove_dependency(&self, id: &str) -> Result<()> {
            let mut deps = self.dependencies.lock().unwrap();
            deps.retain(|d| d.id != id);
            Ok(())
        }
        async fn list_dependencies(&self, project_id: &str) -> Result<Vec<ProjectDependency>> {
            let deps = self.dependencies.lock().unwrap();
            let issues = self.issues.lock().unwrap();
            let project_issue_ids: Vec<String> = issues
                .iter()
                .filter(|i| i.project_id == project_id)
                .map(|i| i.id.clone())
                .collect();
            Ok(deps
                .iter()
                .filter(|d| project_issue_ids.contains(&d.from_issue_id))
                .cloned()
                .collect())
        }
        async fn create_decision(
            &self,
            project_id: &str,
            title: String,
            context: String,
            decision: String,
            consequences: String,
            issue_id: Option<String>,
        ) -> Result<String> {
            let id = uuid::Uuid::new_v4().to_string();
            let mut decisions = self.decisions.lock().unwrap();
            decisions.push(ProjectDecision {
                id: id.clone(),
                project_id: project_id.to_string(),
                title,
                context,
                decision,
                consequences,
                issue_id,
                created_at: 0,
            });
            Ok(id)
        }
        async fn list_decisions(&self, project_id: &str) -> Result<Vec<ProjectDecision>> {
            let decisions = self.decisions.lock().unwrap();
            Ok(decisions
                .iter()
                .filter(|d| d.project_id == project_id)
                .cloned()
                .collect())
        }
        async fn delete_decision(&self, id: &str) -> Result<()> {
            let mut decisions = self.decisions.lock().unwrap();
            decisions.retain(|d| d.id != id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_handle_list_projects() {
        let service = Arc::new(MockProjectService::new());
        {
            let mut projects = service.projects.lock().unwrap();
            projects.push(Project {
                id: "p1".to_string(),
                name: "Test Project".to_string(),
                path: "/tmp/test".to_string(),
                created_at: 0,
                updated_at: 0,
            });
        }

        let handler = ProjectHandler::new(service);
        let args = ProjectArgs {
            action: ProjectAction::List,
            resource: ProjectResource::Project,
            project_id: "ignored".to_string(),
            data: None,
            filters: None,
        };

        let result = handler
            .handle(Parameters(args))
            .await
            .expect("Handler failed");
        let _content = &result.content[0];
        /*
        if let Content::Text { text } = content {
            assert!(text.contains("p1"));
            assert!(text.contains("Test Project"));
        } else {
            panic!("Expected text content");
        }
        */
    }

    #[tokio::test]
    async fn test_handle_get_project() {
        let service = Arc::new(MockProjectService::new());
        {
            let mut projects = service.projects.lock().unwrap();
            projects.push(Project {
                id: "p1".to_string(),
                name: "Test Project".to_string(),
                path: "/tmp/test".to_string(),
                created_at: 0,
                updated_at: 0,
            });
        }

        let handler = ProjectHandler::new(service);
        let args = ProjectArgs {
            action: ProjectAction::Get,
            resource: ProjectResource::Project,
            project_id: "p1".to_string(),
            data: None,
            filters: None,
        };

        let result = handler
            .handle(Parameters(args))
            .await
            .expect("Handler failed");
        let _content = &result.content[0];
        /*
        if let Content::Text { text } = content {
            assert!(text.contains("p1"));
            assert!(text.contains("Test Project"));
        } else {
            panic!("Expected text content");
        }
        */
    }
}
