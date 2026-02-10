//! Project handler for workflow operations.

use std::sync::Arc;

use mcb_domain::ports::services::ProjectServiceInterface;
use mcb_domain::value_objects::OrgContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};
use serde_json::Value;
use tracing::info;

use crate::args::{ProjectAction, ProjectArgs, ProjectResource};
use crate::error_mapping::to_opaque_mcp_error;
use crate::handler_helpers::ok_json;

/// Handler for the consolidated `project` MCP tool.
pub struct ProjectHandler {
    service: Arc<dyn ProjectServiceInterface>,
}

impl ProjectHandler {
    /// Create a new handler wrapping the given service.
    pub fn new(service: Arc<dyn ProjectServiceInterface>) -> Self {
        Self { service }
    }

    /// Route an incoming `project` tool call to the appropriate operation.
    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
    ) -> Result<CallToolResult, McpError> {
        let project_id = &args.project_id;
        let _data = args.data.unwrap_or(Value::Null);

        // TODO(phase-1): extract org_id from auth token / request context
        let org_ctx = OrgContext::default();
        let org_id = org_ctx.org_id.as_str();

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
            (ProjectAction::Get, ProjectResource::Project) => {
                let project = self
                    .service
                    .get_project(org_id, project_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&project)
            }
            (ProjectAction::List, ProjectResource::Project) => {
                let projects = self
                    .service
                    .list_projects(org_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&projects)
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mcb_domain::constants::keys::DEFAULT_ORG_ID;
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

    #[tokio::test]
    async fn test_empty_project_id_rejected_for_get() {
        let service = Arc::new(MockProjectService::new());
        let handler = ProjectHandler::new(service);
        let args = ProjectArgs {
            action: ProjectAction::Get,
            resource: ProjectResource::Project,
            project_id: "  ".to_string(),
            data: None,
            filters: None,
        };

        let err = handler
            .handle(Parameters(args))
            .await
            .expect_err("Should reject empty project_id");
        assert!(err.message.contains("project_id is required"));
    }
}
