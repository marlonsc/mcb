//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Admin Web UI Module
//!
//! Provides a web interface for the admin panel.
//!
//! ## Pages
//!
//! - `/` or `/ui` - Dashboard with real-time metrics via SSE
//! - `/ui/config` - Configuration editor with live reload
//! - `/ui/health` - Health status and dependency monitoring
//! - `/ui/jobs` - Background jobs monitoring
//! - `/ui/browse` - Browse indexed collections, files, and chunks
//! - `/ui/browse/tree` - Tree view for navigating codebases

/// Schema-driven entity CRUD handlers using Handlebars templates.
pub mod entity_handlers;
pub mod filter;
pub mod handlers;
/// List-of-Values handlers for FK dropdown population.
pub mod lov_handlers;
pub mod pipeline;
pub mod router;
pub mod view_model;

// Re-export public functions
pub use handlers::{config_page, dashboard, dashboard_ui, favicon, health_page, jobs_page};
pub use router::web_rocket;
