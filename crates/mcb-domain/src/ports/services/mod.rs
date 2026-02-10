//! Domain service port interfaces for core business operations.

pub mod agent;
pub mod chunking;
pub mod context;
pub mod hash;
pub mod indexing;
pub mod memory;
pub mod project;
pub mod search;
pub mod validation;
pub mod vcs_entity;

pub use agent::AgentSessionServiceInterface;
pub use chunking::{ChunkingOptions, ChunkingOrchestratorInterface, ChunkingResult, CodeChunker};
pub use context::ContextServiceInterface;
pub use hash::FileHashService;
pub use indexing::{
    BatchIndexingServiceInterface, IndexingResult, IndexingServiceInterface, IndexingStats,
    IndexingStatus,
};
pub use memory::MemoryServiceInterface;
pub use project::{ProjectDetectorService, ProjectServiceInterface};
pub use search::{SearchFilters, SearchServiceInterface};
pub use validation::{
    ComplexityReport, FunctionComplexity, RuleInfo, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};
pub use vcs_entity::VcsEntityServiceInterface;
