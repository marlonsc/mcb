//! Domain Entities
//!
//! Core business entities that represent the fundamental concepts
//! of the semantic code search domain. Entities have identity and
//! encapsulate business rules.
//!
//! ## Entities
//!
//! | Entity | Description |
//! | -------- | ------------- |
//! | [`CodeChunk`] | Core entity representing a semantically meaningful code segment |
//! | [`CodebaseSnapshot`] | Entity representing a complete codebase state at a point in time |
//! | [`FileSnapshot`] | Entity representing a file's state for change tracking |
//! | [`ProjectType`] | Detected project type with metadata (Cargo, npm, Python, Go, Maven) |
//! | [`SubmoduleInfo`] | VCS submodule information with parent linking |
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Common metadata for domain entities.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct EntityMetadata {
    /// Unique identifier (UUID).
    pub id: String,
    /// Creation timestamp (Unix epoch).
    pub created_at: i64,
    /// Last update timestamp (Unix epoch).
    pub updated_at: i64,
}

/// Trait for entities that have standard metadata.
pub trait BaseEntity {
    /// Returns the entity's unique identifier.
    fn id(&self) -> &str;
    /// Returns the creation timestamp.
    fn created_at(&self) -> i64;
    /// Returns the last update timestamp.
    fn updated_at(&self) -> i64;
}

/// Macro to implement BaseEntity for structs using EntityMetadata
#[macro_export]
macro_rules! impl_base_entity {
    ($t:ty) => {
        impl $crate::entities::BaseEntity for $t {
            fn id(&self) -> &str {
                &self.metadata.id
            }
            fn created_at(&self) -> i64 {
                self.metadata.created_at
            }
            fn updated_at(&self) -> i64 {
                self.metadata.updated_at
            }
        }
    };
}
/// Agent session tracking entities
pub mod agent;
/// API key entities for authentication
pub mod api_key;
/// Core entity representing a semantically meaningful code segment
pub mod code_chunk;
/// Entities for codebase state management and change tracking
pub mod codebase;
/// Issue comments, labels, and label assignments.
pub mod issue;
pub mod memory;
pub mod observation;
/// Organization entity (top-level tenant)
pub mod organization;
/// Plan, plan version, and plan review entities
pub mod plan;
pub mod project;
/// Persisted VCS repository and branch entities (multi-tenant CRUD)
pub mod repository;
/// Submodule entity for VCS submodule tracking
pub mod submodule;
/// Team and team membership entities
pub mod team;
/// User entity
pub mod user;
/// VCS repository, branch, and commit entities
pub mod vcs;
/// Workflow FSM entities for session state management
pub mod workflow;
/// Worktree and agent-worktree assignment entities
pub mod worktree;

// Re-export commonly used entities
pub use agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, CheckpointType, Delegation, ToolCall,
};
pub use api_key::ApiKey;
pub use code_chunk::CodeChunk;
pub use codebase::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};
pub use issue::{IssueComment, IssueLabel, IssueLabelAssignment};
pub use memory::{
    ErrorPattern, ErrorPatternCategory, ErrorPatternMatch, ExecutionMetadata, ExecutionType,
    MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation, ObservationMetadata,
    ObservationType, QualityGateResult, QualityGateStatus, SessionSummary,
};
pub use organization::{OrgStatus, Organization};
pub use plan::{Plan, PlanReview, PlanStatus, PlanVersion, ReviewVerdict};
pub use project::{
    DependencyType, DetectedProject, IssueFilter, IssueStatus, IssueType, PhaseStatus, Project,
    ProjectDecision, ProjectDependency, ProjectIssue, ProjectPhase, ProjectType,
};
pub use repository::{Branch, Repository, VcsType};
pub use submodule::{SubmoduleDiscoveryConfig, SubmoduleInfo};
pub use team::{Team, TeamMember, TeamMemberRole};
pub use user::{User, UserRole};
pub use vcs::{DiffStatus, FileDiff, RefDiff, RepositoryId, VcsBranch, VcsCommit, VcsRepository};
pub use workflow::{Transition, TransitionTrigger, WorkflowSession, WorkflowState};
pub use worktree::{AgentWorktreeAssignment, Worktree, WorktreeStatus};
