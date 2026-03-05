//! Health API controller — returns provider health status as JSON.

use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;

/// Returns health status of embedding and vector store providers.
///
/// Calls `EmbeddingProvider::health_check()` and `VectorStoreAdmin::health_check()`
/// on the shared providers from `McbState`.
///
/// # Errors
///
/// Returns JSON with degraded status if any health check fails.
pub async fn health(Extension(state): Extension<McbState>) -> Result<Response> {
    let embedding_healthy = state.embedding_provider.health_check().await.is_ok();
    let vector_store_healthy = state.vector_store.health_check().await.is_ok();

    let status = if embedding_healthy && vector_store_healthy {
        "healthy"
    } else {
        "degraded"
    };

    format::json(serde_json::json!({
        "status": status,
        "embedding": {
            "provider": state.embedding_provider.provider_name(),
            "dimensions": state.embedding_provider.dimensions(),
            "healthy": embedding_healthy,
        },
        "vector_store": {
            "healthy": vector_store_healthy,
        },
    }))
}

/// Registers health API routes.
#[must_use]
pub fn routes() -> Routes {
    Routes::new().prefix("health").add("/", get(health))
}
