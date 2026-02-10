use async_trait::async_trait;

use crate::entities::project::Project;
use crate::error::Result;

/// Port for project persistence (CRUD operations on Project entities and related workflow data).
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    // Project CRUD
    /// Creates a new project record.
    async fn create(&self, project: &Project) -> Result<()>;
    /// Retrieves a project by its unique identifier.
    async fn get_by_id(&self, id: &str) -> Result<Option<Project>>;
    /// Retrieves a project by its name.
    async fn get_by_name(&self, name: &str) -> Result<Option<Project>>;
    /// Retrieves a project by its filesystem path.
    async fn get_by_path(&self, path: &str) -> Result<Option<Project>>;
    /// Lists all registered projects.
    async fn list(&self) -> Result<Vec<Project>>;
    /// Updates an existing project record.
    async fn update(&self, project: &Project) -> Result<()>;
    /// Deletes a project by its identifier.
    async fn delete(&self, id: &str) -> Result<()>;
}
