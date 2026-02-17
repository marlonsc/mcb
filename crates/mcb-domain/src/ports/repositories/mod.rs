//! Repository ports for data persistence.

/// Agent session and event persistence
pub mod agent;
/// Code chunk persistence
pub mod chunk;
/// File hash tracking for incremental indexing
pub mod file_hash;
/// Issue entity management
pub mod issue;
/// Memory observation storage
pub mod memory;
/// Organization entity management
pub mod org;
/// Plan entity management
pub mod plan;
/// Project persistence
pub mod project;
/// Semantic and hybrid search operations
pub mod search;
/// VCS entity management
pub mod vcs;

pub use agent::{
    AgentCheckpointRepository, AgentEventRepository, AgentRepository, AgentSessionQuery,
    AgentSessionRepository,
};
pub use chunk::{ChunkRepository, RepositoryStats};
pub use file_hash::FileHashRepository;
pub use issue::{
    IssueCommentRegistry, IssueEntityRepository, IssueLabelAssignmentManager, IssueLabelRegistry,
    IssueRegistry,
};
pub use memory::{FtsSearchResult, MemoryRepository};
pub use org::{
    ApiKeyRegistry, OrgEntityRepository, OrgRegistry, TeamMemberManager, TeamRegistry, UserRegistry,
};
pub use plan::{PlanEntityRepository, PlanRegistry, PlanReviewRegistry, PlanVersionRegistry};
pub use project::ProjectRepository;
pub use search::{SearchRepository, SearchStats};
pub use vcs::{
    AssignmentManager, BranchRegistry, RepositoryRegistry, VcsEntityRepository, WorktreeManager,
};
