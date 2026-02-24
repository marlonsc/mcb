//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Version Control System Providers.

pub mod factory;
pub mod git;
pub mod submodule;

pub use crate::project_detection::{PROJECT_DETECTORS, detect_all_projects};
pub use factory::default_vcs_provider;
pub use git::GitProvider;
pub use submodule::{SubmoduleProvider, collect_submodules, collect_submodules_with_depth};
