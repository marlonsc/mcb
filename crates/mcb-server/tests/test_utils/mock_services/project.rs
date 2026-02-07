//! Mock implementation of ProjectDetectorService
use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::services::ProjectDetectorService;
use std::path::Path;

pub struct MockProjectService;

impl MockProjectService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProjectDetectorService for MockProjectService {
    async fn detect_all(&self, _path: &Path) -> Vec<ProjectType> {
        vec![]
    }
}
