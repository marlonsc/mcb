//! Admin Web UI Module
//!
//! Provides a web interface for the admin panel.
//! Templates are embedded at compile time for zero-dependency deployment.
//! All pages share a unified layout via `shared.js` app-shell injection.
//!
//! ## Pages
//!
//! - `/` or `/ui` - Dashboard with real-time metrics via SSE
//! - `/ui/config` - Configuration editor with live reload
//! - `/ui/health` - Health status and dependency monitoring
//! - `/ui/jobs` - Background jobs monitoring
//! - `/ui/browse` - Browse indexed collections, files, and chunks
//! - `/ui/browse/tree` - Tree view for navigating codebases
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).

pub mod handlers;
pub mod router;

// Re-export public functions
pub use handlers::{config_page, dashboard, dashboard_ui, favicon, health_page, jobs_page};
pub use router::{web_rocket, web_routes};
