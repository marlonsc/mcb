use std::sync::Arc;

use mcb_domain::entities::plan::{Plan, PlanReview, PlanVersion};
use mcb_domain::ports::repositories::PlanEntityRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{PlanEntityAction, PlanEntityArgs, PlanEntityResource};
use crate::handler_helpers::{
    map_opaque_error, ok_json, ok_text, require_data, require_id, resolve_org_id,
};

/// Handler for the consolidated `plan_entity` MCP tool.
pub struct PlanEntityHandler {
    repo: Arc<dyn PlanEntityRepository>,
}

impl PlanEntityHandler {
    /// Create a new plan entity handler backed by a repository implementation.
    pub fn new(repo: Arc<dyn PlanEntityRepository>) -> Self {
        Self { repo }
    }

    /// Route an incoming `plan_entity` tool call to the appropriate CRUD operation.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<PlanEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        let org_id = resolve_org_id(args.org_id.as_deref());

        crate::entity_crud_dispatch! {
            action = args.action,
            resource = args.resource,
            {
            (PlanEntityAction::Create, PlanEntityResource::Plan) => {
                let mut plan: Plan = require_data(args.data, "data required for create")?;
                plan.org_id = org_id.to_string();
                map_opaque_error(self.repo.create_plan(&plan).await)?;
                ok_json(&plan)
            }
            (PlanEntityAction::Get, PlanEntityResource::Plan) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_plan(org_id.as_str(), &id).await)?)
            }
            (PlanEntityAction::List, PlanEntityResource::Plan) => {
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for list", None)
                })?;
                ok_json(&map_opaque_error(self.repo.list_plans(org_id.as_str(), project_id).await)?)
            }
            (PlanEntityAction::Update, PlanEntityResource::Plan) => {
                let mut plan: Plan = require_data(args.data, "data required for update")?;
                plan.org_id = org_id.to_string();
                map_opaque_error(self.repo.update_plan(&plan).await)?;
                ok_text("updated")
            }
            (PlanEntityAction::Delete, PlanEntityResource::Plan) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_plan(org_id.as_str(), &id).await)?;
                ok_text("deleted")
            }
            (PlanEntityAction::Create, PlanEntityResource::Version) => {
                let mut version: PlanVersion = require_data(args.data, "data required")?;
                version.org_id = org_id.to_string();
                map_opaque_error(self.repo.create_plan_version(&version).await)?;
                ok_json(&version)
            }
            (PlanEntityAction::Get, PlanEntityResource::Version) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_plan_version(&id).await)?)
            }
            (PlanEntityAction::List, PlanEntityResource::Version) => {
                let plan_id = args
                    .plan_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("plan_id required", None))?;
                ok_json(&map_opaque_error(self.repo.list_plan_versions_by_plan(plan_id).await)?)
            }
            (PlanEntityAction::Create, PlanEntityResource::Review) => {
                let mut review: PlanReview = require_data(args.data, "data required")?;
                review.org_id = org_id.to_string();
                map_opaque_error(self.repo.create_plan_review(&review).await)?;
                ok_json(&review)
            }
            (PlanEntityAction::Get, PlanEntityResource::Review) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_plan_review(&id).await)?)
            }
            (PlanEntityAction::List, PlanEntityResource::Review) => {
                let plan_version_id = args
                    .plan_version_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("plan_version_id required", None))?;
                ok_json(&map_opaque_error(self.repo.list_plan_reviews_by_version(plan_version_id).await)?)
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
        let handler = PlanEntityHandler::new(service);
        let args = PlanEntityArgs {
            action: PlanEntityAction::List,
            resource: PlanEntityResource::Plan,
            id: None,
            org_id: None,
            project_id: Some("proj-1".into()),
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
        let handler = PlanEntityHandler::new(service);
        let args = PlanEntityArgs {
            action: PlanEntityAction::Get,
            resource: PlanEntityResource::Plan,
            id: Some("p1".into()),
            org_id: None,
            project_id: None,
            plan_id: None,
            plan_version_id: None,
            data: None,
        };
        let result = handler.handle(Parameters(args)).await.expect("handle ok");
        assert!(!result.content.is_empty());
    }
}
