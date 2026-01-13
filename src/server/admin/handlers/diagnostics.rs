//! Diagnostic operation handlers

use super::common::*;
use crate::infrastructure::utils::IntoStatusCode;

/// Run comprehensive health check
pub async fn health_check_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::HealthCheckResult>>, StatusCode> {
    let result = state.admin_service.run_health_check().await.to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

/// Test provider connectivity
pub async fn test_connectivity_handler(
    State(state): State<AdminState>,
    Path(provider_id): Path<String>,
) -> Result<Json<ApiResponse<crate::admin::service::ConnectivityTestResult>>, StatusCode> {
    let result = state
        .admin_service
        .test_provider_connectivity(&provider_id)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

/// Run performance test
pub async fn performance_test_handler(
    State(state): State<AdminState>,
    Json(test_config): Json<crate::admin::service::PerformanceTestConfig>,
) -> Result<Json<ApiResponse<crate::admin::service::PerformanceTestResult>>, StatusCode> {
    let result = state
        .admin_service
        .run_performance_test(test_config)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(result)))
}
