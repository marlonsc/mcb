//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Project Infrastructure Module
//!
//! Provides the implementation of `ProjectDetectorService` that
//! delegates to `mcb-providers` git detection features.

/// Git-based project and VCS context resolution.
pub mod context_resolver;
/// Service implementation details.
mod service;
/// Workspace structure exploration and crate discovery utilities.
pub mod workspace;

pub use service::ProjectService;
pub use workspace::WorkspaceExplorer;
