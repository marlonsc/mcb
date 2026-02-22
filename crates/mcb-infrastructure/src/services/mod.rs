//! Infrastructure Services
//!
//! **Documentation**: [`docs/modules/infrastructure.md`](../../../../docs/modules/infrastructure.md)
//!
//! Domain service implementations provided by the infrastructure layer.

/// Syntax highlighting renderer.
pub mod highlight_renderer;
pub mod highlight_service;

pub use highlight_service::HighlightServiceImpl;
