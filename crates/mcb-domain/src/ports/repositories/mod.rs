//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#repository-ports)
//!
//! Repository ports for data persistence.

pub mod agent;
pub mod chunk;
pub mod file_hash;
pub mod issue;
pub mod memory;
pub mod org;
pub mod plan;
pub mod project;
pub mod search;
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
pub use vcs::VcsEntityRepository;
