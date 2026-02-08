//! Domain Port Interfaces
//!
//! Defines all boundary contracts between domain and external layers.
//! Ports are organized by their purpose and enable dependency injection
//! with clear separation of concerns.
//!
//! ## Architecture
//!
//! Ports define the contracts that external layers must implement.
//! This follows the Dependency Inversion Principle:
//! - High-level modules (domain) define interfaces
//! - Low-level modules (providers, infrastructure) implement them
//!
//! ## Organization
//!
//! - **admin** - Administrative interfaces for system management and monitoring
//! - **infrastructure/** - Infrastructure services (sync, snapshots, auth, events)
//! - **providers/** - External service provider ports (embeddings, vector stores, search)
//! - **services** - Application service ports (validation, etc.)

/// Administrative interfaces for system management and monitoring
pub mod admin;
/// Browse and highlight service ports
pub mod browse;
/// Infrastructure service ports
pub mod infrastructure;
/// External service provider ports
pub mod providers;
/// Repository ports for data persistence
pub mod repositories;
/// Application service ports
pub mod services;

// Re-export commonly used port traits for convenience
pub use admin::{
    DependencyHealthCheck, ExtendedHealthResponse, IndexingOperation, IndexingOperationsInterface,
    LifecycleManaged, PerformanceMetricsData, PerformanceMetricsInterface, PortServiceState,
    ShutdownCoordinator, ValidationOperation, ValidationOperationResult,
    ValidationOperationsInterface,
};
pub use browse::{BrowseError, BrowseServiceInterface, HighlightError, HighlightServiceInterface};
pub use infrastructure::{
    AuthServiceInterface, DatabaseExecutor, DomainEventStream, EventBusProvider, LockGuard,
    LockProvider, ProviderContext, ProviderHealthStatus, ProviderRouter, SharedSyncCoordinator,
    SnapshotProvider, SqlParam, SqlRow, StateStoreProvider, SyncCoordinator, SyncOptions,
    SyncProvider, SyncResult, SystemMetrics, SystemMetricsCollectorInterface,
};
pub use providers::{
    CacheEntryConfig, CacheProvider, CacheProviderFactoryInterface, CacheStats, CryptoProvider,
    EmbeddingProvider, EncryptedData, FileMetrics, FunctionMetrics, HalsteadMetrics,
    HybridSearchProvider, HybridSearchResult, LanguageChunkingProvider, MetricsAnalysisProvider,
    ProviderConfigManagerInterface, ValidationOptions, ValidationProvider, ValidatorInfo,
    VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider,
};
pub use repositories::{AgentRepository, MemoryRepository};
pub use services::{ValidationReport, ValidationServiceInterface, ViolationEntry};
