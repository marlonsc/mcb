//! Data management handlers (backup/restore)

use super::common::*;
use crate::infrastructure::utils::IntoStatusCode;

/// Create system backup
pub async fn create_backup_handler(
    State(state): State<AdminState>,
    Json(backup_config): Json<crate::admin::service::BackupConfig>,
) -> Result<Json<ApiResponse<crate::admin::service::BackupResult>>, StatusCode> {
    let result = state
        .admin_service
        .create_backup(backup_config)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

/// List available backups
pub async fn list_backups_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<crate::admin::service::BackupInfo>>>, StatusCode> {
    let backups = state.admin_service.list_backups().await.to_500()?;

    Ok(Json(ApiResponse::success(backups)))
}

/// Restore from backup
pub async fn restore_backup_handler(
    State(state): State<AdminState>,
    Path(backup_id): Path<String>,
) -> Result<Json<ApiResponse<crate::admin::service::RestoreResult>>, StatusCode> {
    let result = state
        .admin_service
        .restore_backup(&backup_id)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(result)))
}
