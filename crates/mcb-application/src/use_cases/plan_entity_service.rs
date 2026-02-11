//! Plan entity service implementation.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::plan::{Plan, PlanReview, PlanVersion};
use mcb_domain::error::Result;
use mcb_domain::ports::repositories::PlanEntityRepository;
use mcb_domain::ports::services::PlanEntityServiceInterface;

/// Application-layer service for plan entity CRUD operations.
pub struct PlanEntityServiceImpl {
    repository: Arc<dyn PlanEntityRepository>,
}

impl PlanEntityServiceImpl {
    /// Create a new [`PlanEntityServiceImpl`] backed by the given repository.
    pub fn new(repository: Arc<dyn PlanEntityRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl PlanEntityServiceInterface for PlanEntityServiceImpl {
    async fn create_plan(&self, plan: &Plan) -> Result<()> {
        self.repository.create_plan(plan).await
    }

    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Plan> {
        self.repository.get_plan(org_id, id).await
    }

    async fn list_plans(&self, org_id: &str, project_id: &str) -> Result<Vec<Plan>> {
        self.repository.list_plans(org_id, project_id).await
    }

    async fn update_plan(&self, plan: &Plan) -> Result<()> {
        self.repository.update_plan(plan).await
    }

    async fn delete_plan(&self, org_id: &str, id: &str) -> Result<()> {
        self.repository.delete_plan(org_id, id).await
    }

    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()> {
        self.repository.create_plan_version(version).await
    }

    async fn get_plan_version(&self, id: &str) -> Result<PlanVersion> {
        self.repository.get_plan_version(id).await
    }

    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>> {
        self.repository.list_plan_versions_by_plan(plan_id).await
    }

    async fn create_plan_review(&self, review: &PlanReview) -> Result<()> {
        self.repository.create_plan_review(review).await
    }

    async fn get_plan_review(&self, id: &str) -> Result<PlanReview> {
        self.repository.get_plan_review(id).await
    }

    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>> {
        self.repository
            .list_plan_reviews_by_version(plan_version_id)
            .await
    }
}
