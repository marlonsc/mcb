use std::path::Path;

use async_trait::async_trait;

use crate::entities::project::ProjectType;

#[async_trait]
pub trait ProjectDetectorService: Send + Sync {
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType>;
}
