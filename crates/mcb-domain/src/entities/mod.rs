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

/// Agent session tracking entities
pub mod agent;
/// API key entities for authentication
pub mod api_key;
/// Common metadata for domain entities
pub mod base;
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
pub use base::{BaseEntity, EntityMetadata};
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
