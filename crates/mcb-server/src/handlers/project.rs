//! Project handler for workflow operations.

use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
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
        let _data = args.data.unwrap_or(Value::Null);

        // TODO(phase-1): extract org_id from request context / auth token
        let org_id = DEFAULT_ORG_ID;

        match (args.action, args.resource) {
            (ProjectAction::Get, ProjectResource::Project) => {
                let project = self
                    .service
                    .get_project(org_id, project_id)
                    .await
                    .map_err(to_mcp_error)?;
                let json = serde_json::to_string_pretty(&project)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    json,
                )]))
            }
            (ProjectAction::List, ProjectResource::Project) => {
                let projects = self
                    .service
                    .list_projects(org_id)
                    .await
                    .map_err(to_mcp_error)?;
                let json = serde_json::to_string_pretty(&projects)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    json,
                )]))
            }

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
    }

    impl MockProjectService {
        fn new() -> Self {
            Self {
                projects: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl ProjectServiceInterface for MockProjectService {
        async fn get_project(&self, _org_id: &str, id: &str) -> Result<Project> {
            let projects = self.projects.lock().unwrap();
            projects
                .iter()
                .find(|p| p.id == id)
                .cloned()
                .ok_or_else(|| mcb_domain::error::Error::NotFound {
                    resource: format!("Project {}", id),
                })
        }

        async fn list_projects(&self, _org_id: &str) -> Result<Vec<Project>> {
            let projects = self.projects.lock().unwrap();
            Ok(projects.clone())
        }
    }

    #[tokio::test]
    async fn test_handle_list_projects() {
        let service = Arc::new(MockProjectService::new());
        {
            let mut projects = service.projects.lock().unwrap();
            projects.push(Project {
                id: "p1".to_string(),
                org_id: DEFAULT_ORG_ID.to_string(),
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
    }

    #[tokio::test]
    async fn test_handle_get_project() {
        let service = Arc::new(MockProjectService::new());
        {
            let mut projects = service.projects.lock().unwrap();
            projects.push(Project {
                id: "p1".to_string(),
                org_id: DEFAULT_ORG_ID.to_string(),
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
    }
}
