use crate::entities::Project;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, project: &Project) -> Result<()>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Project>>;
    async fn get_by_name(&self, name: &str) -> Result<Option<Project>>;
    async fn get_by_path(&self, path: &str) -> Result<Option<Project>>;
    async fn list(&self) -> Result<Vec<Project>>;
    async fn update(&self, project: &Project) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
}
