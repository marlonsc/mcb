use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::Method;
use axum::routing::get;
use axum::{Json, Router};
use mcb_domain::ports::{
    IndexingOperationsInterface, PerformanceMetricsInterface, VectorStoreBrowser,
};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::McpServer;

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

    Router::new()
        .route("/health", get(health_handler))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
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
