//! Plan registry implementations.
//!
//! Implements `PlanRegistry`, `PlanVersionRegistry`, and `PlanReviewRegistry`
//! for managing plans, versions, and reviews.

use super::*;

sea_impl_crud_scoped!(PlanRegistry for SeaOrmEntityRepository { db: db,
    entity: plan, domain: Plan, label: "Plan",
    scope_col: plan::Column::OrgId,
    create: create_plan(p),
    get: get_plan,
    list: list_plans(plan::Column::ProjectId => project_id),
    update: update_plan(p),
    delete: delete_plan
});

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
        sea_repo_list!(self.db(), plan_version, PlanVersion, "list plan versions",
            plan_version::Column::PlanId => plan_id)
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
        sea_repo_list!(self.db(), plan_review, PlanReview, "list plan reviews",
            plan_review::Column::PlanVersionId => plan_version_id)
    }
}
