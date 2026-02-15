//! Unified project schema: all persistence entities and relationships.
//!
//! Single source of truth for the whole project.

pub mod agent;
pub mod error_patterns;
/// Module for issue entity schema definitions.
pub mod issue_entities;
pub mod multi_tenant;
/// Module for plan entity schema definitions.
pub mod plan_entities;
/// Module for VCS entity schema definitions.
pub mod vcs_entities;

pub mod types;
pub use types::*;
