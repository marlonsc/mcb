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

pub use agent::{AgentRepository, AgentSessionQuery};
pub use chunk::{ChunkRepository, RepositoryStats};
pub use file_hash::FileHashRepository;
pub use issue::IssueEntityRepository;
pub use memory::{FtsSearchResult, MemoryRepository};
pub use org::OrgEntityRepository;
pub use plan::PlanEntityRepository;
pub use project::ProjectRepository;
pub use search::{SearchRepository, SearchStats};
pub use vcs::VcsEntityRepository;
