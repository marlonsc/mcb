//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#service-ports)
//!
//! Domain service port interfaces for core business operations.

pub mod agent;
pub mod browse;
pub mod chunking;
pub mod context;
pub mod hash;
pub mod indexing;
pub mod jobs;
pub mod memory;
pub mod project;
pub mod search;
pub mod validation;

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
pub use jobs::{
    Job, JobCounts, JobId, JobManagerInterface, JobProgressUpdate, JobResult, JobStatus, JobType,
};
pub use memory::{CreateSessionSummaryInput, MemoryServiceInterface};
pub use project::ProjectDetectorService;
pub use search::{SearchFilters, SearchServiceInterface};
pub use validation::{
    ComplexityReport, FunctionComplexity, RuleInfo, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};
