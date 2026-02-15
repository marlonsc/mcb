//! Admin cache endpoints
//!
//! Provides endpoints for monitoring cache statistics.

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get};
use serde::Serialize;

use crate::admin::auth::AdminAuth;
use crate::admin::handlers::AdminState;

/// Cache error response
#[derive(Serialize)]
pub struct CacheErrorResponse {
    /// Error message describing the cache operation failure
    pub error: String,
}

/// Get cache statistics (protected)
#[get("/cache/stats")]
pub async fn get_cache_stats(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<mcb_domain::ports::providers::cache::CacheStats>, (Status, Json<CacheErrorResponse>)>
{
    tracing::info!("get_cache_stats called");
    let Some(cache) = &state.cache else {
        return Err((
            Status::ServiceUnavailable,
            Json(CacheErrorResponse {
                error: "Cache provider not available".to_string(),
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
                    error: "Failed to retrieve cache statistics".to_string(),
                }),
            ))
        }
    }
}
