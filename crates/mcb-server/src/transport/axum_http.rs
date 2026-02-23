use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use axum::{Extension, Json, Router};
use mcb_domain::info;
use mcb_domain::ports::{
    IndexingOperationsInterface, PerformanceMetricsInterface, VectorStoreBrowser,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::McpServer;
use crate::admin::auth::{AdminAuthConfig, axum_admin_auth_layer};
use crate::admin::browse::{
    list_browse_issues, list_browse_organizations, list_browse_plans, list_browse_projects,
    list_browse_repositories,
};
use crate::admin::browse_handlers::{
    BrowseState, get_collection_tree_axum, get_file_chunks_axum, list_collection_files_axum,
    list_collections_axum,
};
use crate::admin::cache::get_cache_stats_axum;
use crate::admin::config::handlers::{
    get_config_axum, reload_config_axum, update_config_section_axum,
};
use crate::admin::control::shutdown_axum;
use crate::admin::handlers::AdminState;
use crate::admin::health::{extended_health_check_axum, get_metrics_axum};
use crate::admin::jobs::get_jobs_status_axum;
use crate::admin::lifecycle_handlers::{
    list_services_axum, restart_service_axum, services_health_axum, start_service_axum,
    stop_service_axum,
};
use crate::admin::models::{AdminHealthResponse, ReadinessResponse};
use crate::admin::sse::events_stream;
use crate::admin::web::router::web_router_with_state;

/// Shared state for Axum transport endpoints.
#[derive(Clone)]
pub struct AppState {
    /// Metrics service used by admin and transport health surfaces.
    pub metrics: Arc<dyn PerformanceMetricsInterface>,
    /// Indexing operations service used by HTTP endpoints.
    pub indexing: Arc<dyn IndexingOperationsInterface>,
    /// Optional vector store browser for browse endpoints.
    pub browser: Option<Arc<dyn VectorStoreBrowser>>,
    /// Optional browse state for collection/file tree endpoints.
    pub browse_state: Option<Arc<BrowseState>>,
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
    pub fn new(state: &Arc<AppState>) -> Self {
        Self {
            router: build_router(state),
        }
    }

    /// Returns the wrapped [`Router`].
    pub fn into_inner(self) -> Router {
        self.router
    }
}

/// Builds the Axum router with middleware and routes.
pub fn build_router(state: &Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers(Any);

    let mut app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .route("/live", get(liveness_handler))
        .with_state(Arc::clone(state));

    // Mount config routes when admin state is available
    if let (Some(admin_state), Some(auth_config)) =
        (state.admin_state.clone(), state.auth_config.clone())
    {
        let admin_routes = Router::new()
            .route("/metrics", get(get_metrics_axum))
            .route("/health/extended", get(extended_health_check_axum))
            .route("/config", get(get_config_axum))
            .route("/config/reload", post(reload_config_axum))
            .route("/config/{section}", patch(update_config_section_axum))
            .route("/shutdown", post(shutdown_axum))
            .route("/cache/stats", get(get_cache_stats_axum))
            .route("/services", get(list_services_axum))
            .route("/services/health", get(services_health_axum))
            .route("/services/{name}/start", post(start_service_axum))
            .route("/services/{name}/stop", post(stop_service_axum))
            .route("/services/{name}/restart", post(restart_service_axum))
            .route("/browse/projects", get(list_browse_projects))
            .route("/browse/repositories", get(list_browse_repositories))
            .route("/browse/plans", get(list_browse_plans))
            .route("/browse/issues", get(list_browse_issues))
            .route("/browse/organizations", get(list_browse_organizations))
            .layer(axum::middleware::from_fn(axum_admin_auth_layer))
            .layer(Extension(auth_config))
            .with_state(Arc::clone(&admin_state));

        let public_admin_routes = Router::new()
            .route("/jobs", get(get_jobs_status_axum))
            .route("/events", get(events_stream))
            .with_state(Arc::clone(&admin_state));

        app = app
            .merge(admin_routes)
            .merge(public_admin_routes)
            .merge(web_router_with_state((*admin_state).clone()));

        if let Some(browse_state) = state.browse_state.clone() {
            let browse_routes = Router::new()
                .route("/collections", get(list_collections_axum))
                .route("/collections/{name}/files", get(list_collection_files_axum))
                .route(
                    "/collections/{name}/chunks/{*path}",
                    get(get_file_chunks_axum),
                )
                .route("/collections/{name}/tree", get(get_collection_tree_axum))
                .layer(axum::middleware::from_fn(axum_admin_auth_layer))
                .layer(Extension(state.auth_config.clone().unwrap_or_default()))
                .with_state(browse_state);
            app = app.merge(browse_routes);
        }
    }

    app.layer(TraceLayer::new_for_http()).layer(cors)
}

/// Binds an address and serves the Axum router.
///
/// # Errors
/// Returns an error if binding or serving fails.
pub async fn run_axum_server(
    addr: SocketAddr,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let router = AxumRouter::new(&state).into_inner();
    let local_addr = listener.local_addr()?;
    info!("AxumTransport", "Axum transport listening", &local_addr);
    axum::serve(listener, router).await?;

    Ok(())
}

async fn health_handler(State(state): State<Arc<AppState>>) -> Json<AdminHealthResponse> {
    let metrics = state.metrics.get_performance_metrics();
    let operations = state.indexing.get_operations();

    Json(AdminHealthResponse {
        status: "healthy",
        uptime_seconds: metrics.uptime_seconds,
        active_indexing_operations: operations.len(),
    })
}

async fn readiness_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metrics = state.metrics.get_performance_metrics();
    let ready = metrics.uptime_seconds >= 1;
    let status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(ReadinessResponse {
            ready,
            uptime_seconds: metrics.uptime_seconds,
        }),
    )
}

async fn liveness_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metrics = state.metrics.get_performance_metrics();
    (
        StatusCode::OK,
        Json(crate::admin::models::LivenessResponse {
            alive: true,
            uptime_seconds: metrics.uptime_seconds,
        }),
    )
}
