//! Web Router Module
//!
//! Router configuration for the admin web interface.

use rocket::{Build, Rocket, routes};

use super::handlers;

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
    rocket::build().mount(
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
    ]
}
