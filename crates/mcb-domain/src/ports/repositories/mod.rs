#![allow(unused_imports)]

//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#repository-ports)
//!
//! Repository ports for data persistence.

pub mod agent;
pub mod auth;
pub mod file_hash;
pub mod index;
pub mod issue;
pub mod memory;
pub mod org;
pub mod plan;
pub mod project;
pub mod vcs;
pub mod workflow;

pub use agent::{
    AgentCheckpointRepository, AgentEventRepository, AgentRepository, AgentSessionQuery,
    AgentSessionRepository,
};
pub use auth::{ApiKeyInfo, AuthRepositoryPort, UserWithApiKey};
pub use file_hash::FileHashRepository;
pub use index::{IndexRepository, IndexStats};
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
pub use vcs::VcsEntityRepository;
pub use workflow::{TransitionRepository, WorkflowSessionRepository};
