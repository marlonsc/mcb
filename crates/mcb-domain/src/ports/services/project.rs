use std::path::Path;

use async_trait::async_trait;

use crate::entities::project::{Project, ProjectType};
use crate::error::Result;

#[async_trait]
pub trait ProjectDetectorService: Send + Sync {
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType>;
}

/// Project workflow service scoped by organization.
#[async_trait]
pub trait ProjectServiceInterface: Send + Sync {
    async fn get_project(&self, org_id: &str, id: &str) -> Result<Project>;
    async fn list_projects(&self, org_id: &str) -> Result<Vec<Project>>;
}
