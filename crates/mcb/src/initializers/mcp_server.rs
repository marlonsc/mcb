//! MCP server Loco initializer.
//!
//! Builds and wires the MCP server and `McbState` through Loco's initializer pipeline.
//! All handler state is managed by Loco; no manual bootstrap in Hooks.

use std::sync::Arc;

use async_trait::async_trait;
use axum::Extension;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;

use mcb_infrastructure::config::{AppConfig, validate_app_config};
use mcb_infrastructure::resolution_context::ServiceResolutionContext;
use mcb_server::build_mcp_server_bootstrap;
use mcb_server::tools::ExecutionFlow;
use mcb_server::transport::stdio::StdioServerExt;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;

/// Loco initializer that builds the MCP server and injects `McbState` into the router.
pub struct McpServerInitializer;

#[async_trait]
impl Initializer for McpServerInitializer {
    fn name(&self) -> String {
        "mcp_server".to_owned()
    }

    async fn after_routes(&self, router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        mcb_domain::infra::logging::set_log_fn(mcb_infrastructure::logging::tracing_log_fn);

        let settings = ctx
            .config
            .settings
            .clone()
            .ok_or_else(|| loco_rs::Error::string("missing loco settings for AppConfig"))?;
        let app_config: AppConfig = serde_json::from_value(settings)
            .map_err(|e| loco_rs::Error::string(&format!("invalid AppConfig settings: {e}")))?;

        validate_app_config(&app_config)
            .map_err(|e| loco_rs::Error::string(&format!("AppConfig validation failed: {e}")))?;

        let event_bus = mcb_domain::registry::events::resolve_event_bus_provider(
            &mcb_domain::registry::events::EventBusProviderConfig::new(
                app_config
                    .system
                    .infrastructure
                    .event_bus
                    .provider
                    .provider_name(),
            ),
        )
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

        let resolution_ctx = ServiceResolutionContext {
            db: ctx.db.clone(),
            config: Arc::new(app_config),
            event_bus,
        };

        let stdio_only = std::env::var("MCB_STDIO_ONLY").is_ok();
        let no_stdio = std::env::var("MCB_NO_STDIO").is_ok();

        let execution_flow = if stdio_only {
            ExecutionFlow::StdioOnly
        } else {
            ExecutionFlow::ServerHybrid
        };

        let bootstrap = build_mcp_server_bootstrap(&resolution_ctx, execution_flow)
            .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

        let mcp_server_for_stdio = Arc::clone(&bootstrap.mcp_server);
        if stdio_only || !no_stdio {
            tokio::spawn(async move {
                let server = (*mcp_server_for_stdio).clone();
                if let Err(e) = server.serve_stdio().await {
                    mcb_domain::error!("mcp_initializer", "MCP stdio server stopped", &e);
                }
            });
        }

        let mcb_state = bootstrap.into_mcb_state();
        ctx.shared_store.insert(mcb_state.clone());

        let ct = CancellationToken::new();
        let mcp_server_for_http = Arc::clone(&mcb_state.mcp_server);

        let mcp_service = StreamableHttpService::new(
            move || {
                let server = (*mcp_server_for_http).clone();
                Ok(server)
            },
            LocalSessionManager::default().into(),
            StreamableHttpServerConfig {
                stateful_mode: false,
                cancellation_token: ct.child_token(),
                ..Default::default()
            },
        );

        // Web UI routes (served at root, not under /api)
        let ui_routes = axum::Router::new()
            .route(
                "/",
                axum::routing::get(|| async { axum::response::Redirect::temporary("/ui/") }),
            )
            .route(
                "/ui",
                axum::routing::get(mcb_server::controllers::web::dashboard),
            )
            .route(
                "/ui/",
                axum::routing::get(mcb_server::controllers::web::dashboard),
            )
            .route(
                "/ui/config",
                axum::routing::get(mcb_server::controllers::web::config_page),
            )
            .route(
                "/ui/health",
                axum::routing::get(mcb_server::controllers::web::health_page),
            )
            .route(
                "/ui/jobs",
                axum::routing::get(mcb_server::controllers::web::jobs_page),
            )
            .route(
                "/ui/browse",
                axum::routing::get(mcb_server::controllers::web::browse_page),
            )
            .route(
                "/health",
                axum::routing::get(mcb_server::controllers::health_api::health),
            )
            .route(
                "/jobs",
                axum::routing::get(mcb_server::controllers::jobs_api::jobs),
            )
            .route(
                "/collections",
                axum::routing::get(mcb_server::controllers::collections_api::collections),
            )
            .route(
                "/config",
                axum::routing::get(mcb_server::controllers::admin::config),
            )
            // Static file routes for assets at root level
            .route(
                "/favicon.ico",
                axum::routing::get(|| async {
                    (
                        [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
                        include_str!("../../../../assets/admin/favicon.ico"),
                    )
                }),
            )
            .route(
                "/ui/theme.css",
                axum::routing::get(|| async {
                    (
                        [(axum::http::header::CONTENT_TYPE, "text/css")],
                        include_str!("../../../../assets/admin/ui/theme.css"),
                    )
                }),
            )
            .route(
                "/ui/shared.js",
                axum::routing::get(|| async {
                    (
                        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
                        include_str!("../../../../assets/admin/ui/shared.js"),
                    )
                }),
            );

        // Merge UI routes first, then apply Extension layer so all routes get McbState
        let router = router.merge(ui_routes).layer(Extension(mcb_state));
        let mcp_routes = axum::Router::new().nest_service("/mcp", mcp_service);

        // 404 fallback handler for unknown routes
        let router = router
            .merge(mcp_routes)
            .fallback(axum::routing::get(|| async {
                (
                    axum::http::StatusCode::NOT_FOUND,
                    axum::response::Html(mcb_server::controllers::web::not_found_html()),
                )
            }));

        Ok(router)
    }
}
