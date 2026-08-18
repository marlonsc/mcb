//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#service-ports)
//!
//! Domain service port interfaces for core business operations.

/// Agent session lifecycle management.
pub mod agent_service;
/// Browse and highlight operations.
pub mod browse_service;
/// Code chunking operations.
pub mod chunking_service;
/// Code intelligence / context operations.
pub mod context_service;
/// File hash state management.
pub mod hash_service;
/// Codebase indexing operations.
pub mod indexing_service;
/// Background job lifecycle management.
pub mod job_service;
/// Memory / observation storage and search.
pub mod memory_service;
/// Project detection operations.
pub mod project_service;
/// Semantic code search operations.
pub mod search_service;
/// Architecture validation operations.
pub mod validation_service;

// Re-exports for canonical access via `ports::services::{...}`
pub use agent_service::{
    AgentSessionManager, AgentSessionServiceInterface, CheckpointManager, DelegationTracker,
};
pub use browse_service::{
    BrowseError, BrowseServiceInterface, HighlightError, HighlightServiceInterface,
};
pub use chunking_service::{
    ChunkingOptions, ChunkingOrchestratorInterface, ChunkingResult, CodeChunker,
};
pub use context_service::ContextServiceInterface;
pub use hash_service::FileHashService;
pub use indexing_service::{
    BatchIndexingServiceInterface, IndexingResult, IndexingServiceInterface, IndexingStats,
    IndexingStatus,
};
pub use job_service::{
    Job, JobCounts, JobId, JobManagerInterface, JobProgressUpdate, JobResult, JobStatus, JobType,
};
pub use memory_service::{
    CreateSessionSummaryInput, ErrorPatternManager, MemorySearcher, MemoryServiceInterface,
    ObservationManager, SessionSummaryManager, StoreObservationInput,
};
pub use project_service::ProjectDetectorService;
pub use search_service::{SearchFilters, SearchServiceInterface};
pub use validation_service::{
    ComplexityReport, FunctionComplexity, RuleInfo, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};
