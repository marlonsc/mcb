//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Project Service Implementation
//!
//! # Overview
//! The `ProjectService` provides capabilities for identifying and classifying software projects
//! within a directory structure. It acts as the infrastructure adapter for project detection logic.
//!
//! # Responsibilities
//! - **Language Detection**: Identifying primary programming languages.
//! - **Framework Recognition**: Detecting project frameworks (e.g., React, Django, cargo).
//! - **Monorepo Handling**: Scanning nested projects within a workspace.

use std::path::Path;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::{ProjectDetectorConfig, ProjectDetectorService};
use mcb_domain::registry::project_detection::PROJECT_DETECTORS;

/// Infrastructure service for project detection and scanning.
///
/// Wraps `mcb-providers` git and filesystem utilities to recursively discover
/// and classify projects across the workspace.
#[derive(Debug, PartialEq)]
pub struct ProjectService;

impl ProjectService {
    /// Creates a project detector service instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProjectService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProjectDetectorService for ProjectService {
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType> {
        let config = ProjectDetectorConfig {
            repo_path: path.to_str().unwrap_or_default().to_owned(),
        };

        let mut results = Vec::new();

        for entry in PROJECT_DETECTORS {
            let has_marker = entry.marker_files.iter().any(|f| path.join(f).exists());
            if !has_marker {
                continue;
            }

            if let Ok(detector) = (entry.build)(&config)
                && let Ok(Some(project_type)) = detector.detect(path).await
            {
                results.push(project_type);
            }
        }

        results
    }
}
