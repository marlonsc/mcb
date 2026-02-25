//! Domain Port Interfaces
//!
//! **Documentation**: [`docs/modules/domain.md#port-interfaces-domain-boundaries`](../../../../docs/modules/domain.md#port-interfaces-domain-boundaries)
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
mod admin;
/// Infrastructure service ports
mod infrastructure;
/// External service provider ports
mod providers;
/// Repository ports for data persistence
mod repositories;
/// Application service ports
mod services;

// ============================================================================
// Canonical re-exports — the ONE import surface for all port traits/types.
// Consumers MUST use `use mcb_domain::ports::{...};` only.
// ============================================================================

// --- Admin ---
pub use admin::{
    CacheAdminInterface, EmbeddingAdminInterface, IndexingOperation, IndexingOperationStatus,
    IndexingOperationsInterface, LanguageAdminInterface, ProviderInfo, ValidationOperation,
    ValidationOperationResult, ValidationOperationsInterface, ValidationStatus,
    VectorStoreAdminInterface,
};

// --- Infrastructure ---
pub use infrastructure::{
    AuthServiceInterface, DependencyHealth, DependencyHealthCheck, DomainEventStream,
    EventBusProvider, ExtendedHealthResponse, LifecycleManaged, LogLevel, OperationLogger,
    PortServiceState, ProviderContext, ProviderHealthStatus, ProviderRouter, SharedSyncCoordinator,
    ShutdownCoordinator, SnapshotProvider, StateStoreProvider, SyncCoordinator, SyncOptions,
    SyncProvider, SyncResult, SystemMetrics, SystemMetricsCollectorInterface,
};

// --- Providers ---
pub use providers::vector_store::{VectorStoreAdmin, VectorStoreBrowser};
pub use providers::{
    CacheEntryConfig, CacheProvider, CacheStats, ComplexityAnalyzer, ComplexityFinding,
    CryptoProvider, DEFAULT_CACHE_NAMESPACE, DEFAULT_CACHE_TTL_SECS, DeadCodeDetector,
    DeadCodeFinding, EmbeddingProvider, EncryptedData, FileMetrics, FunctionMetrics,
    HalsteadMetrics, HttpClientConfig, HttpClientProvider, HybridSearchProvider,
    HybridSearchResult, LanguageChunkingProvider, MetricLabels, MetricsAnalysisProvider,
    MetricsError, MetricsProvider, MetricsResult, ProjectDetector, ProjectDetectorConfig,
    ProjectDetectorEntry, ProviderConfigManagerInterface, TdgFinding, TdgScorer, ValidationOptions,
    ValidationProvider, ValidatorInfo, VcsProvider, VectorStoreProvider,
};

// --- Repositories ---
pub use repositories::{
    AgentCheckpointRepository, AgentEventRepository, AgentRepository, AgentSessionQuery,
    AgentSessionRepository, ApiKeyRegistry, FileHashRepository, FtsSearchResult, IndexRepository,
    IndexStats, IssueCommentRegistry, IssueEntityRepository, IssueLabelAssignmentManager,
    IssueLabelRegistry, IssueRegistry, MemoryRepository, OrgEntityRepository, OrgRegistry,
    PlanEntityRepository, PlanRegistry, PlanReviewRegistry, PlanVersionRegistry, ProjectRepository,
    TeamMemberManager, TeamRegistry, TransitionRepository, UserRegistry, VcsEntityRepository,
    WorkflowSessionRepository,
};

// --- Services ---
pub use services::{
    AgentSessionManager, AgentSessionServiceInterface, BatchIndexingServiceInterface, BrowseError,
    BrowseServiceInterface, CheckpointManager, ChunkingOptions, ChunkingOrchestratorInterface,
    ChunkingResult, CodeChunker, ComplexityReport, ContextServiceInterface,
    CreateSessionSummaryInput, DelegationTracker, FileHashService, FunctionComplexity,
    HighlightError, HighlightServiceInterface, IndexingResult, IndexingServiceInterface,
    IndexingStats, IndexingStatus, Job, JobCounts, JobId, JobManagerInterface, JobProgressUpdate,
    JobResult, JobStatus, JobType, MemoryServiceInterface, ProjectDetectorService, RuleInfo,
    SearchFilters, SearchServiceInterface, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};
