//! Domain Port Interfaces
//!
//! Defines all boundary contracts between domain and external layers.
//! Ports are organized by their purpose and enable dependency injection
//! with clear separation of concerns.
//!
//! ## Organization
//!
//! - **providers/** - External service providers (embeddings, vector stores, search)
//! - **infrastructure/** - Infrastructure services (sync, snapshots)
//! - **registry/** - Auto-registration system for plugin providers
//! - **services.rs** - Application service interfaces (context, search, indexing)
//! - **admin.rs** - Administrative interfaces for system management

pub mod admin;
pub mod infrastructure;

// Re-export commonly used port traits for convenience
pub use admin::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, IndexingOperation,
    IndexingOperationsInterface, LifecycleManaged, PerformanceMetricsData,
    PerformanceMetricsInterface, PortServiceState, ShutdownCoordinator,
};
pub use infrastructure::snapshot::SyncProvider;
pub use infrastructure::{
    AuthServiceInterface, DomainEventStream, EventBusProvider, LockGuard, LockProvider,
    ProviderContext, ProviderHealthStatus, ProviderRouter, SharedSyncCoordinator, SnapshotProvider,
    StateStoreProvider, SyncCoordinator, SyncOptions, SyncResult, SystemMetrics,
    SystemMetricsCollectorInterface,
};

// ============================================================================
// Domain Re-exports (Clean Architecture)
// ============================================================================

pub use mcb_domain::ports::providers;
pub use mcb_domain::ports::services;

pub use mcb_domain::registry::{
    CACHE_PROVIDERS, CacheProviderConfig, CacheProviderEntry, EMBEDDING_PROVIDERS,
    EmbeddingProviderConfig, EmbeddingProviderEntry, LANGUAGE_PROVIDERS, LanguageProviderConfig,
    LanguageProviderEntry, VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig,
    VectorStoreProviderEntry, list_cache_providers, list_embedding_providers,
    list_language_providers, list_vector_store_providers, resolve_cache_provider,
    resolve_embedding_provider, resolve_language_provider, resolve_vector_store_provider,
};

pub use mcb_domain::ports::services::{
    AgentSessionServiceInterface, BatchIndexingServiceInterface, ChunkingOrchestratorInterface,
    ComplexityReport, ContextServiceInterface, FileHashService, FunctionComplexity, IndexingResult,
    IndexingServiceInterface, IndexingStats, IndexingStatus, MemoryServiceInterface, RuleInfo,
    SearchFilters, SearchServiceInterface, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};

pub use mcb_domain::ports::providers::{
    CacheProvider, CryptoProvider, EmbeddingProvider, HybridSearchProvider,
    LanguageChunkingProvider, ProjectDetector, VectorStoreProvider,
};
