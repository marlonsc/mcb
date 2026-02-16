//! Version Control System Providers
//!
//! This module contains implementations of the `VcsProvider` trait.
//! Currently supports local Git repositories via `git2`.

/// Factory for creating VCS providers
pub mod factory;
/// Git2 implementation of VcsProvider
pub mod git2_provider;
/// Submodule handling
pub mod submodule;

pub use crate::project_detection::{PROJECT_DETECTORS, detect_all_projects};
pub use factory::default_vcs_provider;
pub use git2_provider::Git2Provider;
pub use submodule::{SubmoduleProvider, collect_submodules, collect_submodules_with_depth};
