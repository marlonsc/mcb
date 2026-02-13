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
use mcb_domain::ports::services::project::ProjectDetectorService;
use mcb_providers::git::detect_all_projects;

/// Infrastructure service for project detection and scanning.
///
/// Wraps `mcb-providers` git and filesystem utilities to recursively discover
/// and classify projects across the workspace.
#[derive(Debug, PartialEq)]
pub struct ProjectService;

impl ProjectService {
    /// Creates a project detector service instance.
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
        detect_all_projects(path).await
    }
}
