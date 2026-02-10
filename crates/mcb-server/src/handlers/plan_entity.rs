use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::plan::{Plan, PlanReview, PlanVersion};
use mcb_domain::ports::services::PlanEntityServiceInterface;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{PlanEntityAction, PlanEntityArgs, PlanEntityResource};

/// Handler for the consolidated `plan_entity` MCP tool.
pub struct PlanEntityHandler {
    service: Arc<dyn PlanEntityServiceInterface>,
}

impl PlanEntityHandler {
    /// Create a new handler wrapping the given service.
    pub fn new(service: Arc<dyn PlanEntityServiceInterface>) -> Self {
        Self { service }
    }

    /// Route an incoming `plan_entity` tool call to the appropriate CRUD operation.
    pub async fn handle(
        &self,
        Parameters(args): Parameters<PlanEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        let org_id = args.org_id.as_deref().unwrap_or(DEFAULT_ORG_ID);

        match (args.action, args.resource) {
            (PlanEntityAction::Create, PlanEntityResource::Plan) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for create", None))?;
                let plan: Plan = serde_json::from_value(data)
                    .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                self.service.create_plan(&plan).await.map_err(to_mcp)?;
                ok_json(&plan)
            }
            (PlanEntityAction::Get, PlanEntityResource::Plan) => {
                let id = require_id(&args.id)?;
                let plan = self.service.get_plan(org_id, &id).await.map_err(to_mcp)?;
                ok_json(&plan)
            }
            (PlanEntityAction::List, PlanEntityResource::Plan) => {
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for list", None)
                })?;
                let plans = self
                    .service
                    .list_plans(org_id, project_id)
                    .await
                    .map_err(to_mcp)?;
                ok_json(&plans)
            }
            (PlanEntityAction::Update, PlanEntityResource::Plan) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for update", None))?;
                let plan: Plan = serde_json::from_value(data)
                    .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                self.service.update_plan(&plan).await.map_err(to_mcp)?;
                ok_text("updated")
            }
            (PlanEntityAction::Delete, PlanEntityResource::Plan) => {
                let id = require_id(&args.id)?;
                self.service
                    .delete_plan(org_id, &id)
                    .await
                    .map_err(to_mcp)?;
                ok_text("deleted")
            }
            (PlanEntityAction::Create, PlanEntityResource::Version) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required", None))?;
                let version: PlanVersion = serde_json::from_value(data)
                    .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                self.service
                    .create_plan_version(&version)
                    .await
                    .map_err(to_mcp)?;
                ok_json(&version)
            }
            (PlanEntityAction::Get, PlanEntityResource::Version) => {
                let id = require_id(&args.id)?;
                let version = self.service.get_plan_version(&id).await.map_err(to_mcp)?;
                ok_json(&version)
            }
            (PlanEntityAction::List, PlanEntityResource::Version) => {
                let plan_id = args
                    .plan_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("plan_id required", None))?;
                let versions = self
                    .service
                    .list_plan_versions_by_plan(plan_id)
                    .await
                    .map_err(to_mcp)?;
                ok_json(&versions)
            }
            (PlanEntityAction::Create, PlanEntityResource::Review) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required", None))?;
                let review: PlanReview = serde_json::from_value(data)
                    .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                self.service
                    .create_plan_review(&review)
                    .await
                    .map_err(to_mcp)?;
                ok_json(&review)
            }
            (PlanEntityAction::Get, PlanEntityResource::Review) => {
                let id = require_id(&args.id)?;
                let review = self.service.get_plan_review(&id).await.map_err(to_mcp)?;
                ok_json(&review)
            }
            (PlanEntityAction::List, PlanEntityResource::Review) => {
                let plan_version_id = args
                    .plan_version_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("plan_version_id required", None))?;
                let reviews = self
                    .service
                    .list_plan_reviews_by_version(plan_version_id)
                    .await
                    .map_err(to_mcp)?;
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

fn require_id(id: &Option<String>) -> Result<String, McpError> {
    id.clone()
        .ok_or_else(|| McpError::invalid_params("id required", None))
}

fn ok_json<T: serde::Serialize>(val: &T) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(val)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}

fn ok_text(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        msg,
    )]))
}

fn to_mcp(e: mcb_domain::error::Error) -> McpError {
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
    use mcb_domain::error::Result;
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
    impl PlanEntityServiceInterface for MockPlanEntityService {
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
                status: "draft".into(),
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
                status: "draft".into(),
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
