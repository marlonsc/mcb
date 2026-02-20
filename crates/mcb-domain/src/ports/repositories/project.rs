//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#repository-ports)
//!
//! Provides project repository domain definitions.
use async_trait::async_trait;

use crate::entities::project::Project;
use crate::error::Result;

/// Port for project persistence with row-level tenant isolation.
///
/// All query methods require `org_id` to scope data to a single organization.
/// Create/update use the `org_id` embedded in the `Project` entity.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Performs the create operation.
    async fn create(&self, project: &Project) -> Result<()>;
    /// Performs the get by id operation.
    async fn get_by_id(&self, org_id: &str, id: &str) -> Result<Project>;
    /// Performs the get by name operation.
    async fn get_by_name(&self, org_id: &str, name: &str) -> Result<Project>;
    /// Performs the get by path operation.
    async fn get_by_path(&self, org_id: &str, path: &str) -> Result<Project>;
    /// Performs the list operation.
    async fn list(&self, org_id: &str) -> Result<Vec<Project>>;
    /// Performs the update operation.
    async fn update(&self, project: &Project) -> Result<()>;
    /// Performs the delete operation.
    async fn delete(&self, org_id: &str, id: &str) -> Result<()>;
}
