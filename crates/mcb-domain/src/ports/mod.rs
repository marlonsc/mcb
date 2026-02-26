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
    AgentSessionStats, DailyCount, DashboardQueryPort,
    EmbeddingAdminInterface, IndexingOperation, IndexingOperationStatus,
    IndexingOperationsInterface, LanguageAdminInterface, MonthlyCount, ProviderInfo, ToolCallCount,
    ValidationOperation, ValidationOperationResult, ValidationOperationsInterface,
    ValidationStatus, ValidatorJobRunner, VectorStoreAdminInterface,
};

// --- Infrastructure ---
pub use infrastructure::{
    DependencyHealth, DependencyHealthCheck, DomainEventStream,
    EventBusProvider, ExtendedHealthResponse, LifecycleManaged, LogLevel, OperationLogger,
    PortServiceState, ProviderContext, ProviderHealthStatus, ProviderRouter, SharedSyncCoordinator,
    ShutdownCoordinator, SnapshotProvider, SyncCoordinator, SyncOptions,
    SyncProvider, SyncResult,
};

// --- Providers ---
pub use providers::vector_store::{VectorStoreAdmin, VectorStoreBrowser};
pub use providers::{
    ComplexityAnalyzer, ComplexityFinding,
    CryptoProvider, DeadCodeDetector,
    DeadCodeFinding, EmbeddingProvider, EncryptedData, HttpClientConfig, HttpClientProvider, HybridSearchProvider,
    HybridSearchResult, LanguageChunkingProvider, MetricLabels,
    MetricsError, MetricsProvider, MetricsResult, PROJECT_DETECTORS, ProjectDetector,
    ProjectDetectorConfig, ProjectDetectorEntry, ProviderConfigManagerInterface, RuleValidator,
    RuleValidatorRequest, TdgFinding, TdgScorer, ValidationOptions, ValidationProvider,
    ValidatorInfo, VcsProvider, VectorStoreProvider,
};

// --- Repositories ---
pub use repositories::{
    AgentCheckpointRepository, AgentEventRepository, AgentRepository, AgentSessionQuery,
    AgentSessionRepository, ApiKeyInfo, ApiKeyRegistry, AuthRepositoryPort, FileHashRepository,
    FtsSearchResult, IndexRepository, IndexStats, IssueCommentRegistry, IssueEntityRepository,
    IssueLabelAssignmentManager, IssueLabelRegistry, IssueRegistry, MemoryRepository,
    OrgEntityRepository, OrgRegistry, PlanEntityRepository, PlanRegistry, PlanReviewRegistry,
    PlanVersionRegistry, ProjectRepository, TeamMemberManager, TeamRegistry, TransitionRepository,
    UserRegistry, UserWithApiKey, VcsEntityRepository, WorkflowSessionRepository,
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
