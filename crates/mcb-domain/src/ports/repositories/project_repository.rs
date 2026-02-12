use async_trait::async_trait;

use crate::entities::project::Project;
use crate::error::Result;

/// Port for project persistence with row-level tenant isolation.
///
/// All query methods require `org_id` to scope data to a single organization.
/// Create/update use the `org_id` embedded in the `Project` entity.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, project: &Project) -> Result<()>;
    async fn get_by_id(&self, org_id: &str, id: &str) -> Result<Project>;
    async fn get_by_name(&self, org_id: &str, name: &str) -> Result<Project>;
    async fn get_by_path(&self, org_id: &str, path: &str) -> Result<Project>;
    async fn list(&self, org_id: &str) -> Result<Vec<Project>>;
    async fn update(&self, project: &Project) -> Result<()>;
    async fn delete(&self, org_id: &str, id: &str) -> Result<()>;
}
