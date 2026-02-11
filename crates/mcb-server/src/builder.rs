//! MCP Server Builder
//!
//! Builder pattern for constructing MCP servers with dependency injection.
//! Ensures all required dependencies are provided before server construction.

use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
use mcb_domain::ports::repositories::{
    IssueEntityRepository, OrgEntityRepository, PlanEntityRepository, ProjectRepository,
    VcsEntityRepository,
};
use mcb_domain::ports::services::AgentSessionServiceInterface;
use mcb_domain::ports::services::{
    ContextServiceInterface, IndexingServiceInterface, MemoryServiceInterface,
    ProjectDetectorService, SearchServiceInterface, ValidationServiceInterface,
};

use crate::McpServer;

/// Builder for MCP Server with dependency injection
///
/// Ensures all required domain services are provided before server construction.
/// Follows the builder pattern to make server construction explicit and testable.
#[derive(Default)]
pub struct McpServerBuilder {
    indexing_service: Option<Arc<dyn IndexingServiceInterface>>,
    context_service: Option<Arc<dyn ContextServiceInterface>>,
    search_service: Option<Arc<dyn SearchServiceInterface>>,
    validation_service: Option<Arc<dyn ValidationServiceInterface>>,
    memory_service: Option<Arc<dyn MemoryServiceInterface>>,
    agent_session_service: Option<Arc<dyn AgentSessionServiceInterface>>,
    project_service: Option<Arc<dyn ProjectDetectorService>>,
    project_workflow_service: Option<Arc<dyn ProjectRepository>>,
    vcs_provider: Option<Arc<dyn VcsProvider>>,
    vcs_entity_service: Option<Arc<dyn VcsEntityRepository>>,
    plan_entity_service: Option<Arc<dyn PlanEntityRepository>>,
    issue_entity_service: Option<Arc<dyn IssueEntityRepository>>,
    org_entity_service: Option<Arc<dyn OrgEntityRepository>>,
}

impl McpServerBuilder {
    /// Create a new server builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the indexing service
    ///
    /// # Arguments
    /// * `service` - Implementation of the indexing service port
    pub fn with_indexing_service(mut self, service: Arc<dyn IndexingServiceInterface>) -> Self {
        self.indexing_service = Some(service);
        self
    }

    /// Set the context service
    ///
    /// # Arguments
    /// * `service` - Implementation of the context service port
    pub fn with_context_service(mut self, service: Arc<dyn ContextServiceInterface>) -> Self {
        self.context_service = Some(service);
        self
    }

    /// Set the search service
    ///
    /// # Arguments
    /// * `service` - Implementation of the search service port
    pub fn with_search_service(mut self, service: Arc<dyn SearchServiceInterface>) -> Self {
        self.search_service = Some(service);
        self
    }

    /// Set the validation service
    ///
    /// # Arguments
    /// * `service` - Implementation of the validation service port
    pub fn with_validation_service(mut self, service: Arc<dyn ValidationServiceInterface>) -> Self {
        self.validation_service = Some(service);
        self
    }

    /// Set the memory service
    ///
    /// # Arguments
    /// * `service` - Implementation of the memory service port
    pub fn with_memory_service(mut self, service: Arc<dyn MemoryServiceInterface>) -> Self {
        self.memory_service = Some(service);
        self
    }

    /// Set the VCS provider
    ///
    /// # Arguments
    /// * `provider` - Implementation of the VCS provider port
    pub fn with_vcs_provider(mut self, provider: Arc<dyn VcsProvider>) -> Self {
        self.vcs_provider = Some(provider);
        self
    }

    /// Set the agent session service
    pub fn with_agent_session_service(
        mut self,
        service: Arc<dyn AgentSessionServiceInterface>,
    ) -> Self {
        self.agent_session_service = Some(service);
        self
    }

    /// Set the project detector service
    pub fn with_project_service(mut self, service: Arc<dyn ProjectDetectorService>) -> Self {
        self.project_service = Some(service);
        self
    }

    /// Set the project workflow repository used by admin CRUD endpoints.
    pub fn with_project_workflow_service(mut self, repo: Arc<dyn ProjectRepository>) -> Self {
        self.project_workflow_service = Some(repo);
        self
    }

    /// Set the VCS entity repository used by admin CRUD endpoints.
    pub fn with_vcs_entity_service(mut self, repo: Arc<dyn VcsEntityRepository>) -> Self {
        self.vcs_entity_service = Some(repo);
        self
    }

    /// Set the plan entity repository used by admin CRUD endpoints.
    pub fn with_plan_entity_service(mut self, repo: Arc<dyn PlanEntityRepository>) -> Self {
        self.plan_entity_service = Some(repo);
        self
    }

    /// Set the issue entity repository used by admin CRUD endpoints.
    pub fn with_issue_entity_service(mut self, repo: Arc<dyn IssueEntityRepository>) -> Self {
        self.issue_entity_service = Some(repo);
        self
    }

    /// Set the organization entity repository used by admin CRUD endpoints.
    pub fn with_org_entity_service(mut self, repo: Arc<dyn OrgEntityRepository>) -> Self {
        self.org_entity_service = Some(repo);
        self
    }

    /// Build the MCP server
    ///
    /// # Returns
    /// A Result containing the McpServer or an error if dependencies are missing
    ///
    /// # Errors
    /// Returns `BuilderError::MissingDependency` if any required service is not provided
    pub fn build(self) -> Result<McpServer, BuilderError> {
        let indexing_service = self
            .indexing_service
            .ok_or(BuilderError::MissingDependency("indexing service"))?;
        let context_service = self
            .context_service
            .ok_or(BuilderError::MissingDependency("context service"))?;
        let search_service = self
            .search_service
            .ok_or(BuilderError::MissingDependency("search service"))?;
        let validation_service = self
            .validation_service
            .ok_or(BuilderError::MissingDependency("validation service"))?;
        let memory_service = self
            .memory_service
            .ok_or(BuilderError::MissingDependency("memory service"))?;
        let agent_session_service = self
            .agent_session_service
            .ok_or(BuilderError::MissingDependency("agent session service"))?;
        let vcs_provider = self
            .vcs_provider
            .ok_or(BuilderError::MissingDependency("vcs provider"))?;
        let project_service = self
            .project_service
            .ok_or(BuilderError::MissingDependency("project service"))?;
        let project_workflow_service = self
            .project_workflow_service
            .ok_or(BuilderError::MissingDependency("project workflow service"))?;
        let vcs_entity_service = self
            .vcs_entity_service
            .ok_or(BuilderError::MissingDependency("vcs entity service"))?;
        let plan_entity_service = self
            .plan_entity_service
            .ok_or(BuilderError::MissingDependency("plan entity service"))?;
        let issue_entity_service = self
            .issue_entity_service
            .ok_or(BuilderError::MissingDependency("issue entity service"))?;
        let org_entity_service = self
            .org_entity_service
            .ok_or(BuilderError::MissingDependency("org entity service"))?;

        let services = crate::mcp_server::McpServices {
            indexing: indexing_service,
            context: context_service,
            search: search_service,
            validation: validation_service,
            memory: memory_service,
            agent_session: agent_session_service,
            project: project_service,
            project_workflow: project_workflow_service,
            vcs: vcs_provider,
            vcs_entity: vcs_entity_service,
            plan_entity: plan_entity_service,
            issue_entity: issue_entity_service,
            org_entity: org_entity_service,
        };

        Ok(McpServer::new(services))
    }
}

/// Errors that can occur during server building
#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    /// A required dependency was not provided
    #[error("Missing required dependency: {0}")]
    MissingDependency(&'static str),
}
