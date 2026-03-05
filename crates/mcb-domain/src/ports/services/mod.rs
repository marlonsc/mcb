//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#service-ports)
//!
//! Domain service port interfaces for core business operations.

/// Agent session lifecycle management.
pub mod agent;
/// Browse and highlight operations.
pub mod browse;
/// Code chunking operations.
pub mod chunking;
/// Code intelligence / context operations.
pub mod context;
/// File hash state management.
pub mod hash;
/// Codebase indexing operations.
pub mod indexing;
/// Background job lifecycle management.
pub mod job;
/// Memory / observation storage and search.
pub mod memory;
/// Project detection operations.
pub mod project;
/// Semantic code search operations.
pub mod search;
/// Architecture validation operations.
pub mod validation_service;

// Re-exports for canonical access via `ports::services::{...}`
pub use agent::{
    AgentSessionManager, AgentSessionServiceInterface, CheckpointManager, DelegationTracker,
};
pub use browse::{BrowseError, BrowseServiceInterface, HighlightError, HighlightServiceInterface};
pub use chunking::{ChunkingOptions, ChunkingOrchestratorInterface, ChunkingResult, CodeChunker};
pub use context::ContextServiceInterface;
pub use hash::FileHashService;
pub use indexing::{
    BatchIndexingServiceInterface, IndexingResult, IndexingServiceInterface, IndexingStats,
    IndexingStatus,
};
pub use job::{
    Job, JobCounts, JobId, JobManagerInterface, JobProgressUpdate, JobResult, JobStatus, JobType,
};
pub use memory::{
    CreateSessionSummaryInput, ErrorPatternManager, MemorySearcher, MemoryServiceInterface,
    ObservationManager, SessionSummaryManager,
};
pub use project::ProjectDetectorService;
pub use search::{SearchFilters, SearchServiceInterface};
pub use validation_service::{
    ComplexityReport, FunctionComplexity, RuleInfo, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};
