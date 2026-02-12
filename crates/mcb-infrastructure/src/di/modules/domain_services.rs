//! Domain Services DI Module
//!
//! Provides domain service implementations that can be injected into the server.
//! These services implement domain interfaces using infrastructure components.
//!
//! ## Runtime Factory Pattern
//!
//! Services are created via `DomainServicesFactory::create_services()` at runtime
//! using constructor injection, because they require runtime configuration
//! (embedding provider, vector store, cache).

use std::sync::Arc;

use mcb_application::services::RepositoryResolver;
use mcb_application::use_cases::{
    AgentSessionServiceImpl, ContextServiceImpl, IndexingServiceImpl, MemoryServiceImpl,
    SearchServiceImpl,
};
use mcb_domain::error::Result;
use mcb_domain::ports::admin::IndexingOperationsInterface;
use mcb_domain::ports::infrastructure::EventBusProvider;
use mcb_domain::ports::providers::{
    EmbeddingProvider, LanguageChunkingProvider, VcsProvider, VectorStoreProvider,
};
use mcb_domain::ports::repositories::{
    AgentRepository, IssueEntityRepository, MemoryRepository, OrgEntityRepository,
    PlanEntityRepository, ProjectRepository, VcsEntityRepository,
};
use mcb_domain::ports::services::{
    AgentSessionServiceInterface, ContextServiceInterface, IndexingServiceInterface,
    MemoryServiceInterface, ProjectDetectorService, SearchServiceInterface,
    ValidationServiceInterface,
};

use super::super::bootstrap::AppContext;
use crate::cache::provider::SharedCacheProvider;
use crate::config::AppConfig;
use mcb_domain::ports::providers::CryptoProvider;
// Use infrastructure validation service
use crate::validation::InfraValidationService;

/// Domain services container
///
/// Holds all assembled domain service implementations for use throughout the application.
/// This container is created by `DomainServicesFactory` and provides access to all
/// domain-level services that depend on infrastructure components.
#[derive(Clone)]
pub struct DomainServicesContainer {
    /// Service for managing context and semantic search operations
    pub context_service: Arc<dyn ContextServiceInterface>,
    /// Service for searching across indexed code and memory
    pub search_service: Arc<dyn SearchServiceInterface>,
    /// Service for indexing code and managing the search index
    pub indexing_service: Arc<dyn IndexingServiceInterface>,
    /// Service for validating domain objects and operations
    pub validation_service: Arc<dyn ValidationServiceInterface>,
    /// Service for managing persistent memory and observations
    pub memory_service: Arc<dyn MemoryServiceInterface>,
    /// Service for managing agent sessions and state
    pub agent_session_service: Arc<dyn AgentSessionServiceInterface>,
    /// Service for detecting and managing project information
    pub project_service: Arc<dyn ProjectDetectorService>,
    /// Repository for project CRUD operations
    pub project_repository: Arc<dyn ProjectRepository>,
    /// Provider for version control system operations
    pub vcs_provider: Arc<dyn VcsProvider>,
    /// Repository for VCS entity CRUD (repos, branches, worktrees)
    pub vcs_entity_repository: Arc<dyn VcsEntityRepository>,
    /// Repository for plan entity CRUD (plans, versions, reviews)
    pub plan_entity_repository: Arc<dyn PlanEntityRepository>,
    /// Repository for issue entity CRUD (issues, comments, labels)
    pub issue_entity_repository: Arc<dyn IssueEntityRepository>,
    /// Repository for org entity CRUD (orgs, users, teams, members, api keys)
    pub org_entity_repository: Arc<dyn OrgEntityRepository>,
    /// DB-first project_id resolver — sole source of truth for project context
    pub resolver: Arc<RepositoryResolver>,
}

/// Runtime dependencies required to assemble Phase 6 services (Memory Search + Hybrid Search).
///
/// Contains all infrastructure components needed to construct domain service implementations
/// at runtime. These dependencies are gathered from the application context and passed to
/// the factory for service assembly.
///
/// # Fields
///
/// * `project_id` - Unique identifier for the current project
/// * `cache` - Shared cache provider for service-level caching
/// * `crypto` - Cryptographic service for secure operations
/// * `config` - Application configuration
/// * `embedding_provider` - Provider for generating vector embeddings
/// * `vector_store_provider` - Provider for vector storage and retrieval
/// * `language_chunker` - Provider for language-aware code chunking
/// * `indexing_ops` - Interface for indexing operations
/// * `event_bus` - Event bus for domain events
/// * `memory_repository` - Repository for memory persistence
/// * `agent_repository` - Repository for agent session data
/// * `vcs_provider` - Version control system provider
/// * `project_service` - Service for project detection and management
pub struct ServiceDependencies {
    /// Unique identifier for the current project
    pub project_id: String,
    /// Shared cache provider for service-level caching
    pub cache: SharedCacheProvider,
    /// Cryptographic service for secure operations
    pub crypto: Arc<dyn CryptoProvider>,
    /// Application configuration
    pub config: AppConfig,
    /// Provider for generating vector embeddings
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    /// Provider for vector storage and retrieval
    pub vector_store_provider: Arc<dyn VectorStoreProvider>,
    /// Provider for language-aware code chunking
    pub language_chunker: Arc<dyn LanguageChunkingProvider>,
    /// Interface for indexing operations
    pub indexing_ops: Arc<dyn IndexingOperationsInterface>,
    /// Event bus for domain events
    pub event_bus: Arc<dyn EventBusProvider>,
    /// Repository for memory persistence
    pub memory_repository: Arc<dyn MemoryRepository>,
    /// Repository for agent session data
    pub agent_repository: Arc<dyn AgentRepository>,
    /// Version control system provider
    pub vcs_provider: Arc<dyn VcsProvider>,
    /// Service for project detection and management
    pub project_service: Arc<dyn ProjectDetectorService>,
    /// Repository for project CRUD
    pub project_repository: Arc<dyn ProjectRepository>,
    /// Repository for VCS entity CRUD
    pub vcs_entity_repository: Arc<dyn VcsEntityRepository>,
    /// Repository for plan entity CRUD
    pub plan_entity_repository: Arc<dyn PlanEntityRepository>,
    /// Repository for issue entity CRUD
    pub issue_entity_repository: Arc<dyn IssueEntityRepository>,
    /// Repository for org entity CRUD
    pub org_entity_repository: Arc<dyn OrgEntityRepository>,
}

/// Domain services factory - creates services with runtime dependencies
pub struct DomainServicesFactory;

impl DomainServicesFactory {
    /// Creates all domain services from runtime dependencies.
    ///
    /// Assembles the complete set of domain service implementations using constructor injection.
    /// This factory method is called at runtime to create services that require dynamic configuration
    /// (embedding provider, vector store, cache).
    ///
    /// # Arguments
    ///
    /// * `deps` - Runtime dependencies including providers, repositories, and configuration
    ///
    /// # Returns
    ///
    /// A `DomainServicesContainer` with all services initialized and ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if service initialization fails.
    pub async fn create_services(deps: ServiceDependencies) -> Result<DomainServicesContainer> {
        let context_service: Arc<dyn ContextServiceInterface> = Arc::new(ContextServiceImpl::new(
            deps.cache.as_provider(),
            Arc::clone(&deps.embedding_provider),
            Arc::clone(&deps.vector_store_provider),
        ));

        let search_service: Arc<dyn SearchServiceInterface> =
            Arc::new(SearchServiceImpl::new(Arc::clone(&context_service)));

        let indexing_service: Arc<dyn IndexingServiceInterface> =
            Arc::new(IndexingServiceImpl::new(
                Arc::clone(&context_service),
                deps.language_chunker,
                deps.indexing_ops,
                deps.event_bus,
            ));

        let validation_service: Arc<dyn ValidationServiceInterface> =
            Arc::new(InfraValidationService::new());

        let memory_service: Arc<dyn MemoryServiceInterface> = Arc::new(MemoryServiceImpl::new(
            deps.project_id.clone(),
            deps.memory_repository,
            deps.embedding_provider,
            deps.vector_store_provider,
        ));

        let agent_session_service: Arc<dyn AgentSessionServiceInterface> =
            Arc::new(AgentSessionServiceImpl::new(deps.agent_repository));

        let resolver = Arc::new(RepositoryResolver::new(Arc::clone(
            &deps.vcs_entity_repository,
        )));

        Ok(DomainServicesContainer {
            context_service,
            search_service,
            indexing_service,
            validation_service,
            memory_service,
            agent_session_service,
            project_service: deps.project_service,
            project_repository: deps.project_repository,
            vcs_provider: deps.vcs_provider,
            vcs_entity_repository: deps.vcs_entity_repository,
            plan_entity_repository: deps.plan_entity_repository,
            issue_entity_repository: deps.issue_entity_repository,
            org_entity_repository: deps.org_entity_repository,
            resolver,
        })
    }

    /// Create indexing service from app context
    pub async fn create_indexing_service(
        app_context: &AppContext,
    ) -> Result<Arc<dyn IndexingServiceInterface>> {
        let indexing_ops = app_context.indexing();
        let event_bus = app_context.event_bus();
        let language_chunker = app_context.language_handle().get();
        let context_service = Self::create_context_service(app_context).await?;

        Ok(Arc::new(IndexingServiceImpl::new(
            context_service,
            language_chunker,
            indexing_ops,
            event_bus,
        )))
    }

    /// Create context service from app context
    pub async fn create_context_service(
        app_context: &AppContext,
    ) -> Result<Arc<dyn ContextServiceInterface>> {
        let cache_provider = app_context.cache_handle().get();
        let embedding_provider = app_context.embedding_handle().get();
        let vector_store_provider = app_context.vector_store_handle().get();

        Ok(Arc::new(ContextServiceImpl::new(
            cache_provider,
            embedding_provider,
            vector_store_provider,
        )))
    }

    /// Create search service from app context
    pub async fn create_search_service(
        app_context: &AppContext,
    ) -> Result<Arc<dyn SearchServiceInterface>> {
        let context_service = Self::create_context_service(app_context).await?;
        Ok(Arc::new(SearchServiceImpl::new(context_service)))
    }
}
