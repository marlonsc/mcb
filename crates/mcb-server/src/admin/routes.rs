//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Admin API routes
//!
//! Route definitions for the admin API endpoints.
#![allow(clippy::redundant_type_annotations)]

use std::sync::Arc;

use crate::templates::Template;
use rocket::{Build, Rocket};

use super::auth::AdminAuthConfig;
use super::browse_handlers::{
    BrowseState, get_collection_tree, get_file_chunks, list_collection_files, list_collections,
};
use super::cache::get_cache_stats;
use super::config::handlers::{get_config, reload_config, update_config_section};
use super::control::shutdown;
use super::handlers::AdminState;
use super::health::{
    extended_health_check, get_metrics, health_check, liveness_check, readiness_check,
};
use super::jobs::get_jobs_status;
use super::lifecycle_handlers::{
    list_services, restart_service, services_health, start_service, stop_service,
};
use super::web::entity_handlers::{
    entities_bulk_delete, entities_create, entities_delete, entities_delete_confirm,
    entities_detail, entities_edit_form, entities_index, entities_list, entities_new_form,
    entities_update,
};
use super::web::handlers::{
    browse_collection_page, browse_file_page, browse_page, browse_tree_page, config_page,
    dashboard, dashboard_ui, favicon, health_page, jobs_page, shared_js, theme_css,
};
use super::web::lov_handlers::lov_endpoint;
use super::web::router::template_dir;

/// Create the admin API rocket instance
///
/// Routes:
/// - GET /health - Health check with uptime and status
/// - GET /health/extended - Extended health check with dependency status
/// - GET /metrics - Performance metrics
/// - GET /jobs - Jobs operations status
/// - GET /ready - Kubernetes readiness probe (public)
/// - GET /live - Kubernetes liveness probe (public)
/// - POST /shutdown - Initiate graceful server shutdown (protected)
/// - GET /config - View current configuration (protected)
/// - POST /config/reload - Trigger configuration reload (protected)
/// - PATCH /config/:section - Update configuration section (protected)
/// - GET /events - SSE event stream (served via Axum transport)
/// - GET /services - List registered services (protected)
/// - GET /services/health - Health check all services (protected)
/// - POST /services/:name/start - Start a service (protected)
/// - POST /services/:name/stop - Stop a service (protected)
/// - POST /services/:name/restart - Restart a service (protected)
/// - GET /cache/stats - Cache statistics (protected)
/// - GET /collections - List indexed collections (protected)
/// - GET /collections/:name/files - List files in collection (protected)
/// - GET /collections/:name/files/*path/chunks - Get file chunks (protected)
///
/// # Authentication
///
/// Protected endpoints require the `X-Admin-Key` header (or configured header name)
/// with a valid API key. Public endpoints (health probes) are exempt.
#[must_use]
pub fn admin_rocket(
    state: AdminState,
    auth_config: Arc<AdminAuthConfig>,
    browse_state: Option<BrowseState>,
) -> Rocket<Build> {
    let figment = rocket::Config::figment().merge(("template_dir", template_dir()));

    let mut rocket = rocket::custom(figment)
        .manage(state)
        .manage(auth_config)
        .attach(Template::custom(|engines| {
            crate::utils::handlebars::register_helpers(&mut engines.handlebars);
        }));

    rocket = rocket
        .mount(
            "/",
            rocket::routes![
                health_check,
                extended_health_check,
                get_metrics,
                get_jobs_status,
                readiness_check,
                liveness_check,
            ],
        )
        .mount("/", rocket::routes![shutdown])
        .mount(
            "/",
            rocket::routes![get_config, reload_config, update_config_section],
        )
        // SSE events_stream is now served via Axum transport (see axum_http.rs)
        .mount(
            "/",
            rocket::routes![
                list_services,
                services_health,
                start_service,
                stop_service,
                restart_service,
            ],
        )
        .mount("/", rocket::routes![get_cache_stats])
        .mount(
            "/",
            rocket::routes![
                dashboard,
                dashboard_ui,
                favicon,
                config_page,
                health_page,
                jobs_page,
                browse_page,
                browse_collection_page,
                browse_file_page,
                browse_tree_page,
                theme_css,
                shared_js,
            ],
        )
        .mount(
            "/",
            rocket::routes![
                entities_index,
                entities_list,
                entities_new_form,
                entities_detail,
                entities_edit_form,
                entities_delete_confirm,
                entities_create,
                entities_update,
                entities_delete,
                entities_bulk_delete,
                lov_endpoint,
            ],
        );

    // Add browse routes only if BrowseState is available
    if let Some(browse) = browse_state {
        rocket = rocket.manage(browse).mount(
            "/",
            rocket::routes![
                list_collections,
                list_collection_files,
                get_file_chunks,
                get_collection_tree,
            ],
        );
    }

    rocket
}
