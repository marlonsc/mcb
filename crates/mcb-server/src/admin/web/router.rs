//! Web Router Module
//!
//! Router configuration for the admin web interface.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).

use rocket::{Build, Rocket, routes};
use rocket_dyn_templates::Template;

use super::entity_handlers;
use super::handlers;

/// Returns the path to the Handlebars templates directory.
///
/// Searches multiple candidate locations to support both workspace-level
/// execution (cargo test from root) and crate-level execution.
#[must_use]
pub fn template_dir() -> String {
    let candidates = ["crates/mcb-server/templates", "templates"];
    for candidate in &candidates {
        let path = std::path::Path::new(candidate);
        if path.exists() && path.is_dir() {
            tracing::debug!(template_dir = %candidate, "Resolved template directory");
            return (*candidate).to_string();
        }
    }
    tracing::warn!("No template directory found, using default 'templates'");
    "templates".to_string()
}

/// Create the admin web UI rocket instance
///
/// Routes:
/// - GET `/` - Dashboard
/// - GET `/ui` - Dashboard (alias)
/// - GET `/ui/config` - Configuration page
/// - GET `/ui/health` - Health status page
/// - GET `/ui/jobs` - Jobs monitoring page
/// - GET `/ui/browse` - Browse collections page
/// - GET `/ui/browse/<collection>` - Browse collection files page
/// - GET `/ui/browse/<collection>/file` - Browse file chunks page
/// - GET `/ui/browse/tree` - Browse tree view page (Wave 3)
/// - GET `/favicon.ico` - Favicon
pub fn web_rocket() -> Rocket<Build> {
    let figment = rocket::Config::figment().merge(("template_dir", template_dir()));

    rocket::custom(figment).attach(Template::fairing()).mount(
        "/",
        routes![
            handlers::dashboard,
            handlers::dashboard_ui,
            handlers::config_page,
            handlers::health_page,
            handlers::jobs_page,
            handlers::browse_page,
            handlers::browse_collection_page,
            handlers::browse_file_page,
            handlers::browse_tree_page,
            handlers::shared_js,
            handlers::theme_css,
            handlers::favicon,
            entity_handlers::entities_index,
            entity_handlers::entities_list,
            entity_handlers::entities_new_form,
        ],
    )
}

/// Get routes for mounting in a parent Rocket instance
pub fn web_routes() -> Vec<rocket::Route> {
    routes![
        handlers::dashboard,
        handlers::dashboard_ui,
        handlers::config_page,
        handlers::health_page,
        handlers::jobs_page,
        handlers::browse_page,
        handlers::browse_collection_page,
        handlers::browse_file_page,
        handlers::browse_tree_page,
        handlers::shared_js,
        handlers::theme_css,
        handlers::favicon,
        entity_handlers::entities_index,
        entity_handlers::entities_list,
        entity_handlers::entities_new_form,
    ]
}
