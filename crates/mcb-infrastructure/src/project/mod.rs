//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Project Infrastructure Module
//!
//! Provides the implementation of `ProjectDetectorService` that
//! delegates to an injected detection function via DI.

/// Git-based project and VCS context resolution.
pub mod context_resolver;
/// Service implementation details.
mod service;
/// Workspace structure exploration and crate discovery utilities.
pub mod workspace;

pub use service::{DetectAllFn, ProjectService};
pub use workspace::WorkspaceExplorer;
