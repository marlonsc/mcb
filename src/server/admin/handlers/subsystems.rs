//! Subsystem control handlers

use super::common::*;
use super::SubsystemSignalRequest;
use crate::infrastructure::utils::IntoStatusCode;

/// Get all subsystems and their status
pub async fn get_subsystems_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<crate::admin::service::SubsystemInfo>>>, StatusCode> {
    let subsystems = state.admin_service.get_subsystems().await.to_500()?;

    Ok(Json(ApiResponse::success(subsystems)))
}

/// Send a control signal to a subsystem
pub async fn send_subsystem_signal_handler(
    State(state): State<AdminState>,
    Path(subsystem_id): Path<String>,
    Json(request): Json<SubsystemSignalRequest>,
) -> Result<Json<ApiResponse<crate::admin::service::SignalResult>>, StatusCode> {
    let result = state
        .admin_service
        .send_subsystem_signal(&subsystem_id, request.signal)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

/// Get all registered HTTP routes
pub async fn get_routes_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<crate::admin::service::RouteInfo>>>, StatusCode> {
    let routes = state.admin_service.get_routes().await.to_500()?;

    Ok(Json(ApiResponse::success(routes)))
}

/// Reload router configuration
pub async fn reload_routes_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::MaintenanceResult>>, StatusCode> {
    let result = state.admin_service.reload_routes().await.to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

// ============================================================================
// Recovery Management Handlers
// ============================================================================

/// Get recovery status for all subsystems
pub async fn get_recovery_status_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<crate::daemon::types::RecoveryState>>>, StatusCode> {
    if let Some(recovery_manager) = &state.recovery_manager {
        let states = recovery_manager.get_recovery_states();
        Ok(Json(ApiResponse::success(states)))
    } else {
        Ok(Json(ApiResponse::success(Vec::new())))
    }
}

/// Reset recovery state for a specific subsystem
pub async fn reset_recovery_state_handler(
    State(state): State<AdminState>,
    Path(subsystem_id): Path<String>,
) -> Result<Json<ApiResponse<crate::admin::service::SignalResult>>, StatusCode> {
    if let Some(recovery_manager) = &state.recovery_manager {
        recovery_manager
            .reset_recovery_state(&subsystem_id)
            .to_404()?;

        Ok(Json(ApiResponse::success(
            crate::admin::service::SignalResult {
                success: true,
                subsystem_id: subsystem_id.clone(),
                signal: "reset".to_string(),
                message: format!("Recovery state reset for '{}'", subsystem_id),
            },
        )))
    } else {
        Ok(Json(ApiResponse::error(
            "Recovery manager not available".to_string(),
        )))
    }
}

/// Manually trigger recovery for a specific subsystem
pub async fn trigger_recovery_handler(
    State(state): State<AdminState>,
    Path(subsystem_id): Path<String>,
) -> Result<Json<ApiResponse<crate::admin::service::SignalResult>>, StatusCode> {
    if let Some(recovery_manager) = &state.recovery_manager {
        recovery_manager
            .trigger_recovery(&subsystem_id)
            .await
            .to_500()?;

        Ok(Json(ApiResponse::success(
            crate::admin::service::SignalResult {
                success: true,
                subsystem_id: subsystem_id.clone(),
                signal: "trigger".to_string(),
                message: format!("Recovery triggered for '{}'", subsystem_id),
            },
        )))
    } else {
        Ok(Json(ApiResponse::error(
            "Recovery manager not available".to_string(),
        )))
    }
}
