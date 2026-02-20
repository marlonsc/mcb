//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Web Router Module
//!
//! Router configuration for the admin web interface.
#![allow(clippy::redundant_type_annotations)]

use std::sync::Arc;

use crate::templates::Template;
use mcb_infrastructure::infrastructure::{
    AtomicPerformanceMetrics, DefaultIndexingOperations, default_event_bus,
};
use rocket::{Build, Rocket};

use super::entity_handlers;
use super::handlers;
use super::lov_handlers;
use crate::admin::handlers::AdminState;
use crate::constants::limits::DEFAULT_SHUTDOWN_TIMEOUT_SECS;
use crate::utils::config::load_startup_config_or_default;

/// Build a minimal [`AdminState`] with no real service backends.
///
/// Entity handlers degrade gracefully (return empty lists / null records)
/// when the service adapters resolve to `None`.
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

/// Returns the path to the templates directory (Handlebars).
///
/// Searches multiple candidate locations to support both workspace-level
/// execution (cargo test from root) and crate-level execution.
/// Falls back to `"templates"` when no directory is found on disk;
/// the embedded template fallback in `Context::initialize` handles
/// the case where this path does not exist at runtime.
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
/// - GET `/ui/lov/<entity_slug>` - LOV endpoint for FK dropdown selects
/// - GET `/favicon.ico` - Favicon
#[must_use]
pub fn web_rocket() -> Rocket<Build> {
    let figment = rocket::Config::figment().merge(("template_dir", template_dir()));

    rocket::custom(figment)
        .manage(default_admin_state())
        .attach(Template::custom(|engines| {
            crate::utils::handlebars::register_helpers(&mut engines.handlebars);
        }))
        .mount(
            "/",
            rocket::routes![
                handlers::dashboard,
                handlers::dashboard_ui,
                handlers::config_page,
                handlers::health_page,
                handlers::jobs_page,
                handlers::browse_page,
                handlers::browse_collection_page,
                handlers::browse_file_page,
                handlers::browse_tree_page,
            ],
        )
        .mount(
            "/",
            rocket::routes![handlers::shared_js, handlers::theme_css, handlers::favicon],
        )
        .mount(
            "/",
            rocket::routes![
                entity_handlers::entities_index,
                entity_handlers::entities_list,
                entity_handlers::entities_new_form,
                entity_handlers::entities_detail,
                entity_handlers::entities_edit_form,
                entity_handlers::entities_delete_confirm,
                entity_handlers::entities_create,
                entity_handlers::entities_update,
                entity_handlers::entities_delete,
                entity_handlers::entities_bulk_delete,
                lov_handlers::lov_endpoint,
            ],
        )
}
