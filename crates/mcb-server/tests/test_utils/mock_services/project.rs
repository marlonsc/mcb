use async_trait::async_trait;
use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::repositories::ProjectRepository;
use mcb_domain::ports::services::{ProjectDetectorService, ProjectServiceInterface};
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

#[async_trait]
impl ProjectServiceInterface for MockProjectService {
    async fn get_project(
        &self,
        _org_id: &str,
        _id: &str,
    ) -> mcb_domain::error::Result<mcb_domain::entities::project::Project> {
        Ok(mcb_domain::entities::project::Project {
            id: "test".to_string(),
            org_id: DEFAULT_ORG_ID.to_string(),
            name: "test".to_string(),
            path: "/tmp/test".to_string(),
            created_at: 0,
            updated_at: 0,
        })
    }

    async fn list_projects(
        &self,
        _org_id: &str,
    ) -> mcb_domain::error::Result<Vec<mcb_domain::entities::project::Project>> {
        Ok(vec![])
    }
}

#[allow(dead_code)]
pub struct MockProjectRepository;

#[allow(dead_code)]
impl MockProjectRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProjectRepository for MockProjectRepository {
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
    ) -> mcb_domain::error::Result<Option<mcb_domain::entities::project::Project>> {
        Ok(None)
    }
    async fn get_by_name(
        &self,
        _org_id: &str,
        _name: &str,
    ) -> mcb_domain::error::Result<Option<mcb_domain::entities::project::Project>> {
        Ok(None)
    }
    async fn get_by_path(
        &self,
        _org_id: &str,
        _path: &str,
    ) -> mcb_domain::error::Result<Option<mcb_domain::entities::project::Project>> {
        Ok(None)
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

pub struct MockProjectWorkflowService;

#[allow(dead_code)]
impl MockProjectWorkflowService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockProjectWorkflowService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProjectServiceInterface for MockProjectWorkflowService {
    async fn get_project(
        &self,
        _org_id: &str,
        _id: &str,
    ) -> mcb_domain::error::Result<mcb_domain::entities::project::Project> {
        Err(mcb_domain::error::Error::not_found("Project not found"))
    }

    async fn list_projects(
        &self,
        _org_id: &str,
    ) -> mcb_domain::error::Result<Vec<mcb_domain::entities::project::Project>> {
        Ok(vec![])
    }
}
