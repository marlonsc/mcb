//! Plan entity CRUD handler implementation.

use std::sync::Arc;

use mcb_domain::entities::plan::{Plan, PlanReview, PlanVersion};
use mcb_domain::ports::repositories::PlanEntityRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{PlanEntityAction, PlanEntityArgs, PlanEntityResource};
use crate::utils::mcp::{
    map_opaque_error, ok_json, ok_text, require_data, require_id, require_resolved_identifier,
    resolve_org_id,
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
    ///
    /// This method acts as a dispatcher for all plan-related entities including plans,
    /// versions, and reviews. It handles authorization context (`org_id`) and ensures
    /// all required data is present.
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
                plan.project_id = require_resolved_identifier(
                    "project_id",
                    args.project_id.as_deref(),
                    Some(plan.project_id.as_str()),
                    "project_id required for plan create",
                )?;
                plan.org_id = org_id.clone();
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
                plan.org_id = org_id.clone();
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
                version.org_id = org_id.clone();
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
                review.org_id = org_id.clone();
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
