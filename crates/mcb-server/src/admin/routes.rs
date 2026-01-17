//! Admin API routes
//!
//! Route definitions for the admin API endpoints.

use axum::{
    routing::{get, patch, post},
    Router,
};

use super::config_handlers::{get_config, reload_config, update_config_section};
use super::handlers::{
    extended_health_check, get_indexing_status, get_metrics, health_check, liveness_check,
    readiness_check, shutdown, AdminState,
};
use super::lifecycle_handlers::{
    list_services, restart_service, services_health, start_service, stop_service,
};
use super::sse::events_stream;

/// Create the admin API router
///
/// Routes:
/// - GET /health - Health check with uptime and status
/// - GET /health/extended - Extended health check with dependency status
/// - GET /metrics - Performance metrics
/// - GET /indexing - Indexing operations status
/// - GET /ready - Kubernetes readiness probe
/// - GET /live - Kubernetes liveness probe
/// - POST /shutdown - Initiate graceful server shutdown
/// - GET /config - View current configuration (sanitized)
/// - POST /config/reload - Trigger configuration reload
/// - PATCH /config/:section - Update configuration section
/// - GET /events - SSE event stream for real-time updates
/// - GET /services - List registered services
/// - GET /services/health - Health check all services
/// - POST /services/:name/start - Start a service
/// - POST /services/:name/stop - Stop a service
/// - POST /services/:name/restart - Restart a service
pub fn admin_router(state: AdminState) -> Router {
    Router::new()
        // Health and monitoring
        .route("/health", get(health_check))
        .route("/health/extended", get(extended_health_check))
        .route("/metrics", get(get_metrics))
        .route("/indexing", get(get_indexing_status))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
        // Service control
        .route("/shutdown", post(shutdown))
        // Configuration management
        .route("/config", get(get_config))
        .route("/config/reload", post(reload_config))
        .route("/config/{section}", patch(update_config_section))
        // SSE event stream
        .route("/events", get(events_stream))
        // Service lifecycle management
        .route("/services", get(list_services))
        .route("/services/health", get(services_health))
        .route("/services/{name}/start", post(start_service))
        .route("/services/{name}/stop", post(stop_service))
        .route("/services/{name}/restart", post(restart_service))
        .with_state(state)
}
