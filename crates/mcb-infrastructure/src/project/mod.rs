//! Project Infrastructure Module
//!
//! Provides the implementation of `ProjectDetectorService` that
//! delegates to `mcb-providers` git detection features.

/// Git-based project and VCS context resolution.
pub mod context_resolver;
mod service;
pub use service::ProjectService;
