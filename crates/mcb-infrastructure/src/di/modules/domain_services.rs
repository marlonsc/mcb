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

use crate::cache::provider::SharedCacheProvider;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
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
use mcb_domain::ports::repositories::{AgentRepository, MemoryRepository, ProjectRepository};
use mcb_domain::ports::services::{
    AgentSessionServiceInterface, ContextServiceInterface, IndexingServiceInterface,
    MemoryServiceInterface, SearchServiceInterface, ValidationServiceInterface,
};
use std::sync::Arc;

use super::super::bootstrap::AppContext;

// Use infrastructure validation service
use crate::validation::InfraValidationService;

/// Domain services container
#[derive(Clone)]
pub struct DomainServicesContainer {
    pub context_service: Arc<dyn ContextServiceInterface>,
    pub search_service: Arc<dyn SearchServiceInterface>,
    pub indexing_service: Arc<dyn IndexingServiceInterface>,
    pub validation_service: Arc<dyn ValidationServiceInterface>,
    pub memory_service: Arc<dyn MemoryServiceInterface>,
    pub agent_session_service: Arc<dyn AgentSessionServiceInterface>,
    pub vcs_provider: Arc<dyn VcsProvider>,
}

/// Runtime dependencies required to assemble Phase 6 services (Memory Search + Hybrid Search).
///
/// Contains providers, repositories, caches, and event buses that map directly to the Phase 6 pipeline
/// described in `.planning/STATE.md` and `docs/context/project-state.md` so injecting the right
/// combination keeps the memory/indexing services aligned with the roadmap.
pub struct ServiceDependencies {
    pub project_id: String,
    pub cache: SharedCacheProvider,
    pub crypto: CryptoService,
    pub config: AppConfig,
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    pub vector_store_provider: Arc<dyn VectorStoreProvider>,
    pub language_chunker: Arc<dyn LanguageChunkingProvider>,
    pub indexing_ops: Arc<dyn IndexingOperationsInterface>,
    pub event_bus: Arc<dyn EventBusProvider>,
    pub memory_repository: Arc<dyn MemoryRepository>,
    pub agent_repository: Arc<dyn AgentRepository>,
    pub project_repository: Arc<dyn ProjectRepository>,
    pub vcs_provider: Arc<dyn VcsProvider>,
}

/// Domain services factory - creates services with runtime dependencies
pub struct DomainServicesFactory;

impl DomainServicesFactory {
    pub async fn create_services(deps: ServiceDependencies) -> Result<DomainServicesContainer> {
        let context_service: Arc<dyn ContextServiceInterface> = Arc::new(ContextServiceImpl::new(
            deps.cache.into(),
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

        Ok(DomainServicesContainer {
            context_service,
            search_service,
            indexing_service,
            validation_service,
            memory_service,
            agent_session_service,
            vcs_provider: deps.vcs_provider,
        })
    }

    /// Create indexing service from app context
    pub async fn create_indexing_service(
        app_context: &AppContext,
    ) -> Result<Arc<dyn IndexingServiceInterface>> {
        // Get providers from handles (runtime-swappable)
        let language_chunker = app_context.language_handle().get();

        // Create context service first (dependency)
        let context_service = Self::create_context_service(app_context).await?;

        // Get indexing operations tracker and event bus from context
        let indexing_ops = app_context.indexing();
        let event_bus = app_context.event_bus();

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
        // Get providers from handles (runtime-swappable)
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
        // Create context service first (dependency)
        let context_service = Self::create_context_service(app_context).await?;

        Ok(Arc::new(SearchServiceImpl::new(context_service)))
    }
}
