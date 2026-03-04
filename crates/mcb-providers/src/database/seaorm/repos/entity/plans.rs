//! Plan registry implementations.
//!
//! Implements `PlanRegistry`, `PlanVersionRegistry`, and `PlanReviewRegistry`
//! for managing plans, versions, and reviews.

use super::*;

#[async_trait]
impl PlanRegistry for SeaOrmEntityRepository {
    async fn create_plan(&self, p: &Plan) -> Result<()> {
        sea_repo_insert!(self.db(), plan, p, "create plan")
    }

    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Plan> {
        sea_repo_get_filtered!(self.db(), plan, Plan, "Plan", id, "get plan", plan::Column::OrgId => org_id)
    }

    async fn list_plans(&self, org_id: &str, project_id: &str) -> Result<Vec<Plan>> {
        sea_repo_list!(self.db(), plan, Plan, "list plans", plan::Column::OrgId => org_id, plan::Column::ProjectId => project_id)
    }

    async fn update_plan(&self, p: &Plan) -> Result<()> {
        sea_repo_update!(self.db(), plan, p, "update plan")
    }

    async fn delete_plan(&self, org_id: &str, id: &str) -> Result<()> {
        sea_repo_delete_filtered!(self.db(), plan, id, "delete plan", plan::Column::OrgId => org_id)
    }
}

#[async_trait]
impl PlanVersionRegistry for SeaOrmEntityRepository {
    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()> {
        sea_repo_insert!(self.db(), plan_version, version, "create plan version")
    }

    async fn get_plan_version(&self, id: &str) -> Result<PlanVersion> {
        sea_repo_get!(
            self.db(),
            plan_version,
            PlanVersion,
            "PlanVersion",
            id,
            "get plan version"
        )
    }

    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>> {
        sea_repo_list!(self.db(), plan_version, PlanVersion, "get plan versions", plan_version::Column::PlanId => plan_id)
    }
}

#[async_trait]
impl PlanReviewRegistry for SeaOrmEntityRepository {
    async fn create_plan_review(&self, review: &PlanReview) -> Result<()> {
        sea_repo_insert!(self.db(), plan_review, review, "create plan review")
    }

    async fn get_plan_review(&self, id: &str) -> Result<PlanReview> {
        sea_repo_get!(
            self.db(),
            plan_review,
            PlanReview,
            "PlanReview",
            id,
            "get plan review"
        )
    }

    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>> {
        sea_repo_list!(self.db(), plan_review, PlanReview, "get plan reviews", plan_review::Column::PlanVersionId => plan_version_id)
    }
}
