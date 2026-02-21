//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Web Router Module
//!
//! Router configuration for the admin web interface.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use mcb_infrastructure::infrastructure::{
    AtomicPerformanceMetrics, DefaultIndexingOperations, default_event_bus,
};

use super::entity_handlers;
use super::handlers;
use super::lov_handlers;
use crate::admin::handlers::AdminState;
use crate::constants::limits::DEFAULT_SHUTDOWN_TIMEOUT_SECS;
use crate::templates::init_axum_context;
use crate::utils::config::load_startup_config_or_default;

fn default_admin_state() -> AdminState {
    AdminState {
        metrics: Arc::new(AtomicPerformanceMetrics::new()),
        indexing: Arc::new(DefaultIndexingOperations::new()),
        config_watcher: None,
        current_config: load_startup_config_or_default(),
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: DEFAULT_SHUTDOWN_TIMEOUT_SECS,
        event_bus: default_event_bus(),
        service_manager: None,
        cache: None,
        project_workflow: None,
        vcs_entity: None,
        plan_entity: None,
        issue_entity: None,
        org_entity: None,
        tool_handlers: None,
    }
}

/// Resolves the template directory path (disk or fallback for embedded).
#[must_use]
pub fn template_dir() -> String {
    const MANIFEST_TEMPLATE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates");
    let candidates = [
        MANIFEST_TEMPLATE_DIR,
        "crates/mcb-server/templates",
        "templates",
    ];
    for candidate in &candidates {
        let path = std::path::Path::new(candidate);
        if path.exists() && path.is_dir() {
            tracing::debug!(template_dir = %candidate, "Resolved template directory");
            return (*candidate).to_owned();
        }
    }
    tracing::info!("No template directory found on disk, embedded templates will be used");
    "templates".to_owned()
}

/// Builds the admin web UI router with the given state (dashboard, /ui/*, entities, LOV).
#[must_use]
pub fn web_router_with_state(state: AdminState) -> Router {
    init_axum_context(&template_dir(), |engines| {
        crate::utils::handlebars::register_helpers(&mut engines.handlebars);
    });

    Router::new()
        .route("/", get(handlers::dashboard))
        .route("/ui", get(handlers::dashboard_ui))
        .route("/ui/config", get(handlers::config_page))
        .route("/ui/health", get(handlers::health_page))
        .route("/ui/jobs", get(handlers::jobs_page))
        .route("/ui/browse", get(handlers::browse_page))
        .route("/ui/browse/tree", get(handlers::browse_tree_page))
        .route(
            "/ui/browse/{collection}",
            get(handlers::browse_collection_page),
        )
        .route(
            "/ui/browse/{collection}/file",
            get(handlers::browse_file_page),
        )
        .route("/favicon.ico", get(handlers::favicon))
        .route("/ui/theme.css", get(handlers::theme_css))
        .route("/ui/shared.js", get(handlers::shared_js))
        .route("/ui/entities", get(entity_handlers::entities_index))
        .route(
            "/ui/entities/{slug}",
            get(entity_handlers::entities_list).post(entity_handlers::entities_create),
        )
        .route(
            "/ui/entities/{slug}/new",
            get(entity_handlers::entities_new_form),
        )
        .route(
            "/ui/entities/{slug}/bulk-delete",
            post(entity_handlers::entities_bulk_delete),
        )
        .route(
            "/ui/entities/{slug}/{id}",
            get(entity_handlers::entities_detail).post(entity_handlers::entities_update),
        )
        .route(
            "/ui/entities/{slug}/{id}/edit",
            get(entity_handlers::entities_edit_form),
        )
        .route(
            "/ui/entities/{slug}/{id}/delete",
            get(entity_handlers::entities_delete_confirm).post(entity_handlers::entities_delete),
        )
        .route("/ui/lov/{entity_slug}", get(lov_handlers::lov_endpoint))
        .with_state(state)
}

/// Default admin web UI router with default state (for tests and standalone use).
#[must_use]
pub fn web_router() -> Router {
    web_router_with_state(default_admin_state())
}
