use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::Method;
use axum::routing::{get, patch, post};
use axum::{Extension, Json, Router};
use mcb_domain::ports::{
    IndexingOperationsInterface, PerformanceMetricsInterface, VectorStoreBrowser,
};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::McpServer;
use crate::admin::auth::{AdminAuthConfig, axum_admin_auth_layer};
use crate::admin::cache::get_cache_stats_axum;
use crate::admin::config::handlers::{
    get_config_axum, reload_config_axum, update_config_section_axum,
};
use crate::admin::control::shutdown_axum;
use crate::admin::handlers::AdminState;
use crate::admin::jobs::get_jobs_status_axum;

/// Shared state for Axum transport endpoints.
#[derive(Clone)]
pub struct AppState {
    /// Metrics service used by admin and transport health surfaces.
    pub metrics: Arc<dyn PerformanceMetricsInterface>,
    /// Indexing operations service used by HTTP endpoints.
    pub indexing: Arc<dyn IndexingOperationsInterface>,
    /// Optional code browser dependency for browse routes.
    pub browser: Option<Arc<dyn VectorStoreBrowser>>,
    /// Optional MCP server handle for protocol endpoints.
    pub mcp_server: Option<Arc<McpServer>>,
    /// Optional admin state for config/lifecycle endpoints.
    pub admin_state: Option<Arc<AdminState>>,
    /// Optional auth configuration for admin endpoints.
    pub auth_config: Option<Arc<AdminAuthConfig>>,
}

/// Wrapper type holding the composed Axum router.
pub struct AxumRouter {
    router: Router,
}

impl AxumRouter {
    /// Creates an [`AxumRouter`] from shared state.
    #[must_use]
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            router: build_router(state),
        }
    }

    /// Returns the wrapped [`Router`].
    #[must_use]
    pub fn into_inner(self) -> Router {
        self.router
    }
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

/// Builds the Axum router with middleware and routes.
#[must_use]
pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers(Any);

    let mut app = Router::new()
        .route("/health", get(health_handler))
        .with_state(state.clone());

    // Mount config routes when admin state is available
    if let (Some(admin_state), Some(auth_config)) =
        (state.admin_state.clone(), state.auth_config.clone())
    {
        let admin_routes = Router::new()
            .route("/config", get(get_config_axum))
            .route("/config/reload", post(reload_config_axum))
            .route("/config/{section}", patch(update_config_section_axum))
            .route("/shutdown", post(shutdown_axum))
            .route("/cache/stats", get(get_cache_stats_axum))
            .layer(axum::middleware::from_fn(axum_admin_auth_layer))
            .layer(Extension(auth_config))
            .with_state(Arc::clone(&admin_state));

        let public_admin_routes = Router::new()
            .route("/jobs", get(get_jobs_status_axum))
            .with_state(admin_state);

        app = app.merge(admin_routes).merge(public_admin_routes);
    }

    app.layer(TraceLayer::new_for_http()).layer(cors)
}

/// Binds an address and serves the Axum router.
pub async fn run_axum_server(
    addr: SocketAddr,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let router = AxumRouter::new(state).into_inner();

    tracing::info!("Axum transport listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}

async fn health_handler(State(_state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse { status: "healthy" })
}
