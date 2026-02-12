//! Project handler for workflow operations.

use std::sync::Arc;

use mcb_application::services::RepositoryResolver;
use mcb_domain::ports::repositories::ProjectRepository;
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
    repo: Arc<dyn ProjectRepository>,
    resolver: Arc<RepositoryResolver>,
}

impl ProjectHandler {
    /// Create a new project handler backed by a repository implementation.
    pub fn new(repo: Arc<dyn ProjectRepository>, resolver: Arc<RepositoryResolver>) -> Self {
        Self { repo, resolver }
    }

    /// Route an incoming `project` tool call to the appropriate operation.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
    ) -> Result<CallToolResult, McpError> {
        let _data = args.data.unwrap_or(Value::Null);

        let org_ctx = OrgContext::default();
        let org_id = org_ctx.org_id.as_str();
        let project_id = self.resolver.resolve_project_id(org_id).await;

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
                    .repo
                    .get_by_id(org_id, &project_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&project)
            }
            (ProjectAction::List, ProjectResource::Project) => {
                let projects = self.repo.list(org_id).await.map_err(to_opaque_mcp_error)?;
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
    use mcb_domain::ports::repositories::VcsEntityRepository;
    use mcb_domain::value_objects::project_context::ProjectContext;
    use std::sync::Mutex;

    fn test_resolver() -> Arc<RepositoryResolver> {
        let mock_vcs: Arc<dyn VcsEntityRepository> = Arc::new(MockVcsEntityService);
        Arc::new(RepositoryResolver::with_context(
            mock_vcs,
            ProjectContext::new("test/project", "project"),
        ))
    }

    struct MockVcsEntityService;

    #[async_trait]
    impl VcsEntityRepository for MockVcsEntityService {
        async fn create_repository(
            &self,
            _repo: &mcb_domain::entities::repository::Repository,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_repository(
            &self,
            _org_id: &str,
            _id: &str,
        ) -> Result<mcb_domain::entities::repository::Repository> {
            Err(mcb_domain::error::Error::not_found("repo"))
        }
        async fn find_repository_by_url(
            &self,
            _org_id: &str,
            _url: &str,
        ) -> Result<Option<mcb_domain::entities::repository::Repository>> {
            Ok(None)
        }
        async fn list_repositories(
            &self,
            _org_id: &str,
            _project_id: &str,
        ) -> Result<Vec<mcb_domain::entities::repository::Repository>> {
            Ok(vec![])
        }
        async fn update_repository(
            &self,
            _repo: &mcb_domain::entities::repository::Repository,
        ) -> Result<()> {
            Ok(())
        }
        async fn delete_repository(&self, _org_id: &str, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn ensure_org_and_project(&self, _project_id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_branch(
            &self,
            _branch: &mcb_domain::entities::repository::Branch,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_branch(&self, _id: &str) -> Result<mcb_domain::entities::repository::Branch> {
            Err(mcb_domain::error::Error::not_found("branch"))
        }
        async fn list_branches(
            &self,
            _repo_id: &str,
        ) -> Result<Vec<mcb_domain::entities::repository::Branch>> {
            Ok(vec![])
        }
        async fn update_branch(
            &self,
            _branch: &mcb_domain::entities::repository::Branch,
        ) -> Result<()> {
            Ok(())
        }
        async fn delete_branch(&self, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_worktree(
            &self,
            _wt: &mcb_domain::entities::worktree::Worktree,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_worktree(
            &self,
            _id: &str,
        ) -> Result<mcb_domain::entities::worktree::Worktree> {
            Err(mcb_domain::error::Error::not_found("worktree"))
        }
        async fn list_worktrees(
            &self,
            _repo_id: &str,
        ) -> Result<Vec<mcb_domain::entities::worktree::Worktree>> {
            Ok(vec![])
        }
        async fn update_worktree(
            &self,
            _wt: &mcb_domain::entities::worktree::Worktree,
        ) -> Result<()> {
            Ok(())
        }
        async fn delete_worktree(&self, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_assignment(
            &self,
            _asgn: &mcb_domain::entities::worktree::AgentWorktreeAssignment,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_assignment(
            &self,
            _id: &str,
        ) -> Result<mcb_domain::entities::worktree::AgentWorktreeAssignment> {
            Err(mcb_domain::error::Error::not_found("assignment"))
        }
        async fn list_assignments_by_worktree(
            &self,
            _worktree_id: &str,
        ) -> Result<Vec<mcb_domain::entities::worktree::AgentWorktreeAssignment>> {
            Ok(vec![])
        }
        async fn release_assignment(&self, _id: &str, _released_at: i64) -> Result<()> {
            Ok(())
        }
    }

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
    impl ProjectRepository for MockProjectService {
        async fn create(&self, _project: &Project) -> Result<()> {
            Ok(())
        }
        async fn get_by_id(&self, _org_id: &str, id: &str) -> Result<Project> {
            let projects = self.projects.lock().unwrap();
            projects
                .iter()
                .find(|p| p.id == id)
                .cloned()
                .ok_or_else(|| mcb_domain::error::Error::NotFound {
                    resource: format!("Project {}", id),
                })
        }
        async fn get_by_name(&self, _org_id: &str, _name: &str) -> Result<Project> {
            Err(mcb_domain::error::Error::not_found("not found"))
        }
        async fn get_by_path(&self, _org_id: &str, _path: &str) -> Result<Project> {
            Err(mcb_domain::error::Error::not_found("not found"))
        }
        async fn list(&self, _org_id: &str) -> Result<Vec<Project>> {
            let projects = self.projects.lock().unwrap();
            Ok(projects.clone())
        }
        async fn update(&self, _project: &Project) -> Result<()> {
            Ok(())
        }
        async fn delete(&self, _org_id: &str, _id: &str) -> Result<()> {
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
                org_id: DEFAULT_ORG_ID.to_string(),
                name: "Test Project".to_string(),
                path: "/tmp/test".to_string(),
                created_at: 0,
                updated_at: 0,
            });
        }

        let handler = ProjectHandler::new(service, test_resolver());
        let args = ProjectArgs {
            action: ProjectAction::List,
            resource: ProjectResource::Project,
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
            let resolver = test_resolver();
            let project_id = resolver.resolve_project_id(DEFAULT_ORG_ID).await;
            projects.push(Project {
                id: project_id,
                org_id: DEFAULT_ORG_ID.to_string(),
                name: "Test Project".to_string(),
                path: "/tmp/test".to_string(),
                created_at: 0,
                updated_at: 0,
            });
        }

        let handler = ProjectHandler::new(service, test_resolver());
        let args = ProjectArgs {
            action: ProjectAction::Get,
            resource: ProjectResource::Project,
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
