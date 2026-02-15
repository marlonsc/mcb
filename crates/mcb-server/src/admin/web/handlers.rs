//! Web Handlers Module
//!
//! HTTP handlers for the admin web interface.

use crate::templates::Template;
use rocket::State;
use rocket::get;
use rocket::http::ContentType;
use serde::Serialize;

use crate::admin::AdminRegistry;
use crate::admin::crud_adapter::resolve_adapter;
use crate::admin::handlers::AdminState;
use crate::admin::web::view_model::{DashboardEntityCard, nav_groups};

// Static assets remain as compile-time embeds (not Handlebars templates)
const SHARED_JS: &str = include_str!("templates/shared.js");
const THEME_CSS: &str = include_str!("templates/theme.css");

/// Dashboard page handler
#[get("/")]
pub async fn dashboard(state: Option<&State<AdminState>>) -> Template {
    tracing::info!("dashboard called");
    render_dashboard_template("Dashboard", state).await
}

/// Dashboard page handler (alias)
#[get("/ui")]
pub async fn dashboard_ui(state: Option<&State<AdminState>>) -> Template {
    tracing::info!("dashboard_ui called");
    render_dashboard_template("Dashboard", state).await
}

/// Configuration page handler
#[get("/ui/config")]
pub fn config_page() -> Template {
    tracing::info!("config_page called");
    Template::render(
        "admin/config",
        context! {
            title: "Configuration",
            current_page: "config",
            nav_groups: nav_groups(),
        },
    )
}

/// Health status page handler
#[get("/ui/health")]
pub fn health_page() -> Template {
    tracing::info!("health_page called");
    Template::render(
        "admin/health",
        context! {
            title: "Health Status",
            current_page: "health",
            nav_groups: nav_groups(),
        },
    )
}

/// Jobs page handler
#[get("/ui/jobs")]
pub fn jobs_page() -> Template {
    tracing::info!("jobs_page called");
    Template::render(
        "admin/jobs",
        context! {
            title: "Jobs",
            current_page: "jobs",
            nav_groups: nav_groups(),
        },
    )
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
    Template::render(
        "admin/browse",
        context! {
            title: "Browse Indexed Code",
            current_page: "browse",
            nav_groups: nav_groups(),
        },
    )
}

/// Browse collection files page handler
#[get("/ui/browse/<_collection>")]
pub fn browse_collection_page(_collection: &str) -> Template {
    tracing::info!("browse_collection_page called");
    Template::render(
        "admin/browse_collection",
        context! {
            title: "Browse Files",
            current_page: "browse",
            nav_groups: nav_groups(),
        },
    )
}

/// Browse file chunks page handler
#[get("/ui/browse/<_collection>/file")]
pub fn browse_file_page(_collection: &str) -> Template {
    tracing::info!("browse_file_page called");
    Template::render(
        "admin/browse_file",
        context! {
            title: "View Code",
            current_page: "browse",
            nav_groups: nav_groups(),
        },
    )
}

/// Browse tree view page handler (Phase 8b Wave 3)
#[get("/ui/browse/tree")]
pub fn browse_tree_page() -> Template {
    tracing::info!("browse_tree_page called");
    Template::render(
        "admin/browse_tree",
        context! {
            title: "Browse Collection Tree",
            current_page: "browse-tree",
            nav_groups: nav_groups(),
        },
    )
}

#[derive(Debug, Clone, Serialize)]
struct RecentActivityItem {
    entity_title: String,
    record_count: usize,
    timestamp: i64,
}

async fn render_dashboard_template(title: &str, state: Option<&State<AdminState>>) -> Template {
    let mut cards = Vec::<DashboardEntityCard>::new();
    let mut recent_activity = Vec::<RecentActivityItem>::new();
    let now_ts = chrono::Utc::now().timestamp();
    let mut total_records = 0usize;

    for entity in AdminRegistry::all() {
        let record_count = match state.and_then(|s| resolve_adapter(entity.slug, s.inner())) {
            Some(adapter) => match adapter.list_all().await {
                Ok(rows) => rows.len(),
                Err(e) => {
                    tracing::warn!(entity = entity.slug, error = %e, "list_all failed");
                    0
                }
            },
            None => 0,
        };
        total_records += record_count;

        let field_count = entity.fields().iter().filter(|field| !field.hidden).count();
        cards.push(DashboardEntityCard {
            slug: entity.slug.to_owned(),
            title: entity.title.to_owned(),
            group: entity.group.to_owned(),
            field_count,
            record_count,
        });

        if record_count > 0 {
            recent_activity.push(RecentActivityItem {
                entity_title: entity.title.to_owned(),
                record_count,
                timestamp: now_ts,
            });
        }
    }

    recent_activity.truncate(10);
    let entity_count = cards.len();
    let active_entity_count = cards.iter().filter(|card| card.record_count > 0).count();

    Template::render(
        "admin/dashboard",
        context! {
            title: title,
            current_page: "dashboard",
            nav_groups: nav_groups(),
            entity_cards: cards,
            entity_count: entity_count,
            active_entity_count: active_entity_count,
            total_records: total_records,
            recent_activity: recent_activity,
        },
    )
}
