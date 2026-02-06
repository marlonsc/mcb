use crate::entities::project::ProjectType;
use async_trait::async_trait;
use std::path::Path;

/// Detects project types inside a repository path
#[async_trait]
pub trait ProjectDetectorService: Send + Sync {
    /// Detect all project types under the given path
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType>;
}
