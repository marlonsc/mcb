//! Project Infrastructure Module
//!
//! Provides the implementation of `ProjectDetectorService` that
//! delegates to `mcb-providers` git detection features.

mod service;
pub use service::ProjectService;
