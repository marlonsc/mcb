use std::sync::Arc;

use mcb_application::services::RepositoryResolver;
use mcb_domain::entities::plan::{Plan, PlanReview, PlanVersion};
use mcb_domain::ports::repositories::PlanEntityRepository;
use mcb_domain::value_objects::OrgContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{PlanEntityAction, PlanEntityArgs, PlanEntityResource};
use crate::error_mapping::to_opaque_mcp_error;
use crate::handler_helpers::{ok_json, ok_text, require_id};

/// Handler for the consolidated `plan_entity` MCP tool.
pub struct PlanEntityHandler {
    repo: Arc<dyn PlanEntityRepository>,
    resolver: Arc<RepositoryResolver>,
}

impl PlanEntityHandler {
    pub fn new(repo: Arc<dyn PlanEntityRepository>, resolver: Arc<RepositoryResolver>) -> Self {
        Self { repo, resolver }
    }

    /// Route an incoming `plan_entity` tool call to the appropriate CRUD operation.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<PlanEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        let org_ctx = OrgContext::current();
        let org_id = org_ctx.id_str();

        match (args.action, args.resource) {
            (PlanEntityAction::Create, PlanEntityResource::Plan) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for create", None))?;
                let mut plan: Plan = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                plan.org_id = org_id.to_string();
                self.repo
                    .create_plan(&plan)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&plan)
            }
            (PlanEntityAction::Get, PlanEntityResource::Plan) => {
                let id = require_id(&args.id)?;
                let plan = self
                    .repo
                    .get_plan(org_id, &id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&plan)
            }
            (PlanEntityAction::List, PlanEntityResource::Plan) => {
                let project_id = self.resolver.resolve_project_id(org_id).await;
                let plans = self
                    .repo
                    .list_plans(org_id, &project_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&plans)
            }
            (PlanEntityAction::Update, PlanEntityResource::Plan) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for update", None))?;
                let mut plan: Plan = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                plan.org_id = org_id.to_string();
                self.repo
                    .update_plan(&plan)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("updated")
            }
            (PlanEntityAction::Delete, PlanEntityResource::Plan) => {
                let id = require_id(&args.id)?;
                self.repo
                    .delete_plan(org_id, &id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("deleted")
            }
            (PlanEntityAction::Create, PlanEntityResource::Version) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required", None))?;
                let mut version: PlanVersion = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                version.org_id = org_id.to_string();
                self.repo
                    .create_plan_version(&version)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&version)
            }
            (PlanEntityAction::Get, PlanEntityResource::Version) => {
                let id = require_id(&args.id)?;
                let version = self
                    .repo
                    .get_plan_version(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&version)
            }
            (PlanEntityAction::List, PlanEntityResource::Version) => {
                let plan_id = args
                    .plan_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("plan_id required", None))?;
                let versions = self
                    .repo
                    .list_plan_versions_by_plan(plan_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&versions)
            }
            (PlanEntityAction::Create, PlanEntityResource::Review) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required", None))?;
                let mut review: PlanReview = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                review.org_id = org_id.to_string();
                self.repo
                    .create_plan_review(&review)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&review)
            }
            (PlanEntityAction::Get, PlanEntityResource::Review) => {
                let id = require_id(&args.id)?;
                let review = self
                    .repo
                    .get_plan_review(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&review)
            }
            (PlanEntityAction::List, PlanEntityResource::Review) => {
                let plan_version_id = args
                    .plan_version_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("plan_version_id required", None))?;
                let reviews = self
                    .repo
                    .list_plan_reviews_by_version(plan_version_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&reviews)
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
    use mcb_domain::error::Result;
    use mcb_domain::keys::DEFAULT_ORG_ID;
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

    struct MockPlanEntityService {
        plans: Mutex<Vec<Plan>>,
    }

    impl MockPlanEntityService {
        fn new() -> Self {
            Self {
                plans: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl PlanEntityRepository for MockPlanEntityService {
        async fn create_plan(&self, plan: &Plan) -> Result<()> {
            self.plans.lock().expect("lock plans").push(plan.clone());
            Ok(())
        }
        async fn get_plan(&self, _org_id: &str, id: &str) -> Result<Plan> {
            self.plans
                .lock()
                .expect("lock plans")
                .iter()
                .find(|p| p.id == id)
                .cloned()
                .ok_or_else(|| mcb_domain::error::Error::not_found(format!("Plan {id}")))
        }
        async fn list_plans(&self, _org_id: &str, project_id: &str) -> Result<Vec<Plan>> {
            Ok(self
                .plans
                .lock()
                .expect("lock plans")
                .iter()
                .filter(|p| p.project_id == project_id)
                .cloned()
                .collect())
        }
        async fn update_plan(&self, _plan: &Plan) -> Result<()> {
            Ok(())
        }
        async fn delete_plan(&self, _org_id: &str, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_plan_version(&self, _version: &PlanVersion) -> Result<()> {
            Ok(())
        }
        async fn get_plan_version(&self, _id: &str) -> Result<PlanVersion> {
            Err(mcb_domain::error::Error::not_found("version"))
        }
        async fn list_plan_versions_by_plan(&self, _plan_id: &str) -> Result<Vec<PlanVersion>> {
            Ok(vec![])
        }
        async fn create_plan_review(&self, _review: &PlanReview) -> Result<()> {
            Ok(())
        }
        async fn get_plan_review(&self, _id: &str) -> Result<PlanReview> {
            Err(mcb_domain::error::Error::not_found("review"))
        }
        async fn list_plan_reviews_by_version(
            &self,
            _plan_version_id: &str,
        ) -> Result<Vec<PlanReview>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_list_plans() {
        let service = Arc::new(MockPlanEntityService::new());
        {
            service.plans.lock().expect("lock plans").push(Plan {
                id: "p1".into(),
                org_id: DEFAULT_ORG_ID.into(),
                project_id: "proj-1".into(),
                title: "plan".into(),
                description: "desc".into(),
                status: mcb_domain::entities::plan::PlanStatus::Draft,
                created_by: "user-1".into(),
                created_at: 0,
                updated_at: 0,
            });
        }
        let handler = PlanEntityHandler::new(service, test_resolver());
        let args = PlanEntityArgs {
            action: PlanEntityAction::List,
            resource: PlanEntityResource::Plan,
            id: None,
            plan_id: None,
            plan_version_id: None,
            data: None,
        };
        let result = handler.handle(Parameters(args)).await.expect("handle ok");
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_get_plan() {
        let service = Arc::new(MockPlanEntityService::new());
        {
            service.plans.lock().expect("lock plans").push(Plan {
                id: "p1".into(),
                org_id: DEFAULT_ORG_ID.into(),
                project_id: "proj-1".into(),
                title: "plan".into(),
                description: "desc".into(),
                status: mcb_domain::entities::plan::PlanStatus::Draft,
                created_by: "user-1".into(),
                created_at: 0,
                updated_at: 0,
            });
        }
        let handler = PlanEntityHandler::new(service, test_resolver());
        let args = PlanEntityArgs {
            action: PlanEntityAction::Get,
            resource: PlanEntityResource::Plan,
            id: Some("p1".into()),
            plan_id: None,
            plan_version_id: None,
            data: None,
        };
        let result = handler.handle(Parameters(args)).await.expect("handle ok");
        assert!(!result.content.is_empty());
    }
}
