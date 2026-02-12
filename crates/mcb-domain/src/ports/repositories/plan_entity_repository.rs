use async_trait::async_trait;

use crate::entities::plan::{Plan, PlanReview, PlanVersion};
use crate::error::Result;

#[async_trait]
/// Plan entity repository trait

pub trait PlanEntityRepository: Send + Sync {
    /// Create a new plan
    async fn create_plan(&self, plan: &Plan) -> Result<()>;
    /// Get a plan by its ID
    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Plan>;
    /// List all plans for an organization
    async fn list_plans(&self, org_id: &str, project_id: &str) -> Result<Vec<Plan>>;
    /// Update a plan
    async fn update_plan(&self, plan: &Plan) -> Result<()>;
    /// Delete a plan
    async fn delete_plan(&self, org_id: &str, id: &str) -> Result<()>;
    /// Create a new plan version

    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()>;
    /// Get a plan version by its ID
    async fn get_plan_version(&self, id: &str) -> Result<PlanVersion>;
    /// List all plan versions by a plan ID
    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>>;

    /// Create a new plan review
    async fn create_plan_review(&self, review: &PlanReview) -> Result<()>;
    /// Get a plan review by its ID
    async fn get_plan_review(&self, id: &str) -> Result<PlanReview>;
    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>>;
}
