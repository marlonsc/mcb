//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Version Control System Providers.

// factory module DELETED — use linkme VCS registry
mod git;
mod submodule;

// default_vcs_provider DELETED — resolve via registry
pub use git::GitProvider;
pub use submodule::{SubmoduleProvider, collect_submodules, collect_submodules_with_depth};
