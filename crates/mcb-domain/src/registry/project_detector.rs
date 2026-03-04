//! Project detection registry.

use std::sync::Arc;

use crate::error::Result;
use crate::ports::ProjectDetector;

/// Configuration for project detector initialization.
#[derive(Debug, Clone)]
pub struct ProjectDetectorConfig {
    /// Absolute path to the repository being analyzed.
    pub repo_path: String,
}

/// Registry entry for project detectors.
#[doc(hidden)]
pub struct ProjectDetectorEntry {
    /// Unique name of the detector (e.g., "rust-cargo").
    pub name: &'static str,
    /// Human-readable explanation of what it identifies.
    pub description: &'static str,
    /// List of file names that indicate this project type.
    pub marker_files: &'static [&'static str],
    /// Factory function to build the detector instance.
    pub build: fn(&ProjectDetectorConfig) -> Result<Arc<dyn ProjectDetector>>,
}

/// Distributed slice of registered project detectors.
#[doc(hidden)]
#[linkme::distributed_slice]
pub static PROJECT_DETECTORS: [ProjectDetectorEntry] = [..];
