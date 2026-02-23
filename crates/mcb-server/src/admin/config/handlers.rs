//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Configuration Management Handlers
//!
//! HTTP handlers for runtime configuration management.

use super::service::{
    ConfigUpdateError, read_update_config, validate_update_prerequisites, write_and_reload_config,
};
use super::{
    ConfigReloadResponse, ConfigResponse, ConfigSectionUpdateRequest, ConfigSectionUpdateResponse,
    SanitizedConfig,
};
use mcb_domain::{error, info};

use crate::admin::handlers::AdminState;

// ---------------------------------------------------------------------------
// Axum handler variants
// ---------------------------------------------------------------------------

use std::sync::Arc;

use axum::Json as AxumJson;
use axum::extract::{Path, State as AxumState};
use axum::http::StatusCode;

use crate::admin::auth::AxumAdminAuth;
use crate::admin::error::{AdminError, AdminStatusResult};

/// Axum variant: get current configuration (sanitized, protected).
/// Axum handler: get current configuration (sanitized, protected).
///
/// # Errors
/// Returns `500` only if response serialization fails.
pub async fn get_config_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
) -> AdminStatusResult<ConfigResponse> {
    info!("get_config_axum", "admin config request");
    let config = if let Some(watcher) = &state.config_watcher {
        watcher.get_config().await
    } else {
        state.current_config.clone()
    };

    let sanitized = SanitizedConfig::from_app_config(&config);

    Ok((
        StatusCode::OK,
        AxumJson(ConfigResponse {
            success: true,
            config: sanitized,
            config_path: state.config_path.as_ref().map(|p| p.display().to_string()),
            last_reload: state
                .config_watcher
                .as_ref()
                .map(|_| chrono::Utc::now().to_rfc3339()),
        }),
    ))
}

/// Axum variant: reload configuration from file (protected).
/// Axum handler: reload configuration from file (protected).
///
/// # Errors
/// Returns `503` when config watcher is unavailable and `500` on reload failures.
pub async fn reload_config_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
) -> AdminStatusResult<ConfigReloadResponse> {
    info!("reload_config_axum", "admin config reload request");
    let watcher = require_service!(state, config_watcher, "Configuration watcher not available");

    match watcher.reload().await {
        Ok(new_config) => {
            let sanitized = SanitizedConfig::from_app_config(&new_config);
            Ok((
                StatusCode::OK,
                AxumJson(ConfigReloadResponse::success(sanitized)),
            ))
        }
        Err(e) => {
            error!("reload_config_axum", "configuration reload failed", &e);
            Err(AdminError::internal("Failed to reload configuration"))
        }
    }
}

/// Axum variant: update a specific configuration section (protected).
/// Axum handler: update a specific configuration section (protected).
///
/// # Errors
/// Returns `400` for invalid input, `503` when configuration services are unavailable,
/// and `500` for read/write/reload failures.
pub async fn update_config_section_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
    Path(section): Path<String>,
    AxumJson(request): AxumJson<ConfigSectionUpdateRequest>,
) -> AdminStatusResult<ConfigSectionUpdateResponse> {
    info!(
        "update_config_section_axum",
        "admin config update request", &section
    );

    let (watcher, config_path) = validate_update_prerequisites(
        &section,
        state.config_watcher.as_ref(),
        state.config_path.as_ref(),
    )
    .map_err(|e| e.to_admin_error(&section))?;

    let updated_config = read_update_config(&config_path, &section, &request.values)
        .map_err(|e| e.to_admin_error(&section))?;

    let sanitized = write_and_reload_config(&config_path, &updated_config, &watcher)
        .await
        .map_err(|e| e.to_admin_error(&section))?;

    Ok((
        StatusCode::OK,
        AxumJson(ConfigSectionUpdateResponse::success(&section, sanitized)),
    ))
}

impl ConfigUpdateError {
    fn to_admin_error(&self, section: &str) -> AdminError {
        match self {
            Self::InvalidSection => AdminError::bad_request(format!("Invalid section: {section}")),
            Self::WatcherUnavailable => {
                AdminError::unavailable("Configuration watcher not available")
            }
            Self::PathUnavailable => {
                AdminError::unavailable("Configuration file path not available")
            }
            Self::ReadFailed(e) => {
                error!(
                    "update_config_section_axum",
                    "failed to read configuration file", e
                );
                AdminError::internal("Failed to read configuration file")
            }
            Self::ParseFailed(e) => {
                error!(
                    "update_config_section_axum",
                    "failed to parse configuration file", e
                );
                AdminError::internal("Failed to parse configuration file")
            }
            Self::InvalidFormat => AdminError::bad_request("Invalid configuration value format"),
            Self::SerializeFailed(e) => {
                error!(
                    "update_config_section_axum",
                    "failed to serialize configuration", e
                );
                AdminError::internal("Failed to serialize configuration")
            }
            Self::WriteFailed(e) => {
                error!(
                    "update_config_section_axum",
                    "failed to write configuration file", e
                );
                AdminError::internal("Failed to write configuration file")
            }
            Self::ReloadFailed(e) => {
                error!(
                    "update_config_section_axum",
                    "configuration updated but reload failed", e
                );
                AdminError::internal("Configuration updated but reload failed")
            }
        }
    }
}
