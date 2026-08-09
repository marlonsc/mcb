//! Health API controller — returns provider health status as JSON.

use crate::state::McbState;
use axum::extract::Extension;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use loco_rs::prelude::*;
use mcb_domain::ports::{IndexingOperationStatus, ValidationStatus};

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

/// Returns a lightweight liveness status for infrastructure probes.
///
/// # Errors
///
/// Returns an error if JSON response serialization fails.
pub async fn alive() -> Result<Response> {
    format::json(serde_json::json!({
        "status": "alive",
    }))
}

/// Returns dependency-aware readiness for traffic admission.
///
/// # Errors
///
/// Returns an error if the readiness response cannot be constructed.
pub async fn ready(Extension(state): Extension<McbState>) -> Result<Response> {
    let report = state.readiness.check().await;
    let status = if report.ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    Ok((
        status,
        axum::Json(serde_json::json!({
            "status": if report.ready { "ready" } else { "not_ready" },
            "dependencies": report.dependencies,
        })),
    )
        .into_response())
}

/// Returns live Prometheus text metrics.
///
/// # Errors
///
/// Returns an error if the metrics response cannot be constructed.
pub async fn metrics(Extension(state): Extension<McbState>) -> Result<Response> {
    let readiness = state.readiness.check().await;
    let indexing = state.indexing_ops.get_operations();
    let validation = state.validation_ops.get_operations();
    let active_indexing = indexing
        .values()
        .filter(|operation| {
            matches!(
                operation.status,
                IndexingOperationStatus::Starting | IndexingOperationStatus::InProgress
            )
        })
        .count();
    let active_validation = validation
        .values()
        .filter(|operation| {
            matches!(
                operation.status,
                ValidationStatus::Queued | ValidationStatus::InProgress
            )
        })
        .count();
    let body = state
        .metrics
        .render(&readiness, active_indexing, active_validation);
    Ok((
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
        .into_response())
}

/// Registers health API routes.
#[must_use]
pub fn routes() -> Routes {
    Routes::new().prefix("health").add("/", get(health))
}
