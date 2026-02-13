use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::repositories::ProjectRepository;
use mcb_domain::ports::services::ProjectDetectorService;
use std::path::Path;

#[allow(dead_code)]
pub struct TestProjectDetectorService;

impl TestProjectDetectorService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProjectDetectorService for TestProjectDetectorService {
    async fn detect_all(&self, _path: &Path) -> Vec<ProjectType> {
        vec![]
    }
}

#[allow(dead_code)]
pub struct TestProjectRepository;

#[allow(dead_code)]
impl TestProjectRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProjectRepository for TestProjectRepository {
    async fn create(
        &self,
        _project: &mcb_domain::entities::project::Project,
    ) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn get_by_id(
        &self,
        _org_id: &str,
        _id: &str,
    ) -> mcb_domain::error::Result<mcb_domain::entities::project::Project> {
        Err(mcb_domain::error::Error::not_found("Project not found"))
    }
    async fn get_by_name(
        &self,
        _org_id: &str,
        _name: &str,
    ) -> mcb_domain::error::Result<mcb_domain::entities::project::Project> {
        Err(mcb_domain::error::Error::not_found("Project not found"))
    }
    async fn get_by_path(
        &self,
        _org_id: &str,
        _path: &str,
    ) -> mcb_domain::error::Result<mcb_domain::entities::project::Project> {
        Err(mcb_domain::error::Error::not_found("Project not found"))
    }
    async fn list(
        &self,
        _org_id: &str,
    ) -> mcb_domain::error::Result<Vec<mcb_domain::entities::project::Project>> {
        Ok(vec![])
    }
    async fn update(
        &self,
        _project: &mcb_domain::entities::project::Project,
    ) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn delete(&self, _org_id: &str, _id: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }
}
