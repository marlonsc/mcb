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
/// Generic background job management ports
pub mod jobs;
/// External service provider ports
pub mod providers;
/// Repository ports for data persistence
pub mod repositories;
/// Application service ports
pub mod services;

// ============================================================================
// Canonical re-exports — the ONE import surface for all port traits/types.
// Consumers MUST use `use mcb_domain::ports::{...};` only.
// ============================================================================

// --- Admin ---
pub use admin::{
    CacheAdminInterface, DependencyHealth, DependencyHealthCheck, EmbeddingAdminInterface,
    ExtendedHealthResponse, IndexingOperation, IndexingOperationStatus,
    IndexingOperationsInterface, LanguageAdminInterface, LifecycleManaged, PerformanceMetricsData,
    PerformanceMetricsInterface, PortServiceState, ProviderInfo, ShutdownCoordinator,
    ValidationOperation, ValidationOperationResult, ValidationOperationsInterface,
    ValidationStatus, VectorStoreAdminInterface,
};

// --- Browse ---
pub use browse::{BrowseError, BrowseServiceInterface, HighlightError, HighlightServiceInterface};

// --- Infrastructure ---
pub use infrastructure::{
    AuthServiceInterface, DatabaseExecutor, DatabaseProvider, DomainEventStream, EventBusProvider,
    ProviderContext, ProviderHealthStatus, ProviderRouter, SharedSyncCoordinator, SnapshotProvider,
    SqlParam, SqlRow, StateStoreProvider, SyncCoordinator, SyncOptions, SyncProvider, SyncResult,
    SystemMetrics, SystemMetricsCollectorInterface,
};

// --- Jobs ---
pub use jobs::{
    Job, JobCounts, JobId, JobManagerInterface, JobProgressUpdate, JobResult, JobStatus, JobType,
};

// --- Providers ---
pub use providers::{
    CacheEntryConfig, CacheProvider, CacheProviderFactoryInterface, CacheStats, ComplexityAnalyzer,
    ComplexityFinding, CryptoProvider, DeadCodeDetector, DeadCodeFinding, EmbeddingProvider,
    EncryptedData, FileMetrics, FunctionMetrics, HalsteadMetrics, HybridSearchProvider,
    HybridSearchResult, LanguageChunkingProvider, MetricLabels, MetricsAnalysisProvider,
    MetricsProvider, ProjectDetector, ProjectDetectorConfig, ProjectDetectorEntry,
    ProviderConfigManagerInterface, TdgFinding, TdgScorer, ValidationOptions, ValidationProvider,
    ValidatorInfo, VcsProvider, VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider,
};

// --- Repositories ---
pub use repositories::{
    AgentCheckpointRepository, AgentEventRepository, AgentRepository, AgentSessionQuery,
    AgentSessionRepository, ApiKeyRegistry, AssignmentManager, BranchRegistry, ChunkRepository,
    FileHashRepository, FtsSearchResult, IssueCommentRegistry, IssueEntityRepository,
    IssueLabelAssignmentManager, IssueLabelRegistry, IssueRegistry, MemoryRepository,
    OrgEntityRepository, OrgRegistry, PlanEntityRepository, PlanRegistry, PlanReviewRegistry,
    PlanVersionRegistry, ProjectRepository, RepositoryRegistry, RepositoryStats, SearchRepository,
    SearchStats, TeamMemberManager, TeamRegistry, UserRegistry, VcsEntityRepository,
    WorktreeManager,
};

// --- Services ---
pub use services::{
    AgentSessionManager, AgentSessionServiceInterface, CheckpointManager, ChunkingOptions,
    ChunkingResult, CodeChunker, ComplexityReport, ContextServiceInterface,
    CreateSessionSummaryInput, DelegationTracker, FunctionComplexity, IndexingResult,
    IndexingServiceInterface, IndexingStatus, MemoryServiceInterface, ProjectDetectorService,
    RuleInfo, SearchFilters, SearchServiceInterface, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};
