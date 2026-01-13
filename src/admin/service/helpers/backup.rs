//! Backup operations helper module
//!
//! Provides functions for backup creation, listing, and restoration.

use crate::admin::service::helpers::admin_defaults;
use crate::admin::service::types::{
    AdminError, BackupConfig, BackupInfo, BackupResult, RestoreResult,
};
use crate::infrastructure::events::SharedEventBusProvider;
use crate::infrastructure::service_helpers::TimedOperation;

/// Create a new backup
pub async fn create_backup(
    event_bus: &SharedEventBusProvider,
    backup_config: BackupConfig,
) -> Result<BackupResult, AdminError> {
    let backup_id = format!("backup_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let path = format!(
        "{}/{}.tar.gz",
        admin_defaults::DEFAULT_BACKUPS_DIR,
        backup_config.name
    );

    // Publish backup event - actual backup created asynchronously by BackupManager
    // Use list_backups() to check completion status and get actual file size
    event_bus
        .publish(crate::infrastructure::events::SystemEvent::BackupCreate { path: path.clone() })
        .await
        .map_err(|e| {
            AdminError::McpServerError(format!("Failed to publish BackupCreate event: {}", e))
        })?;

    tracing::info!(
        "[ADMIN] Backup creation initiated: {} -> {}",
        backup_config.name,
        path
    );

    // Return immediately - size_bytes is 0 until backup completes
    // Client should poll list_backups() for completion status
    Ok(BackupResult {
        backup_id,
        name: backup_config.name,
        size_bytes: 0, // Async - check list_backups() for actual size
        created_at: chrono::Utc::now(),
        path,
    })
}

/// List all available backups
pub fn list_backups() -> Result<Vec<BackupInfo>, AdminError> {
    let backups_dir = std::path::PathBuf::from(admin_defaults::DEFAULT_BACKUPS_DIR);
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    let entries = std::fs::read_dir(&backups_dir)
        .map_err(|e| AdminError::ConfigError(format!("Failed to read backups directory: {}", e)))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "gz") {
            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(metadata) = entry.metadata() {
                    let created_at = metadata
                        .created()
                        .or_else(|_| metadata.modified())
                        .map(chrono::DateTime::<chrono::Utc>::from)
                        .unwrap_or_else(|_| chrono::Utc::now());

                    backups.push(BackupInfo {
                        id: filename.to_string(),
                        name: filename.replace("_", " ").replace(".tar", ""),
                        created_at,
                        size_bytes: metadata.len(),
                        status: "completed".to_string(),
                    });
                }
            }
        }
    }

    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(backups)
}

/// Restore from a backup with automated process
pub async fn restore_backup(
    event_bus: &SharedEventBusProvider,
    backup_id: &str,
) -> Result<RestoreResult, AdminError> {
    let timer = TimedOperation::start();
    let backups_dir = std::path::PathBuf::from(admin_defaults::DEFAULT_BACKUPS_DIR);
    let backup_path = backups_dir.join(format!("{}.tar.gz", backup_id));

    // Validate backup exists
    if !backup_path.exists() {
        return Err(AdminError::ConfigError(format!(
            "Backup not found: {}",
            backup_id
        )));
    }

    // Validate backup file is readable and not empty
    match std::fs::metadata(&backup_path) {
        Ok(metadata) => {
            if metadata.len() == 0 {
                return Err(AdminError::ConfigError(format!(
                    "Backup file is empty: {}",
                    backup_id
                )));
            }
        }
        Err(e) => {
            return Err(AdminError::ConfigError(format!(
                "Cannot read backup file {}: {}",
                backup_id, e
            )));
        }
    }

    // Create rollback backup before restore
    let rollback_id = format!("rollback_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let rollback_path = backups_dir.join(format!("{}.tar.gz", rollback_id));

    tracing::info!(
        "[ADMIN] Creating rollback backup before restore: {}",
        rollback_id
    );

    // Attempt to create a rollback backup of current data
    if let Ok(data_dir) = std::fs::read_dir(admin_defaults::DEFAULT_DATA_DIR) {
        let mut found_files = false;
        for entry in data_dir.flatten() {
            if entry.path().is_file() {
                found_files = true;
                break;
            }
        }

        if found_files {
            match create_rollback_backup(&rollback_path) {
                Ok(_) => {
                    tracing::info!("[ADMIN] Rollback backup created: {}", rollback_id);
                }
                Err(e) => {
                    tracing::warn!("[ADMIN] Failed to create rollback backup: {}", e);
                    // Continue anyway, but log the failure
                }
            }
        }
    }

    // Perform the restore
    tracing::info!(
        "[ADMIN] Starting restore from backup: {} -> {}",
        backup_id,
        admin_defaults::DEFAULT_DATA_DIR
    );

    match extract_backup(&backup_path, admin_defaults::DEFAULT_DATA_DIR) {
        Ok(_) => {
            // Publish restore event
            let _ = event_bus
                .publish(crate::infrastructure::events::SystemEvent::BackupRestore {
                    path: backup_path.to_string_lossy().to_string(),
                })
                .await;

            tracing::info!(
                "[ADMIN] Backup restore completed successfully: {}",
                backup_id
            );

            Ok(RestoreResult {
                success: true,
                backup_id: backup_id.to_string(),
                restored_at: chrono::Utc::now(),
                items_restored: count_restored_files(admin_defaults::DEFAULT_DATA_DIR),
                rollback_id: Some(rollback_id.clone()),
                message: format!(
                    "Backup '{}' restored successfully. \
                     Rollback backup saved as '{}'",
                    backup_id, rollback_id
                ),
                execution_time_ms: timer.elapsed_ms(),
            })
        }
        Err(e) => {
            tracing::error!("[ADMIN] Backup restore failed for {}: {}", backup_id, e);

            // Attempt to restore from rollback if restore failed
            if rollback_path.exists() {
                tracing::warn!("[ADMIN] Attempting rollback restoration");
                match extract_backup(&rollback_path, admin_defaults::DEFAULT_DATA_DIR) {
                    Ok(_) => {
                        tracing::info!("[ADMIN] Rollback restoration completed");
                        return Ok(RestoreResult {
                            success: false,
                            backup_id: backup_id.to_string(),
                            restored_at: chrono::Utc::now(),
                            items_restored: 0,
                            rollback_id: Some(rollback_id),
                            message: format!(
                                "Failed to restore backup '{}': {}. \
                                 Rolled back to previous state.",
                                backup_id, e
                            ),
                            execution_time_ms: timer.elapsed_ms(),
                        });
                    }
                    Err(rollback_err) => {
                        tracing::error!(
                            "[ADMIN] Rollback restoration also failed: {}",
                            rollback_err
                        );
                    }
                }
            }

            Err(AdminError::ConfigError(format!(
                "Failed to restore backup '{}': {}",
                backup_id, e
            )))
        }
    }
}

/// Helper function to extract backup archive
fn extract_backup(backup_path: &std::path::Path, target_dir: &str) -> Result<(), String> {
    // Ensure target directory exists
    std::fs::create_dir_all(target_dir)
        .map_err(|e| format!("Cannot create target directory: {}", e))?;

    // Extract tar.gz file
    let file =
        std::fs::File::open(backup_path).map_err(|e| format!("Cannot open backup file: {}", e))?;

    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    archive
        .unpack(target_dir)
        .map_err(|e| format!("Cannot extract archive: {}", e))?;

    Ok(())
}

/// Helper function to create rollback backup
fn create_rollback_backup(backup_path: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::create(backup_path)
        .map_err(|e| format!("Cannot create backup file: {}", e))?;

    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut archive = tar::Builder::new(encoder);

    archive
        .append_dir_all("data", admin_defaults::DEFAULT_DATA_DIR)
        .map_err(|e| format!("Cannot archive directory: {}", e))?;

    archive
        .finish()
        .map_err(|e| format!("Cannot finalize archive: {}", e))?;

    Ok(())
}

/// Helper function to count restored files
fn count_restored_files(path: &str) -> u64 {
    match std::fs::read_dir(path) {
        Ok(entries) => entries
            .flatten()
            .filter(|entry| entry.path().is_file())
            .count() as u64,
        Err(_) => 0,
    }
}
