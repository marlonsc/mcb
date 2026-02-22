//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Admin cache endpoints
//!
//! Provides endpoints for monitoring cache statistics.

use std::sync::Arc;

use axum::Json as AxumJson;
use axum::extract::State as AxumState;

use mcb_domain::info;

use crate::admin::auth::AxumAdminAuth;
use crate::admin::error::{AdminError, AdminResult};
use crate::admin::handlers::AdminState;

/// Axum handler: get cache statistics (protected).
///
/// # Errors
/// Returns `503` when cache provider is unavailable and `500` when stats retrieval fails.
pub async fn get_cache_stats_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
) -> AdminResult<mcb_domain::ports::CacheStats> {
    info!("cache", "get_cache_stats called");
    let cache = require_service!(state, cache, "Cache provider not available");
    cache.stats().await.map(AxumJson).map_err(|e| {
        mcb_domain::error!("AdminCache", "failed to get cache stats", &e);
        AdminError::internal("Failed to retrieve cache statistics")
    })
}
