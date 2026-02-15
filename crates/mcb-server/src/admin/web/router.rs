//! Web Router Module
//!
//! Router configuration for the admin web interface.

use std::sync::Arc;

use crate::templates::Template;
use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::infrastructure::{DomainEventStream, EventBusProvider};
use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::infrastructure::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use rocket::{Build, Rocket, routes};

use super::entity_handlers;
use super::handlers;
use super::lov_handlers;
use crate::admin::handlers::AdminState;

/// Minimal no-op event bus for the standalone web UI Rocket instance.
///
/// Used by [`web_rocket`] so that Rocket's sentinel check for
/// `Option<&State<AdminState>>` in entity handlers passes without
/// requiring a full production event bus.
struct NullEventBus;

#[async_trait]
impl EventBusProvider for NullEventBus {
    async fn publish_event(&self, _event: DomainEvent) -> Result<()> {
        Ok(())
    }
    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        Ok(Box::pin(futures::stream::empty()))
    }
    fn has_subscribers(&self) -> bool {
        false
    }
    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }
    async fn subscribe(&self, _topic: &str) -> Result<String> {
        Ok(String::new())
    }
}

/// Build a minimal [`AdminState`] with no real service backends.
///
/// Entity handlers degrade gracefully (return empty lists / null records)
/// when the service adapters resolve to `None`.
fn default_admin_state() -> AdminState {
    AdminState {
        metrics: Arc::new(AtomicPerformanceMetrics::new()),
        indexing: Arc::new(DefaultIndexingOperations::new()),
        config_watcher: None,
        #[allow(clippy::expect_used)]
        current_config: ConfigLoader::new()
            .load()
            .expect("startup: configuration file must be loadable"),
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
        event_bus: Arc::new(NullEventBus),
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
        .attach(Template::custom(
            |engines: &mut crate::templates::Engines| {
                crate::admin::web::helpers::register_helpers(&mut engines.handlebars);
            },
        ))
        .mount(
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
