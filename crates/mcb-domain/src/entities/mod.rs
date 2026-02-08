//! Domain Entities
//!
//! Core business entities that represent the fundamental concepts
//! of the semantic code search domain. Entities have identity and
//! encapsulate business rules.
//!
//! ## Entities
//!
//! | Entity | Description |
//! |--------|-------------|
//! | [`CodeChunk`] | Core entity representing a semantically meaningful code segment |
//! | [`CodebaseSnapshot`] | Entity representing a complete codebase state at a point in time |
//! | [`FileSnapshot`] | Entity representing a file's state for change tracking |
//! | [`ProjectType`] | Detected project type with metadata (Cargo, npm, Python, Go, Maven) |
//! | [`SubmoduleInfo`] | VCS submodule information with parent linking |

/// Agent session tracking entities
pub mod agent;
/// Core entity representing a semantically meaningful code segment
pub mod code_chunk;
/// Entities for codebase state management and change tracking
pub mod codebase;
/// Memory entities for observations and session tracking
pub mod memory;
/// Project type entity for detected projects within repositories
pub mod project;
/// Submodule entity for VCS submodule tracking
pub mod submodule;
/// VCS repository, branch, and commit entities
pub mod vcs;
/// Workflow FSM entities for session state management
pub mod workflow;

// Re-export commonly used entities
pub use agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, CheckpointType, Delegation, ToolCall,
};
pub use code_chunk::CodeChunk;
pub use codebase::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};
pub use memory::{
    ErrorPattern, ErrorPatternCategory, ErrorPatternMatch, ExecutionMetadata, ExecutionType,
    MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation, ObservationMetadata,
    ObservationType, QualityGateResult, QualityGateStatus, SessionSummary,
};
pub use project::{
    DependencyType, DetectedProject, IssueStatus, IssueType, PhaseStatus, Project, ProjectDecision,
    ProjectDependency, ProjectIssue, ProjectPhase, ProjectType,
};
pub use submodule::{SubmoduleDiscoveryConfig, SubmoduleInfo};
pub use vcs::{DiffStatus, FileDiff, RefDiff, RepositoryId, VcsBranch, VcsCommit, VcsRepository};
pub use workflow::{Transition, TransitionTrigger, WorkflowSession, WorkflowState};
