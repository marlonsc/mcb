//! Domain Entities
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!
//! Core business entities representing the main concepts of the MCB domain.
//! Entities have identity and are usually persisted in repositories.

/// Agent session tracking entities
pub mod agent;
pub use agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, CheckpointType, Delegation, ToolCall,
};

/// API key entities for authentication
pub mod api_key;
pub use api_key::ApiKey;

/// Core entity representing a semantically meaningful code segment
pub mod code_chunk;
pub use code_chunk::CodeChunk;

/// Entities for codebase state management and change tracking
pub mod codebase;
pub use codebase::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};

/// Issue-related entities (comments, labels, etc.)
pub mod issue;
pub use issue::{IssueComment, IssueLabel, IssueLabelAssignment};

/// Memory-related entities
pub mod memory;
pub use memory::Observation;

/// Observation entities
pub mod observation;

/// Organization entity (top-level tenant)
pub mod organization;
pub use organization::{OrgStatus, Organization};

/// Plan, plan version, and plan review entities
pub mod plan;
pub use plan::{Plan, PlanReview, PlanStatus, PlanVersion, ReviewVerdict};

/// Project entities
pub mod project;
pub use project::{
    DependencyType, DetectedProject, IssueFilter, IssueStatus, IssueType, PhaseStatus, Project,
    ProjectDecision, ProjectDependency, ProjectIssue, ProjectPhase, ProjectType,
};

/// Persisted VCS repository and branch entities (multi-tenant CRUD)
pub mod repository;
pub use repository::{Branch, Repository, VcsType};

/// Git submodule entities
pub mod submodule;
pub use submodule::{SubmoduleDiscoveryConfig, SubmoduleInfo};

/// Team and team membership entities
pub mod team;
pub use team::{Team, TeamMember, TeamMemberRole};

/// User entity
pub mod user;
pub use user::{User, UserRole};

/// VCS repository, branch, and commit entities
pub mod vcs;
pub use vcs::{VcsBranch, VcsCommit, VcsRepository};

/// Workflow FSM entities for session state management
pub mod workflow;
pub use workflow::{Transition, TransitionTrigger, WorkflowSession, WorkflowState};

/// Git worktree entities
pub mod worktree;
pub use worktree::{AgentWorktreeAssignment, Worktree, WorktreeStatus};
