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
//! | [`SubmoduleInfo`] | Git submodule information with parent linking |

/// Core entity representing a semantically meaningful code segment
pub mod code_chunk;
/// Entities for codebase state management and change tracking
pub mod codebase;
/// Git repository, branch, and commit entities
pub mod git;
/// Memory entities for observations and session tracking
pub mod memory;
/// Project type entity for detected projects within repositories
pub mod project;
/// Submodule entity for git submodule tracking
pub mod submodule;

// Re-export commonly used entities
pub use code_chunk::CodeChunk;
pub use codebase::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};
pub use git::{DiffStatus, FileDiff, GitBranch, GitCommit, GitRepository, RefDiff, RepositoryId};
pub use memory::{
    MemoryFilter, MemorySearchResult, Observation, ObservationMetadata, ObservationType,
    SessionSummary,
};
pub use project::{DetectedProject, ProjectType};
pub use submodule::{SubmoduleDiscoveryConfig, SubmoduleInfo};
