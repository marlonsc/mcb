//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Admin cache endpoints
//!
//! Provides endpoints for monitoring cache statistics.

use std::sync::Arc;

use axum::Json as AxumJson;
use axum::extract::State as AxumState;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get};
use serde::Serialize;

use crate::admin::auth::{AdminAuth, AxumAdminAuth};
use crate::admin::error::{AdminError, AdminResult};
use crate::admin::handlers::AdminState;

/// Cache error response
#[derive(Serialize)]
pub struct CacheErrorResponse {
    /// Error message describing the cache operation failure
    pub error: String,
}

/// Get cache statistics (protected)
///
/// # Errors
/// Returns `503` when cache provider is unavailable and `500` when stats retrieval fails.
#[get("/cache/stats")]
pub async fn get_cache_stats(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<mcb_domain::ports::CacheStats>, (Status, Json<CacheErrorResponse>)> {
    tracing::info!("get_cache_stats called");
    let Some(cache) = &state.cache else {
        return Err((
            Status::ServiceUnavailable,
            Json(CacheErrorResponse {
                error: "Cache provider not available".to_owned(),
            }),
        ));
    };

    match cache.stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            tracing::error!(error = %e, "failed to get cache stats");
            Err((
                Status::InternalServerError,
                Json(CacheErrorResponse {
                    error: "Failed to retrieve cache statistics".to_owned(),
                }),
            ))
        }
    }
}

/// Axum handler: get cache statistics (protected).
///
/// # Errors
/// Returns `503` when cache provider is unavailable and `500` when stats retrieval fails.
pub async fn get_cache_stats_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
) -> AdminResult<mcb_domain::ports::CacheStats> {
    tracing::info!("get_cache_stats called");
    let cache = require_service!(state, cache, "Cache provider not available");
    cache.stats().await.map(AxumJson).map_err(|e| {
        tracing::error!(error = %e, "failed to get cache stats");
        AdminError::internal("Failed to retrieve cache statistics")
    })
}
