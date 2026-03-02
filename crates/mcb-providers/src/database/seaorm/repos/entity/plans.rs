//! Plan registry implementations.
//!
//! Implements `PlanRegistry`, `PlanVersionRegistry`, and `PlanReviewRegistry`
//! for managing plans, versions, and reviews.

use super::*;

#[async_trait]
impl PlanRegistry for SeaOrmEntityRepository {
    async fn create_plan(&self, p: &Plan) -> Result<()> {
        sea_insert!(self, plan, p)
    }

    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Plan> {
        sea_get_filtered!(self, plan, Plan, "Plan", id, plan::Column::OrgId => org_id)
    }

    async fn list_plans(&self, org_id: &str, project_id: &str) -> Result<Vec<Plan>> {
        sea_list!(self, plan, Plan, plan::Column::OrgId => org_id, plan::Column::ProjectId => project_id)
    }

    async fn update_plan(&self, p: &Plan) -> Result<()> {
        sea_update!(self, plan, p)
    }

    async fn delete_plan(&self, org_id: &str, id: &str) -> Result<()> {
        sea_delete_filtered!(self, plan, id, plan::Column::OrgId => org_id)
    }
}

#[async_trait]
impl PlanVersionRegistry for SeaOrmEntityRepository {
    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()> {
        sea_insert!(self, plan_version, version)
    }

    async fn get_plan_version(&self, id: &str) -> Result<PlanVersion> {
        sea_get!(self, plan_version, PlanVersion, "PlanVersion", id)
    }

    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>> {
        sea_list!(self, plan_version, PlanVersion, plan_version::Column::PlanId => plan_id)
    }
}

#[async_trait]
impl PlanReviewRegistry for SeaOrmEntityRepository {
    async fn create_plan_review(&self, review: &PlanReview) -> Result<()> {
        sea_insert!(self, plan_review, review)
    }

    async fn get_plan_review(&self, id: &str) -> Result<PlanReview> {
        sea_get!(self, plan_review, PlanReview, "PlanReview", id)
    }

    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>> {
        sea_list!(self, plan_review, PlanReview, plan_review::Column::PlanVersionId => plan_version_id)
    }
}
