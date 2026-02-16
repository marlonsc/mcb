//! Git-related providers for repository operations.
//!
//! This module provides services for git-aware indexing including:
//! - Git repository operations (open, branch list, commit history, file read)
//! - Project type detection (Cargo, npm, Python, Go, Maven)
//! - Submodule discovery and recursive traversal

use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;

pub mod git2_provider;
pub mod submodule;

pub use crate::project_detection::{PROJECT_DETECTORS, detect_all_projects};
pub use git2_provider::Git2Provider;
pub use submodule::{SubmoduleProvider, collect_submodules, collect_submodules_with_depth};

/// Builds the default VCS provider implementation.
#[must_use]
pub fn default_vcs_provider() -> Arc<dyn VcsProvider> {
    Arc::new(git2_provider::Git2Provider::new())
}
