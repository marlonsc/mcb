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
use crate::admin::auth::AdminAuth;
use crate::admin::handlers::AdminState;

/// Get current configuration (sanitized, protected)
#[get("/config")]
pub async fn get_config(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> (Status, Json<ConfigResponse>) {
    tracing::info!("get_config called");
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
    tracing::info!("reload_config called");
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
        Err(e) => (
            Status::InternalServerError,
            Json(ConfigReloadResponse::failure(format!(
                "Failed to reload configuration: {}",
                e
            ))),
        ),
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
    tracing::info!("update_config_section called");
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
            Self::ReadFailed(e) => (
                Status::InternalServerError,
                Json(Resp::failure(
                    section,
                    format!("Failed to read configuration file: {}", e),
                )),
            ),
            Self::ParseFailed(e) => (
                Status::InternalServerError,
                Json(Resp::failure(
                    section,
                    format!("Failed to parse configuration file: {}", e),
                )),
            ),
            Self::InvalidFormat => (
                Status::BadRequest,
                Json(Resp::failure(section, "Invalid configuration value format")),
            ),
            Self::SerializeFailed(e) => (
                Status::InternalServerError,
                Json(Resp::failure(
                    section,
                    format!("Failed to serialize configuration: {}", e),
                )),
            ),
            Self::WriteFailed(e) => (
                Status::InternalServerError,
                Json(Resp::failure(
                    section,
                    format!("Failed to write configuration file: {}", e),
                )),
            ),
            Self::ReloadFailed(e) => (
                Status::InternalServerError,
                Json(Resp::failure(
                    section,
                    format!("Configuration updated but reload failed: {}", e),
                )),
            ),
        }
    }
}
