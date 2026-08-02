//! Axum route construction for MCB admin, UI, and MCP endpoints.
//!
//! Provides reusable, framework-agnostic route builders that accept [`McbState`]
//! and produce a complete [`AxumRouter`]. This module is used both by the Loco
//! initializer in `mcb` and by integration tests in `mcb-server` so tests exercise
//! the exact same route/middleware configuration as production.

use std::sync::Arc;

use axum::Router as AxumRouter;
use axum::http::StatusCode;
use axum::response::Html;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;

use crate::McpServer;
use crate::state::McbState;

/// Public routes — no auth required (static assets + redirect).
pub fn build_public_routes() -> AxumRouter {
    axum::Router::new()
        .route(
            "/",
            axum::routing::get(|| async { axum::response::Redirect::temporary("/ui/") }),
        )
        .route(
            "/favicon.ico",
            axum::routing::get(|| async {
                (
                    [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
                    include_str!("../../../../assets/admin/favicon.svg"),
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
        )
}

/// Admin web UI page routes.
pub fn admin_ui_routes() -> AxumRouter {
    axum::Router::new()
        .route("/ui", axum::routing::get(super::web::dashboard))
        .route("/ui/", axum::routing::get(super::web::dashboard))
        .route("/ui/config", axum::routing::get(super::web::config_page))
        .route("/ui/health", axum::routing::get(super::web::health_page))
        .route("/ui/jobs", axum::routing::get(super::web::jobs_page))
        .route("/ui/browse", axum::routing::get(super::web::browse_page))
}

/// Admin JSON API routes.
pub fn admin_api_routes() -> AxumRouter {
    axum::Router::new()
        .route("/health", axum::routing::get(super::health_api::health))
        .route("/jobs", axum::routing::get(super::jobs_api::jobs))
        .route(
            "/collections",
            axum::routing::get(super::collections_api::collections),
        )
        .route(
            "/chunks",
            axum::routing::get(super::collections_api::chunks),
        )
        .route(
            "/config",
            axum::routing::get(super::admin::config_via_middleware),
        )
}

/// Admin route table (without auth layer applied).
pub fn admin_route_table() -> AxumRouter {
    admin_ui_routes().merge(admin_api_routes())
}

/// Protected routes — require admin API-key auth.
///
/// Captures `state`/`settings` clones for the admin-auth middleware closure so
/// authorization does not depend on Extension-layer ordering.
pub fn build_protected_routes(state: McbState, settings: Option<serde_json::Value>) -> AxumRouter {
    let admin_auth_middleware = axum::middleware::from_fn(
        move |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
            let settings = settings.clone();
            let state = state.clone();
            async move {
                if let Err(_e) = crate::auth::authorize_admin_api_key(
                    state.auth_repo.as_ref(),
                    req.headers(),
                    settings.as_ref(),
                )
                .await
                {
                    return Err(StatusCode::UNAUTHORIZED);
                }
                Ok(next.run(req).await)
            }
        },
    );

    admin_route_table().layer(admin_auth_middleware)
}

/// Build the HTTP MCP streamable service from the resolved server handle.
#[must_use]
pub fn build_mcp_service(
    mcp_server: Arc<McpServer>,
) -> StreamableHttpService<McpServer, LocalSessionManager> {
    let ct = CancellationToken::new();
    // rmcp 1.x marks StreamableHttpServerConfig #[non_exhaustive]; build via Default.
    let mut config = StreamableHttpServerConfig::default();
    config.stateful_mode = false;
    config.cancellation_token = ct.child_token();
    StreamableHttpService::new(
        move || {
            let server = (*mcp_server).clone();
            Ok(server)
        },
        LocalSessionManager::default().into(),
        config,
    )
}

/// Build the Axum router that serves the MCP streamable HTTP endpoint.
pub fn build_mcp_router(mcp_server: Arc<McpServer>) -> AxumRouter {
    let mcp_service = build_mcp_service(mcp_server);
    AxumRouter::new().nest_service("/mcp", mcp_service)
}

/// Build the MCB router without a catch-all fallback.
///
/// Use this when the embedding application installs the final unmatched-route
/// response after merging, or intentionally owns all unmatched routes.
pub fn build_router_without_fallback(
    state: McbState,
    mcp_server: Arc<McpServer>,
    settings: Option<serde_json::Value>,
) -> AxumRouter {
    let protected_routes = build_protected_routes(state.clone(), settings);

    AxumRouter::new()
        .merge(build_public_routes())
        .merge(protected_routes)
        .layer(axum::Extension(state))
        .merge(build_mcp_router(mcp_server))
}

/// Build the complete MCB router: public assets, protected admin UI/API, MCP,
/// and a catch-all 404 fallback page.
///
/// This is the canonical standalone route table for integration tests and
/// callers where MCB owns the entire Axum application.
pub fn build_full_router(
    state: McbState,
    mcp_server: Arc<McpServer>,
    settings: Option<serde_json::Value>,
) -> AxumRouter {
    build_router_without_fallback(state, mcp_server, settings).fallback(axum::routing::get(
        || async { (StatusCode::NOT_FOUND, Html(super::web::not_found_html())) },
    ))
}
