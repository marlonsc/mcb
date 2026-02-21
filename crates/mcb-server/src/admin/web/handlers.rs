//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Web Handlers Module
//!
//! HTTP handlers for the admin web interface.

use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;
use mcb_domain::warn;
use serde::Serialize;

use crate::admin::AdminRegistry;
use crate::admin::crud_adapter::resolve_adapter;
use crate::admin::handlers::AdminState;
use crate::admin::web::view_model::{DashboardEntityCard, nav_groups};
use crate::templates::Template;

// Static assets remain as compile-time embeds (not Handlebars templates)
const SHARED_JS: &str = include_str!("templates/shared.js");
const THEME_CSS: &str = include_str!("templates/theme.css");

/// Dashboard page handler
pub async fn dashboard(State(state): State<AdminState>) -> Template {
    tracing::info!("dashboard called");
    render_dashboard_template("Dashboard", Some(&state)).await
}

/// Dashboard page handler (alias)
pub async fn dashboard_ui(State(state): State<AdminState>) -> Template {
    tracing::info!("dashboard_ui called");
    render_dashboard_template("Dashboard", Some(&state)).await
}

template_page!(config_page, "admin/config", "Configuration", "config");
template_page!(health_page, "admin/health", "Health Status", "health");
template_page!(jobs_page, "admin/jobs", "Jobs", "jobs");

/// Favicon handler - returns a simple SVG icon
pub async fn favicon() -> impl IntoResponse {
    tracing::info!("favicon called");
    (
        [(header::CONTENT_TYPE, "image/svg+xml")],
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><text y=".9em" font-size="90">📊</text></svg>"#,
    )
}

/// Theme CSS handler
pub async fn theme_css() -> impl IntoResponse {
    tracing::info!("theme_css called");
    ([(header::CONTENT_TYPE, "text/css")], THEME_CSS)
}

/// Shared JavaScript utilities for admin UI
pub async fn shared_js() -> impl IntoResponse {
    tracing::info!("shared_js called");
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        SHARED_JS,
    )
}

template_page!(browse_page, "admin/browse", "Browse Indexed Code", "browse");

template_page_with_path!(
    browse_collection_page,
    collection: String,
    "admin/browse_collection",
    "Browse Files",
    "browse"
);

template_page_with_path!(
    browse_file_page,
    collection: String,
    "admin/browse_file",
    "View Code",
    "browse"
);

template_page!(
    browse_tree_page,
    "admin/browse_tree",
    "Browse Collection Tree",
    "browse-tree"
);

#[derive(Debug, Clone, Serialize)]
struct RecentActivityItem {
    entity_title: String,
    record_count: usize,
    timestamp: i64,
}

async fn render_dashboard_template(title: &str, state: Option<&AdminState>) -> Template {
    let mut cards = Vec::<DashboardEntityCard>::new();
    let mut recent_activity = Vec::<RecentActivityItem>::new();
    let now_ts = chrono::Utc::now().timestamp();
    let mut total_records = 0usize;

    for entity in AdminRegistry::all() {
        let record_count = match state.and_then(|s| resolve_adapter(entity.slug, s)) {
            Some(adapter) => match adapter.list_all().await {
                Ok(rows) => rows.len(),
                Err(e) => {
                    warn!("AdminWeb", "list_all failed", &e);
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
