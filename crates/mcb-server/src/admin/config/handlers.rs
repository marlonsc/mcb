//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Configuration Management Handlers
//!
//! HTTP handlers for runtime configuration management.

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get, patch, post};

use super::service::{
    ConfigUpdateError, read_update_config, validate_update_prerequisites, write_and_reload_config,
};
use super::{
    ConfigReloadResponse, ConfigResponse, ConfigSectionUpdateRequest, ConfigSectionUpdateResponse,
    SanitizedConfig,
};
use mcb_domain::error;

use crate::admin::auth::AdminAuth;
use crate::admin::handlers::AdminState;

/// Get current configuration (sanitized, protected)
#[get("/config")]
pub async fn get_config(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> (Status, Json<ConfigResponse>) {
    tracing::info!(handler = "get_config", "admin config request");
    let config = if let Some(watcher) = &state.config_watcher {
        watcher.get_config().await
    } else {
        state.current_config.clone()
    };

    let sanitized = SanitizedConfig::from_app_config(&config);

    (
        Status::Ok,
        Json(ConfigResponse {
            success: true,
            config: sanitized,
            config_path: state.config_path.as_ref().map(|p| p.display().to_string()),
            last_reload: state
                .config_watcher
                .as_ref()
                .map(|_| chrono::Utc::now().to_rfc3339()),
        }),
    )
}

/// Reload configuration from file (protected)
#[post("/config/reload")]
pub async fn reload_config(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> (Status, Json<ConfigReloadResponse>) {
    tracing::info!(handler = "reload_config", "admin config reload request");
    let Some(watcher) = &state.config_watcher else {
        return (
            Status::ServiceUnavailable,
            Json(ConfigReloadResponse::watcher_unavailable()),
        );
    };

    match watcher.reload().await {
        Ok(new_config) => {
            let sanitized = SanitizedConfig::from_app_config(&new_config);
            (Status::Ok, Json(ConfigReloadResponse::success(sanitized)))
        }
        Err(e) => {
            error!("reload_config", "configuration reload failed", &e);
            (
                Status::InternalServerError,
                Json(ConfigReloadResponse::failure(
                    "Failed to reload configuration".to_owned(),
                )),
            )
        }
    }
}

/// Update a specific configuration section (protected)
#[patch("/config/<section>", format = "json", data = "<request>")]
pub async fn update_config_section(
    _auth: AdminAuth,
    state: &State<AdminState>,
    section: &str,
    request: Json<ConfigSectionUpdateRequest>,
) -> (Status, Json<ConfigSectionUpdateResponse>) {
    tracing::info!(
        handler = "update_config_section",
        section = section,
        "admin config update request"
    );
    let request = request.into_inner();

    // Validate and get required resources
    let (watcher, config_path) = match validate_update_prerequisites(
        section,
        state.config_watcher.as_ref(),
        state.config_path.as_ref(),
    ) {
        Ok(resources) => resources,
        Err(e) => return e.to_response(section),
    };

    // Read and update configuration
    let updated_config = match read_update_config(&config_path, section, &request.values) {
        Ok(config) => config,
        Err(e) => return e.to_response(section),
    };

    // Write and reload
    match write_and_reload_config(&config_path, &updated_config, &watcher).await {
        Ok(sanitized) => (
            Status::Ok,
            Json(ConfigSectionUpdateResponse::success(section, sanitized)),
        ),
        Err(e) => e.to_response(section),
    }
}

impl ConfigUpdateError {
    fn to_response(&self, section: &str) -> (Status, Json<ConfigSectionUpdateResponse>) {
        use ConfigSectionUpdateResponse as Resp;
        match self {
            Self::InvalidSection => (Status::BadRequest, Json(Resp::invalid_section(section))),
            Self::WatcherUnavailable => (
                Status::ServiceUnavailable,
                Json(Resp::watcher_unavailable(section)),
            ),
            Self::PathUnavailable => (
                Status::ServiceUnavailable,
                Json(Resp::failure(
                    section,
                    "Configuration file path not available",
                )),
            ),
            Self::ReadFailed(e) => {
                error!(
                    "update_config_section",
                    "failed to read configuration file", e
                );
                (
                    Status::InternalServerError,
                    Json(Resp::failure(section, "Failed to read configuration file")),
                )
            }
            Self::ParseFailed(e) => {
                error!(
                    "update_config_section",
                    "failed to parse configuration file", e
                );
                (
                    Status::InternalServerError,
                    Json(Resp::failure(section, "Failed to parse configuration file")),
                )
            }
            Self::InvalidFormat => (
                Status::BadRequest,
                Json(Resp::failure(section, "Invalid configuration value format")),
            ),
            Self::SerializeFailed(e) => {
                error!(
                    "update_config_section",
                    "failed to serialize configuration", e
                );
                (
                    Status::InternalServerError,
                    Json(Resp::failure(section, "Failed to serialize configuration")),
                )
            }
            Self::WriteFailed(e) => {
                error!(
                    "update_config_section",
                    "failed to write configuration file", e
                );
                (
                    Status::InternalServerError,
                    Json(Resp::failure(section, "Failed to write configuration file")),
                )
            }
            Self::ReloadFailed(e) => {
                error!(
                    "update_config_section",
                    "configuration updated but reload failed", e
                );
                (
                    Status::InternalServerError,
                    Json(Resp::failure(
                        section,
                        "Configuration updated but reload failed",
                    )),
                )
            }
        }
    }
}

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
    tracing::info!(handler = "get_config_axum", "admin config request");
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
    tracing::info!(
        handler = "reload_config_axum",
        "admin config reload request"
    );
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
    tracing::info!(
        handler = "update_config_section_axum",
        section = %section,
        "admin config update request"
    );

    let (watcher, config_path) = validate_update_prerequisites(
        &section,
        state.config_watcher.as_ref(),
        state.config_path.as_ref(),
    )
    .map_err(|e| e.into_admin_error(&section))?;

    let updated_config = read_update_config(&config_path, &section, &request.values)
        .map_err(|e| e.into_admin_error(&section))?;

    let sanitized = write_and_reload_config(&config_path, &updated_config, &watcher)
        .await
        .map_err(|e| e.into_admin_error(&section))?;

    Ok((
        StatusCode::OK,
        AxumJson(ConfigSectionUpdateResponse::success(&section, sanitized)),
    ))
}

impl ConfigUpdateError {
    fn into_admin_error(&self, section: &str) -> AdminError {
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
