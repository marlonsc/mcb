//! Project detection port for identifying project types in repositories.

use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

use crate::entities::project::ProjectType;
use crate::error::Result;

/// Configuration for project detector initialization
#[derive(Debug, Clone)]
pub struct ProjectDetectorConfig {
    /// Root path of the repository being analyzed
    pub repo_path: String,
}

/// Registry entry for project detectors (linkme pattern)
pub struct ProjectDetectorEntry {
    /// Unique detector name
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Marker files this detector looks for (e.g., ["Cargo.toml"])
    pub marker_files: &'static [&'static str],
    /// Factory function to create detector instance
    pub factory:
        fn(&ProjectDetectorConfig) -> std::result::Result<Arc<dyn ProjectDetector>, String>,
}

/// Project detector trait - implementations detect specific project types
#[async_trait]
pub trait ProjectDetector: Send + Sync {
    /// Detect if this project type exists at the given path
    /// Returns Some(ProjectType) if detected, None otherwise
    async fn detect(&self, path: &Path) -> Result<Option<ProjectType>>;

    /// Get marker files this detector looks for
    fn marker_files(&self) -> &[&str];

    /// Detector name for logging
    fn detector_name(&self) -> &str;
}
