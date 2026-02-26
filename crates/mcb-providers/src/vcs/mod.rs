//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Version Control System Providers.

// factory module DELETED — use linkme VCS registry
pub mod git;
pub mod submodule;

pub use crate::project_detection::detect_all_projects;
// default_vcs_provider DELETED — resolve via registry
pub use git::GitProvider;
pub use submodule::{SubmoduleProvider, collect_submodules, collect_submodules_with_depth};
