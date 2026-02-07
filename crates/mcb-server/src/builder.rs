//! MCP Server Builder
//!
//! Builder pattern for constructing MCP servers with dependency injection.
//! Ensures all required dependencies are provided before server construction.

use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
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
    vcs_provider: Option<Arc<dyn VcsProvider>>,
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

        let services = crate::mcp_server::McpServices {
            indexing: indexing_service,
            context: context_service,
            search: search_service,
            validation: validation_service,
            memory: memory_service,
            agent_session: agent_session_service,
            project: project_service,
            vcs: vcs_provider,
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
