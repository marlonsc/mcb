//! Domain Port Interfaces
//!
//! Defines all boundary contracts between domain and external layers.
//! Ports are organized by their purpose and enable dependency injection
//! with clear separation of concerns.
//!
//! Note: Most port definitions have been moved to mcb-domain.
//! This module primarily exists to expose domain ports to the application use cases.

// Re-export ports from domain
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
