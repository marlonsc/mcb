//! MCP server Loco initializer.
//!
//! Builds and wires the MCP server and `McbState` through Loco's initializer pipeline.
//! All handler state is managed by Loco; no manual bootstrap in Hooks.
//!
//! Route construction lives in `mcb_server::axum_routes` so the production route
//! table can be reused verbatim by integration tests without duplicating logic.

use std::sync::Arc;

use async_trait::async_trait;
use axum::{Router as AxumRouter, http::StatusCode, response::Html};
use loco_rs::prelude::*;

use mcb_domain::registry::ServiceResolutionContext;
use mcb_domain::registry::config::{ConfigProviderConfig, resolve_config_provider};
use mcb_domain::registry::embedding::{EmbeddingProviderConfig, resolve_embedding_provider};
use mcb_domain::registry::vector_store::{
    VectorStoreProviderConfig, resolve_vector_store_provider,
};
use mcb_server::tools::ExecutionFlow;
use mcb_server::transport::stdio::StdioServerExt;
use mcb_server::{McpBootstrapProviders, build_mcp_server_bootstrap};

/// Watch receiver that fires when the stdio MCP server shuts down.
///
/// Inserted into the Loco `AppContext` shared store so the `serve` command can
/// await stdio-only shutdown without starting the HTTP server.
#[derive(Clone)]
pub struct StdioServerShutdown(tokio::sync::watch::Receiver<bool>);

impl StdioServerShutdown {
    /// Wait for the stdio server to terminate.
    pub async fn wait(mut self) {
        let _ = self.0.changed().await;
    }
}

/// Build the embedding provider config from the resolved `AppConfig`.
fn build_embedding_config(
    app_config: &mcb_infrastructure::config::app::AppConfig,
) -> Result<EmbeddingProviderConfig, Box<loco_rs::Error>> {
    let provider = app_config
        .providers
        .embedding
        .provider
        .as_deref()
        .ok_or_else(|| {
            Box::new(loco_rs::Error::string(
                "Embedding provider is not configured",
            ))
        })?;
    let mut embed_cfg = EmbeddingProviderConfig::new(provider);
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
    Ok(embed_cfg)
}

/// Build the vector store provider config from the resolved `AppConfig`.
fn build_vector_store_config(
    app_config: &mcb_infrastructure::config::app::AppConfig,
) -> Result<VectorStoreProviderConfig, Box<loco_rs::Error>> {
    let provider = app_config
        .providers
        .vector_store
        .provider
        .as_deref()
        .ok_or_else(|| {
            Box::new(loco_rs::Error::string(
                "Vector store provider is not configured",
            ))
        })?;
    let mut vec_cfg = VectorStoreProviderConfig::new(provider);
    if let Some(ref v) = app_config.providers.vector_store.address {
        vec_cfg = vec_cfg.with_uri(v.clone());
    }
    if let Some(ref v) = app_config.providers.vector_store.collection {
        vec_cfg = vec_cfg.with_collection(v.clone());
    }
    if let Some(d) = app_config.providers.vector_store.dimensions {
        vec_cfg = vec_cfg.with_dimensions(d);
    }
    Ok(vec_cfg)
}

/// Whether the MCP stdio transport should be started.
fn stdio_enabled(mcp: &mcb_infrastructure::config::app::McpConfig) -> bool {
    mcp.stdio_only || !mcp.no_stdio
}

/// Resolve and validate `AppConfig` from Loco settings via the config provider.
fn resolve_app_config(
    ctx: &AppContext,
) -> Result<mcb_infrastructure::config::app::AppConfig, Box<loco_rs::Error>> {
    let settings = ctx.config.settings.clone().ok_or_else(|| {
        Box::new(loco_rs::Error::string(
            "missing loco settings for AppConfig",
        ))
    })?;

    // Resolve config provider via CA/DI registry
    let config_provider = resolve_config_provider(&ConfigProviderConfig::new(
        mcb_utils::constants::DEFAULT_CONFIG_PROVIDER,
    ))
    .map_err(|e| Box::new(loco_rs::Error::string(&e.to_string())))?;

    // Deserialize + validate via resolved provider (production path)
    let app_config_any = config_provider
        .deserialize_from_value(&settings)
        .map_err(|e| Box::new(loco_rs::Error::string(&format!("AppConfig: {e}"))))?;

    let app_config = *app_config_any
        .downcast::<mcb_infrastructure::config::app::AppConfig>()
        .map_err(|_| loco_rs::Error::string("ConfigProvider returned unexpected type"))?;

    Ok(app_config)
}

/// Resolve event bus and provider adapters into a `ServiceResolutionContext`.
fn build_resolution_ctx(
    ctx: &AppContext,
    app_config: mcb_infrastructure::config::app::AppConfig,
) -> Result<ServiceResolutionContext, Box<loco_rs::Error>> {
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
    .map_err(|e| Box::new(loco_rs::Error::string(&e.to_string())))?;

    // Resolve providers via mcb-domain registries — no infrastructure helpers
    let embedding_provider = resolve_embedding_provider(&build_embedding_config(&app_config)?)
        .map_err(|e| Box::new(loco_rs::Error::string(&e.to_string())))?;

    let vector_store_provider =
        resolve_vector_store_provider(&build_vector_store_config(&app_config)?)
            .map_err(|e| Box::new(loco_rs::Error::string(&e.to_string())))?;

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
fn build_bootstrap(
    ctx: &AppContext,
) -> Result<(mcb_server::state::McpServerBootstrap, bool), Box<loco_rs::Error>> {
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
        .map_err(|e| Box::new(loco_rs::Error::string(&e.to_string())))?;

    let bootstrap = build_mcp_server_bootstrap(
        &resolution_ctx,
        Arc::clone(&resolution_ctx.db),
        McpBootstrapProviders {
            embedding: Arc::clone(&resolution_ctx.embedding_provider),
            vector_store: Arc::clone(&resolution_ctx.vector_store_provider),
            hybrid_search,
            execution_flow,
        },
    )
    .map_err(|e| Box::new(loco_rs::Error::string(&e.to_string())))?;
    Ok((bootstrap, start_stdio))
}

/// Spawn the MCP stdio server, detaching the task.
///
/// The supplied watch sender is set to `true` when the stdio server finishes,
/// allowing the main thread to await shutdown in stdio-only mode.
fn spawn_stdio_server(
    mcp_server: Arc<mcb_server::McpServer>,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
) {
    // Detached: handle intentionally dropped so the stdio server runs for the
    // process lifetime. `let _ =` is rejected by clippy::let_underscore_future.
    let _handle = tokio::spawn(async move {
        let server = (*mcp_server).clone();
        if let Err(e) = server.serve_stdio().await {
            mcb_domain::error!("mcp_initializer", "MCP stdio server stopped", &e);
        }
        let _ = shutdown_tx.send(true);
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

        let (bootstrap, start_stdio) = build_bootstrap(ctx).map_err(|e| *e)?;

        if start_stdio {
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            ctx.shared_store.insert(StdioServerShutdown(shutdown_rx));
            spawn_stdio_server(Arc::clone(&bootstrap.mcp_server), shutdown_tx);
        }

        let mcp_server = Arc::clone(&bootstrap.mcp_server);
        let mcb_state = bootstrap.into_mcb_state();
        ctx.shared_store.insert(mcb_state.clone());

        let mcb_router = mcb_server::axum_routes::build_router_without_fallback(
            mcb_state,
            mcp_server,
            ctx.config.settings.clone(),
        );

        Ok(router
            .reset_fallback()
            .merge(mcb_router)
            .fallback(axum::routing::get(|| async {
                (
                    StatusCode::NOT_FOUND,
                    Html(mcb_server::controllers::web::not_found_html()),
                )
            })))
    }
}
