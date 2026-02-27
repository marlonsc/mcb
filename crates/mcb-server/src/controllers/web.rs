//! Web controllers for custom MCB admin HTML pages served at `/ui/*`.

use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;

/// Dashboard page — main admin landing page.
///
/// # Errors
///
/// Fails when dashboard data cannot be loaded.
pub async fn dashboard(Extension(_state): Extension<McbState>) -> Result<Response> {
    // Stub — real implementation in Wave 4 (Task 15)
    format::html("<h1>Dashboard</h1>")
}

/// Configuration page — shows current MCB configuration.
///
/// # Errors
///
/// Fails when config cannot be loaded.
pub async fn config_page(Extension(_state): Extension<McbState>) -> Result<Response> {
    // Stub — real implementation in Wave 4 (Task 15)
    format::html("<h1>Configuration</h1>")
}

/// Health page — shows provider health status.
///
/// # Errors
///
/// Fails when health checks fail.
pub async fn health_page(Extension(_state): Extension<McbState>) -> Result<Response> {
    // Stub — real implementation in Wave 4 (Task 16)
    format::html("<h1>Health</h1><p>Status</p>")
}

/// Jobs page — shows indexing and validation operations.
///
/// # Errors
///
/// Fails when job data cannot be loaded.
pub async fn jobs_page(Extension(_state): Extension<McbState>) -> Result<Response> {
    // Stub — real implementation in Wave 4 (Task 16)
    format::html("<h1>Jobs</h1>")
}

/// Browse page — shows vector store collections.
///
/// # Errors
///
/// Fails when collection data cannot be loaded.
pub async fn browse_page(Extension(_state): Extension<McbState>) -> Result<Response> {
    // Stub — real implementation in Wave 4 (Task 17)
    format::html("<h1>Browse</h1>")
}

/// Custom 404 page.
///
/// # Errors
///
/// Returns a 404 HTML response.
pub async fn not_found_page() -> Result<Response> {
    format::html("<h1>404</h1><p>Not Found</p>")
}

/// Registers web UI routes under `/ui`.
#[must_use]
pub fn routes() -> Routes {
    Routes::new()
        .prefix("ui")
        .add("/", get(dashboard))
        .add("/config", get(config_page))
        .add("/health", get(health_page))
        .add("/jobs", get(jobs_page))
        .add("/browse", get(browse_page))
}
