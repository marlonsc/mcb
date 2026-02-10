use std::path::Path;

use async_trait::async_trait;

use crate::entities::project::{Project, ProjectType};
use crate::error::Result;

/// Detects project types inside a repository path
#[async_trait]
pub trait ProjectDetectorService: Send + Sync {
    /// Detect all project types under the given path
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType>;
}

/// Service for managing project workflow resources.
#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait ProjectServiceInterface: Send + Sync {
    // Project operations
    /// Gets a project by ID.
    async fn get_project(&self, id: &str) -> Result<Project>;
    /// Lists all registered projects.
    async fn list_projects(&self) -> Result<Vec<Project>>;
}
