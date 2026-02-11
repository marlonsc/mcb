//! Project Service Implementation

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::Project;
use mcb_domain::error::Result;
use mcb_domain::ports::repositories::ProjectRepository;
use mcb_domain::ports::services::project::ProjectServiceInterface;

/// Service implementation for managing project workflow resources.
pub struct ProjectServiceImpl {
    repository: Arc<dyn ProjectRepository>,
}

impl ProjectServiceImpl {
    /// Creates a new ProjectServiceImpl.
    pub fn new(repository: Arc<dyn ProjectRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl ProjectServiceInterface for ProjectServiceImpl {
    async fn get_project(&self, org_id: &str, id: &str) -> Result<Project> {
        self.repository.get_by_id(org_id, id).await
    }

    async fn list_projects(&self, org_id: &str) -> Result<Vec<Project>> {
        self.repository.list(org_id).await
    }
}
