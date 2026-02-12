use std::sync::Arc;

use mcb_domain::entities::repository::{Branch, Repository};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
use mcb_domain::ports::repositories::VcsEntityRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{VcsEntityAction, VcsEntityArgs, VcsEntityResource};
use crate::handler_helpers::{
    current_timestamp, map_opaque_error, ok_json, ok_text, require_data, require_id, resolve_org_id,
};

/// Handler for the consolidated `vcs_entity` MCP tool.
pub struct VcsEntityHandler {
    repo: Arc<dyn VcsEntityRepository>,
}

impl VcsEntityHandler {
    /// Create a new VCS entity handler backed by a repository implementation.
    pub fn new(repo: Arc<dyn VcsEntityRepository>) -> Self {
        Self { repo }
    }

    /// Route an incoming `vcs_entity` tool call to the appropriate CRUD operation.
    #[tracing::instrument(
        skip(self),
        fields(action = ?args.action, resource = ?args.resource, org_id = tracing::field::Empty)
    )]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<VcsEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        let org_id = resolve_org_id(args.org_id.as_deref());
        tracing::Span::current().record("org_id", org_id.as_str());

        crate::entity_crud_dispatch! {
            action = args.action,
            resource = args.resource,
            fallback = |action, resource| {
                tracing::warn!(
                    ?action,
                    ?resource,
                    "unsupported action/resource combination"
                );
                Err(McpError::invalid_params(
                    "unsupported action/resource combination",
                    None,
                ))
            },
            {
            // -- Repository --
            (VcsEntityAction::Create, VcsEntityResource::Repository) => {
                let mut repo: Repository = require_data(args.data, "data required for create")?;
                repo.org_id = org_id.to_string();
                map_opaque_error(self.repo.create_repository(&repo).await)?;
                ok_json(&repo)
            }
            (VcsEntityAction::Get, VcsEntityResource::Repository) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_repository(&org_id, &id).await)?)
            }
            (VcsEntityAction::List, VcsEntityResource::Repository) => {
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for list", None)
                })?;
                ok_json(&map_opaque_error(self.repo.list_repositories(&org_id, project_id).await)?)
            }
            (VcsEntityAction::Update, VcsEntityResource::Repository) => {
                let mut repo: Repository = require_data(args.data, "data required for update")?;
                repo.org_id = org_id.to_string();
                map_opaque_error(self.repo.update_repository(&repo).await)?;
                ok_text("updated")
            }
            (VcsEntityAction::Delete, VcsEntityResource::Repository) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_repository(&org_id, &id).await)?;
                ok_text("deleted")
            }

            // -- Branch --
            (VcsEntityAction::Create, VcsEntityResource::Branch) => {
                let branch: Branch = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.create_branch(&branch).await)?;
                ok_json(&branch)
            }
            (VcsEntityAction::Get, VcsEntityResource::Branch) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_branch(&id).await)?)
            }
            (VcsEntityAction::List, VcsEntityResource::Branch) => {
                let repo_id = args
                    .repository_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("repository_id required", None))?;
                ok_json(&map_opaque_error(self.repo.list_branches(repo_id).await)?)
            }
            (VcsEntityAction::Update, VcsEntityResource::Branch) => {
                let branch: Branch = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.update_branch(&branch).await)?;
                ok_text("updated")
            }
            (VcsEntityAction::Delete, VcsEntityResource::Branch) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_branch(&id).await)?;
                ok_text("deleted")
            }

            // -- Worktree --
            (VcsEntityAction::Create, VcsEntityResource::Worktree) => {
                let wt: Worktree = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.create_worktree(&wt).await)?;
                ok_json(&wt)
            }
            (VcsEntityAction::Get, VcsEntityResource::Worktree) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_worktree(&id).await)?)
            }
            (VcsEntityAction::List, VcsEntityResource::Worktree) => {
                let repo_id = args
                    .repository_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("repository_id required", None))?;
                ok_json(&map_opaque_error(self.repo.list_worktrees(repo_id).await)?)
            }
            (VcsEntityAction::Update, VcsEntityResource::Worktree) => {
                let wt: Worktree = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.update_worktree(&wt).await)?;
                ok_text("updated")
            }
            (VcsEntityAction::Delete, VcsEntityResource::Worktree) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_worktree(&id).await)?;
                ok_text("deleted")
            }

            // -- Assignment --
            (VcsEntityAction::Create, VcsEntityResource::Assignment) => {
                let asgn: AgentWorktreeAssignment = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.create_assignment(&asgn).await)?;
                ok_json(&asgn)
            }
            (VcsEntityAction::Get, VcsEntityResource::Assignment) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_assignment(&id).await)?)
            }
            (VcsEntityAction::List, VcsEntityResource::Assignment) => {
                let wt_id = args
                    .worktree_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("worktree_id required", None))?;
                ok_json(&map_opaque_error(self.repo.list_assignments_by_worktree(wt_id).await)?)
            }
            (VcsEntityAction::Release, VcsEntityResource::Assignment) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.release_assignment(&id, current_timestamp()).await)?;
                ok_text("released")
            }

            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mcb_domain::error::Result;
    use mcb_domain::keys::DEFAULT_ORG_ID;
    use std::sync::Mutex;

    struct MockVcsEntityService {
        repos: Mutex<Vec<Repository>>,
    }

    impl MockVcsEntityService {
        fn new() -> Self {
            Self {
                repos: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl VcsEntityRepository for MockVcsEntityService {
        async fn create_repository(&self, repo: &Repository) -> Result<()> {
            self.repos.lock().unwrap().push(repo.clone());
            Ok(())
        }
        async fn get_repository(&self, _org_id: &str, id: &str) -> Result<Repository> {
            self.repos
                .lock()
                .unwrap()
                .iter()
                .find(|r| r.id == id)
                .cloned()
                .ok_or_else(|| mcb_domain::error::Error::not_found(format!("Repository {id}")))
        }
        async fn list_repositories(
            &self,
            _org_id: &str,
            project_id: &str,
        ) -> Result<Vec<Repository>> {
            Ok(self
                .repos
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.project_id == project_id)
                .cloned()
                .collect())
        }
        async fn update_repository(&self, _repo: &Repository) -> Result<()> {
            Ok(())
        }
        async fn delete_repository(&self, _org_id: &str, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_branch(&self, _branch: &Branch) -> Result<()> {
            Ok(())
        }
        async fn get_branch(&self, _id: &str) -> Result<Branch> {
            Err(mcb_domain::error::Error::not_found("branch"))
        }
        async fn list_branches(&self, _repository_id: &str) -> Result<Vec<Branch>> {
            Ok(vec![])
        }
        async fn update_branch(&self, _branch: &Branch) -> Result<()> {
            Ok(())
        }
        async fn delete_branch(&self, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_worktree(&self, _wt: &Worktree) -> Result<()> {
            Ok(())
        }
        async fn get_worktree(&self, _id: &str) -> Result<Worktree> {
            Err(mcb_domain::error::Error::not_found("worktree"))
        }
        async fn list_worktrees(&self, _repository_id: &str) -> Result<Vec<Worktree>> {
            Ok(vec![])
        }
        async fn update_worktree(&self, _wt: &Worktree) -> Result<()> {
            Ok(())
        }
        async fn delete_worktree(&self, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_assignment(&self, _asgn: &AgentWorktreeAssignment) -> Result<()> {
            Ok(())
        }
        async fn get_assignment(&self, _id: &str) -> Result<AgentWorktreeAssignment> {
            Err(mcb_domain::error::Error::not_found("assignment"))
        }
        async fn list_assignments_by_worktree(
            &self,
            _worktree_id: &str,
        ) -> Result<Vec<AgentWorktreeAssignment>> {
            Ok(vec![])
        }
        async fn release_assignment(&self, _id: &str, _released_at: i64) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_handle_list_repositories() {
        let service = Arc::new(MockVcsEntityService::new());
        {
            service.repos.lock().unwrap().push(Repository {
                id: "r1".into(),
                org_id: DEFAULT_ORG_ID.into(),
                project_id: "p1".into(),
                name: "my-repo".into(),
                url: "https://github.com/org/repo".into(),
                local_path: "/tmp/repo".into(),
                vcs_type: mcb_domain::entities::repository::VcsType::Git,
                created_at: 0,
                updated_at: 0,
            });
        }
        let handler = VcsEntityHandler::new(service);
        let args = VcsEntityArgs {
            action: VcsEntityAction::List,
            resource: VcsEntityResource::Repository,
            id: None,
            org_id: None,
            project_id: Some("p1".into()),
            repository_id: None,
            worktree_id: None,
            data: None,
        };
        let result = handler.handle(Parameters(args)).await.expect("handle ok");
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_handle_get_repository() {
        let service = Arc::new(MockVcsEntityService::new());
        {
            service.repos.lock().unwrap().push(Repository {
                id: "r1".into(),
                org_id: DEFAULT_ORG_ID.into(),
                project_id: "p1".into(),
                name: "my-repo".into(),
                url: "https://github.com/org/repo".into(),
                local_path: "/tmp/repo".into(),
                vcs_type: mcb_domain::entities::repository::VcsType::Git,
                created_at: 0,
                updated_at: 0,
            });
        }
        let handler = VcsEntityHandler::new(service);
        let args = VcsEntityArgs {
            action: VcsEntityAction::Get,
            resource: VcsEntityResource::Repository,
            id: Some("r1".into()),
            org_id: None,
            project_id: None,
            repository_id: None,
            worktree_id: None,
            data: None,
        };
        let result = handler.handle(Parameters(args)).await.expect("handle ok");
        assert!(!result.content.is_empty());
    }
}
