//! Git-related providers for repository operations.
//!
//! This module provides services for git-aware indexing including:
//! - Git repository operations (open, branch list, commit history, file read)
//! - Project type detection (Cargo, npm, Python, Go, Maven)
//! - Submodule discovery and recursive traversal

mod git2_provider;
pub mod project_detection;
pub mod submodule;

pub use git2_provider::Git2Provider;
pub use project_detection::{detect_all_projects, PROJECT_DETECTORS};
pub use submodule::{collect_submodules, collect_submodules_with_depth, SubmoduleProvider};
