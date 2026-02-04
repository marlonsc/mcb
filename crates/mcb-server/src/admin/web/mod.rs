//! Admin Web UI Module
//!
//! Provides an HTMX-powered web interface for the admin panel.
//! Templates are embedded at compile time for zero-dependency deployment.
//!
//! ## Pages
//!
//! - `/` or `/ui` - Dashboard with real-time metrics
//! - `/ui/config` - Configuration editor with live reload
//! - `/ui/health` - Health status and dependency monitoring
//! - `/ui/indexing` - Indexing operation progress
//!
//! ## Duplication
//!
//! Nav and footer are duplicated across templates (index, config, health, indexing, browse,
//! browse_collection, browse_file). When changing nav/footer structure, update all of these.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).

pub mod handlers;
pub mod router;

// Re-export public functions
pub use handlers::{config_page, dashboard, dashboard_ui, favicon, health_page, indexing_page};
pub use router::{web_rocket, web_routes};
