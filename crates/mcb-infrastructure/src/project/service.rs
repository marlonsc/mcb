//! Project Service Implementation
//!
//! Implements `ProjectDetectorService` using `mcb-providers` git detection features.

use std::path::Path;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::services::project::ProjectDetectorService;
use mcb_providers::git::detect_all_projects;

/// Real implementation of project detector service
#[derive(Debug, PartialEq)]
pub struct ProjectService;

impl ProjectService {
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
