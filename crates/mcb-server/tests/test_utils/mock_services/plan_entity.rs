use async_trait::async_trait;
use mcb_domain::entities::plan::{Plan, PlanReview, PlanVersion};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::repositories::PlanEntityRepository;

#[allow(dead_code)]
pub struct MockPlanEntityService;

#[allow(dead_code)]
impl MockPlanEntityService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockPlanEntityService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlanEntityRepository for MockPlanEntityService {
    async fn create_plan(&self, _plan: &Plan) -> Result<()> {
        Ok(())
    }

    async fn get_plan(&self, _org_id: &str, _id: &str) -> Result<Plan> {
        Err(Error::not_found("not found"))
    }

    async fn list_plans(&self, _org_id: &str, _project_id: &str) -> Result<Vec<Plan>> {
        Ok(vec![])
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
        Err(Error::not_found("not found"))
    }

    async fn list_plan_versions_by_plan(&self, _plan_id: &str) -> Result<Vec<PlanVersion>> {
        Ok(vec![])
    }

    async fn create_plan_review(&self, _review: &PlanReview) -> Result<()> {
        Ok(())
    }

    async fn get_plan_review(&self, _id: &str) -> Result<PlanReview> {
        Err(Error::not_found("not found"))
    }

    async fn list_plan_reviews_by_version(
        &self,
        _plan_version_id: &str,
    ) -> Result<Vec<PlanReview>> {
        Ok(vec![])
    }
}
