//! Project detection provider ports.

use std::path::Path;

use async_trait::async_trait;

use crate::entities::project::ProjectType;
use crate::error::Result;

/// Project detector trait.
#[async_trait]
pub trait ProjectDetector: Send + Sync {
    /// Attempt to identify the project type at the given path.
    async fn detect(&self, path: &Path) -> Result<Option<ProjectType>>;
    /// Get the marker files used by this detector.
    fn marker_files(&self) -> &[&str];
    /// Get the unique name of this detector implementation.
    fn detector_name(&self) -> &str;
}
