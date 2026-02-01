//! Git-related providers for repository operations.
//!
//! This module provides services for git-aware indexing including:
//! - Project type detection (Cargo, npm, Python, Go, Maven)
//! - Submodule discovery and recursive traversal

pub mod project_detection;
pub mod submodule;

pub use project_detection::{PROJECT_DETECTORS, detect_all_projects};
pub use submodule::{SubmoduleService, collect_submodules, collect_submodules_with_depth};
