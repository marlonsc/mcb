//! Web Handlers Module
//!
//! HTTP handlers for the admin web interface.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).
//! Migrated from include_str! to Handlebars templates in Wave 4 (Task 14).

use rocket::get;
use rocket::http::ContentType;
use rocket_dyn_templates::{Template, context};

// Static assets remain as compile-time embeds (not Handlebars templates)
const SHARED_JS: &str = include_str!("templates/shared.js");
const THEME_CSS: &str = include_str!("templates/theme.css");

/// Dashboard page handler
#[get("/")]
pub fn dashboard() -> Template {
    tracing::info!("dashboard called");
    Template::render("admin/dashboard", context! { title: "Dashboard" })
}

/// Dashboard page handler (alias)
#[get("/ui")]
pub fn dashboard_ui() -> Template {
    tracing::info!("dashboard_ui called");
    Template::render("admin/dashboard", context! { title: "Dashboard" })
}

/// Configuration page handler
#[get("/ui/config")]
pub fn config_page() -> Template {
    tracing::info!("config_page called");
    Template::render("admin/config", context! { title: "Configuration" })
}

/// Health status page handler
#[get("/ui/health")]
pub fn health_page() -> Template {
    tracing::info!("health_page called");
    Template::render("admin/health", context! { title: "Health Status" })
}

/// Jobs page handler
#[get("/ui/jobs")]
pub fn jobs_page() -> Template {
    tracing::info!("jobs_page called");
    Template::render("admin/jobs", context! { title: "Jobs" })
}

/// Favicon handler - returns a simple SVG icon
#[get("/favicon.ico")]
pub fn favicon() -> (ContentType, &'static str) {
    tracing::info!("favicon called");
    (
        ContentType::SVG,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><text y=".9em" font-size="90">📊</text></svg>"#,
    )
}

/// Theme CSS handler
#[get("/ui/theme.css")]
pub fn theme_css() -> (ContentType, &'static str) {
    tracing::info!("theme_css called");
    (ContentType::CSS, THEME_CSS)
}

/// Shared JavaScript utilities for admin UI
#[get("/ui/shared.js")]
pub fn shared_js() -> (ContentType, &'static str) {
    tracing::info!("shared_js called");
    (ContentType::JavaScript, SHARED_JS)
}

/// Browse collections page handler
#[get("/ui/browse")]
pub fn browse_page() -> Template {
    tracing::info!("browse_page called");
    Template::render("admin/browse", context! { title: "Browse Indexed Code" })
}

/// Browse collection files page handler
#[get("/ui/browse/<_collection>")]
pub fn browse_collection_page(_collection: &str) -> Template {
    tracing::info!("browse_collection_page called");
    Template::render(
        "admin/browse_collection",
        context! { title: "Browse Files" },
    )
}

/// Browse file chunks page handler
#[get("/ui/browse/<_collection>/file")]
pub fn browse_file_page(_collection: &str) -> Template {
    tracing::info!("browse_file_page called");
    Template::render("admin/browse_file", context! { title: "View Code" })
}

/// Browse tree view page handler (Phase 8b Wave 3)
#[get("/ui/browse/tree")]
pub fn browse_tree_page() -> Template {
    tracing::info!("browse_tree_page called");
    Template::render(
        "admin/browse_tree",
        context! { title: "Browse Collection Tree" },
    )
}
