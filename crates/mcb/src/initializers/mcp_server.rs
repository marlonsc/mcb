//! MCP server Loco initializer.
//!
//! Builds and wires the MCP server and `McbState` through Loco's initializer pipeline.
//! All handler state is managed by Loco; no manual bootstrap in Hooks.

use std::sync::Arc;

use async_trait::async_trait;
use axum::Extension;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;

use mcb_domain::registry::ServiceResolutionContext;
use mcb_domain::registry::config::{ConfigProviderConfig, resolve_config_provider};
use mcb_domain::registry::embedding::{EmbeddingProviderConfig, resolve_embedding_provider};
use mcb_domain::registry::vector_store::{
    VectorStoreProviderConfig, resolve_vector_store_provider,
};
use mcb_server::build_mcp_server_bootstrap;
use mcb_server::tools::ExecutionFlow;
use mcb_server::transport::stdio::StdioServerExt;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;

/// Build the embedding provider config from the resolved `AppConfig`.
fn build_embedding_config(
    app_config: &mcb_infrastructure::config::app::AppConfig,
) -> EmbeddingProviderConfig {
    let mut embed_cfg = EmbeddingProviderConfig::new(
        app_config
            .providers
            .embedding
            .provider
            .as_deref()
            .unwrap_or(mcb_utils::constants::DEFAULT_NULL_PROVIDER),
    );
    if let Some(ref v) = app_config.providers.embedding.cache_dir {
        embed_cfg = embed_cfg.with_cache_dir(v.clone());
    }
    if let Some(ref v) = app_config.providers.embedding.model {
        embed_cfg = embed_cfg.with_model(v.clone());
    }
    if let Some(ref v) = app_config.providers.embedding.base_url {
        embed_cfg = embed_cfg.with_base_url(v.clone());
    }
    if let Some(ref v) = app_config.providers.embedding.api_key {
        embed_cfg = embed_cfg.with_api_key(v.clone());
    }
    if let Some(d) = app_config.providers.embedding.dimensions {
        embed_cfg = embed_cfg.with_dimensions(d);
    }
    embed_cfg
}

/// Build the vector store provider config from the resolved `AppConfig`.
fn build_vector_store_config(
    app_config: &mcb_infrastructure::config::app::AppConfig,
) -> VectorStoreProviderConfig {
    let mut vec_cfg = VectorStoreProviderConfig::new(
        app_config
            .providers
            .vector_store
            .provider
            .as_deref()
            .unwrap_or(mcb_utils::constants::DEFAULT_NULL_PROVIDER),
    );
    if let Some(ref v) = app_config.providers.vector_store.address {
        vec_cfg = vec_cfg.with_uri(v.clone());
    }
    if let Some(ref v) = app_config.providers.vector_store.collection {
        vec_cfg = vec_cfg.with_collection(v.clone());
    }
    if let Some(d) = app_config.providers.vector_store.dimensions {
        vec_cfg = vec_cfg.with_dimensions(d);
    }
    vec_cfg
}

/// Public routes — no auth required (static assets + redirect).
fn build_public_routes() -> AxumRouter {
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
fn admin_ui_routes() -> AxumRouter {
    axum::Router::new()
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
}

/// Admin JSON API routes.
fn admin_api_routes() -> AxumRouter {
    axum::Router::new()
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
            "/chunks",
            axum::routing::get(mcb_server::controllers::collections_api::chunks),
        )
        .route(
            "/config",
            axum::routing::get(mcb_server::controllers::admin::config_via_middleware),
        )
}

/// Admin route table (without auth layer applied).
fn admin_route_table() -> AxumRouter {
    admin_ui_routes().merge(admin_api_routes())
}

/// Protected routes — require admin API-key auth.
///
/// Captures `state`/`settings` clones for the admin-auth middleware closure so
/// authorization does not depend on Extension-layer ordering.
fn build_protected_routes(
    state: mcb_server::McbState,
    settings: Option<serde_json::Value>,
) -> AxumRouter {
    let admin_auth_middleware = axum::middleware::from_fn(
        move |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
            let settings = settings.clone();
            let state = state.clone();
            async move {
                if let Err(_e) = mcb_server::auth::authorize_admin_api_key(
                    state.auth_repo.as_ref(),
                    req.headers(),
                    settings.as_ref(),
                )
                .await
                {
                    return Err(axum::http::StatusCode::UNAUTHORIZED);
                }
                Ok(next.run(req).await)
            }
        },
    );

    admin_route_table().layer(admin_auth_middleware)
}

/// Whether the MCP stdio transport should be started.
fn stdio_enabled(mcp: &mcb_infrastructure::config::app::McpConfig) -> bool {
    mcp.stdio_only || !mcp.no_stdio
}

/// Resolve and validate `AppConfig` from Loco settings via the config provider.
fn resolve_app_config(ctx: &AppContext) -> Result<mcb_infrastructure::config::app::AppConfig> {
    let settings = ctx
        .config
        .settings
        .clone()
        .ok_or_else(|| loco_rs::Error::string("missing loco settings for AppConfig"))?;

    // Resolve config provider via CA/DI registry
    let config_provider = resolve_config_provider(&ConfigProviderConfig::new(
        mcb_utils::constants::DEFAULT_CONFIG_PROVIDER,
    ))
    .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

    // Deserialize + validate via resolved provider (production path)
    let app_config_any = config_provider
        .deserialize_from_value(&settings)
        .map_err(|e| loco_rs::Error::string(&format!("AppConfig: {e}")))?;

    let app_config = *app_config_any
        .downcast::<mcb_infrastructure::config::app::AppConfig>()
        .map_err(|_| loco_rs::Error::string("ConfigProvider returned unexpected type"))?;

    Ok(app_config)
}

/// Resolve event bus and provider adapters into a `ServiceResolutionContext`.
fn build_resolution_ctx(
    ctx: &AppContext,
    app_config: mcb_infrastructure::config::app::AppConfig,
) -> Result<ServiceResolutionContext> {
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

    // Resolve providers via mcb-domain registries — no infrastructure helpers
    let embedding_provider = resolve_embedding_provider(&build_embedding_config(&app_config))
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

    let vector_store_provider =
        resolve_vector_store_provider(&build_vector_store_config(&app_config))
            .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

    Ok(ServiceResolutionContext {
        db: Arc::new(ctx.db.clone()),
        config: Arc::new(app_config),
        event_bus,
        embedding_provider,
        vector_store_provider,
    })
}

/// Resolve `AppConfig` from Loco settings and build the MCP server bootstrap.
///
/// Centralizes config-provider deserialization, provider resolution, and the
/// bootstrap wiring so `after_routes` reads as a short orchestration. Returns
/// the bootstrap plus whether the stdio transport should be started.
fn build_bootstrap(ctx: &AppContext) -> Result<(mcb_server::state::McpServerBootstrap, bool)> {
    let app_config = resolve_app_config(ctx)?;

    let execution_flow = if app_config.mcp.stdio_only {
        ExecutionFlow::StdioOnly
    } else {
        ExecutionFlow::ServerHybrid
    };
    let start_stdio = stdio_enabled(&app_config.mcp);

    let resolution_ctx = build_resolution_ctx(ctx, app_config)?;

    let hybrid_search: Arc<dyn mcb_domain::ports::HybridSearchProvider> =
        mcb_domain::registry::hybrid_search::resolve_hybrid_search_provider(
            &mcb_domain::registry::hybrid_search::HybridSearchProviderConfig::new(
                mcb_utils::constants::DEFAULT_HYBRID_SEARCH_PROVIDER,
            ),
        )
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

    let bootstrap = build_mcp_server_bootstrap(
        &resolution_ctx,
        Arc::clone(&resolution_ctx.db),
        Arc::clone(&resolution_ctx.embedding_provider),
        Arc::clone(&resolution_ctx.vector_store_provider),
        hybrid_search,
        execution_flow,
    )
    .map_err(|e| loco_rs::Error::string(&e.to_string()))?;
    Ok((bootstrap, start_stdio))
}

/// Build the HTTP MCP streamable service from the resolved server handle.
fn build_mcp_service(
    mcp_server: Arc<mcb_server::McpServer>,
) -> StreamableHttpService<mcb_server::McpServer, LocalSessionManager> {
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

/// Spawn the MCP stdio server, detaching the task.
fn spawn_stdio_server(mcp_server: Arc<mcb_server::McpServer>) {
    // Detached: handle intentionally dropped so the stdio server runs for the
    // process lifetime. `let _ =` is rejected by clippy::let_underscore_future.
    let _handle = tokio::spawn(async move {
        let server = (*mcp_server).clone();
        if let Err(e) = server.serve_stdio().await {
            mcb_domain::error!("mcp_initializer", "MCP stdio server stopped", &e);
        }
    });
}

/// Loco initializer that builds the MCP server and injects `McbState` into the router.
pub struct McpServerInitializer;

#[async_trait]
impl Initializer for McpServerInitializer {
    fn name(&self) -> String {
        "mcp_server".to_owned()
    }

    async fn after_routes(&self, router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        mcb_domain::infra::logging::set_log_fn(mcb_infrastructure::logging::tracing_log_fn);

        let (bootstrap, start_stdio) = build_bootstrap(ctx)?;

        if start_stdio {
            spawn_stdio_server(Arc::clone(&bootstrap.mcp_server));
        }

        let mcb_state = bootstrap.into_mcb_state();
        ctx.shared_store.insert(mcb_state.clone());

        let mcp_service = build_mcp_service(Arc::clone(&mcb_state.mcp_server));

        let protected_routes =
            build_protected_routes(mcb_state.clone(), ctx.config.settings.clone());

        // Merge public + protected routes, then apply Extension layer so all routes get McbState
        let router = router
            .merge(build_public_routes())
            .merge(protected_routes)
            .layer(Extension(mcb_state));
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
