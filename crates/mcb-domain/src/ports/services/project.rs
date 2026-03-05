//! Project detection service ports.

use std::path::Path;

use async_trait::async_trait;

use crate::entities::project::ProjectType;

/// Defines behavior for `ProjectDetectorService`.
#[async_trait]
pub trait ProjectDetectorService: Send + Sync {
    /// Perform project detection on all subdirectories within `path`.
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType>;
}
