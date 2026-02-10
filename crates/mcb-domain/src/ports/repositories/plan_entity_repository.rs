use async_trait::async_trait;

use crate::entities::plan::{Plan, PlanReview, PlanVersion};
use crate::error::Result;

#[async_trait]
pub trait PlanEntityRepository: Send + Sync {
    async fn create_plan(&self, plan: &Plan) -> Result<()>;
    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Option<Plan>>;
    async fn list_plans(&self, org_id: &str, project_id: &str) -> Result<Vec<Plan>>;
    async fn update_plan(&self, plan: &Plan) -> Result<()>;
    async fn delete_plan(&self, org_id: &str, id: &str) -> Result<()>;

    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()>;
    async fn get_plan_version(&self, id: &str) -> Result<Option<PlanVersion>>;
    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>>;

    async fn create_plan_review(&self, review: &PlanReview) -> Result<()>;
    async fn get_plan_review(&self, id: &str) -> Result<Option<PlanReview>>;
    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>>;
}
