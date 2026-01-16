//! Admin request handlers
//!
//! HTTP handlers for admin API endpoints including health checks,
//! performance metrics, indexing status, and runtime configuration management.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mcb_domain::ports::admin::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_infrastructure::config::watcher::ConfigWatcher;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

use super::config::{
    ConfigReloadResponse, ConfigResponse, ConfigSectionUpdateRequest, ConfigSectionUpdateResponse,
    SanitizedConfig,
};

/// Admin handler state containing shared service references
#[derive(Clone)]
pub struct AdminState {
    /// Performance metrics tracker
    pub metrics: Arc<dyn PerformanceMetricsInterface>,
    /// Indexing operations tracker
    pub indexing: Arc<dyn IndexingOperationsInterface>,
    /// Configuration watcher for hot-reload support
    pub config_watcher: Option<Arc<ConfigWatcher>>,
    /// Configuration file path (for updates)
    pub config_path: Option<PathBuf>,
}

/// Health check response for admin API
#[derive(Serialize)]
pub struct AdminHealthResponse {
    /// Server status
    pub status: &'static str,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Number of active indexing operations
    pub active_indexing_operations: usize,
}

/// Health check endpoint
pub async fn health_check(State(state): State<AdminState>) -> impl IntoResponse {
    let metrics = state.metrics.get_performance_metrics();
    let operations = state.indexing.get_operations();

    Json(AdminHealthResponse {
        status: "healthy",
        uptime_seconds: metrics.uptime_seconds,
        active_indexing_operations: operations.len(),
    })
}

/// Get performance metrics endpoint
pub async fn get_metrics(State(state): State<AdminState>) -> impl IntoResponse {
    let metrics = state.metrics.get_performance_metrics();
    Json(metrics)
}

/// Indexing status response
#[derive(Serialize)]
pub struct IndexingStatusResponse {
    /// Whether indexing is currently active
    pub is_indexing: bool,
    /// Number of active operations
    pub active_operations: usize,
    /// Details of each operation
    pub operations: Vec<IndexingOperationStatus>,
}

/// Individual indexing operation status
#[derive(Serialize)]
pub struct IndexingOperationStatus {
    /// Operation ID
    pub id: String,
    /// Collection being indexed
    pub collection: String,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Progress as percentage
    pub progress_percent: f32,
    /// Files processed
    pub processed_files: usize,
    /// Total files
    pub total_files: usize,
}

/// Get indexing status endpoint
pub async fn get_indexing_status(State(state): State<AdminState>) -> impl IntoResponse {
    let operations = state.indexing.get_operations();

    let operation_statuses: Vec<IndexingOperationStatus> = operations
        .values()
        .map(|op| {
            let progress = if op.total_files > 0 {
                (op.processed_files as f32 / op.total_files as f32) * 100.0
            } else {
                0.0
            };

            IndexingOperationStatus {
                id: op.id.clone(),
                collection: op.collection.clone(),
                current_file: op.current_file.clone(),
                progress_percent: progress,
                processed_files: op.processed_files,
                total_files: op.total_files,
            }
        })
        .collect();

    Json(IndexingStatusResponse {
        is_indexing: !operation_statuses.is_empty(),
        active_operations: operation_statuses.len(),
        operations: operation_statuses,
    })
}

/// Readiness check endpoint (for k8s/docker health checks)
pub async fn readiness_check(State(state): State<AdminState>) -> impl IntoResponse {
    let metrics = state.metrics.get_performance_metrics();

    // Consider ready if server has been up for at least 1 second
    if metrics.uptime_seconds >= 1 {
        (StatusCode::OK, Json(serde_json::json!({ "ready": true })))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "ready": false })),
        )
    }
}

/// Liveness check endpoint (for k8s/docker health checks)
pub async fn liveness_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "alive": true })))
}

// ============================================================================
// Configuration Management Endpoints
// ============================================================================

/// Get current configuration (sanitized)
///
/// Returns the current configuration with sensitive fields removed.
/// API keys, secrets, and passwords are not exposed.
pub async fn get_config(State(state): State<AdminState>) -> impl IntoResponse {
    let Some(watcher) = &state.config_watcher else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ConfigResponse {
                success: false,
                config: SanitizedConfig::default(),
                config_path: None,
                last_reload: None,
            }),
        );
    };

    let config = watcher.get_config().await;
    let sanitized = SanitizedConfig::from_app_config(&config);

    (
        StatusCode::OK,
        Json(ConfigResponse {
            success: true,
            config: sanitized,
            config_path: state.config_path.as_ref().map(|p| p.display().to_string()),
            last_reload: Some(chrono::Utc::now().to_rfc3339()),
        }),
    )
}

/// Reload configuration from file
///
/// Triggers a manual configuration reload. The new configuration
/// will be validated before being applied.
pub async fn reload_config(State(state): State<AdminState>) -> impl IntoResponse {
    let Some(watcher) = &state.config_watcher else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ConfigReloadResponse::watcher_unavailable()),
        );
    };

    match watcher.reload().await {
        Ok(new_config) => {
            let sanitized = SanitizedConfig::from_app_config(&new_config);
            (StatusCode::OK, Json(ConfigReloadResponse::success(sanitized)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConfigReloadResponse::failure(format!(
                "Failed to reload configuration: {}",
                e
            ))),
        ),
    }
}

/// Update a specific configuration section
///
/// Updates a configuration section by merging the provided values
/// with the existing configuration, then writing to the config file
/// and triggering a reload.
///
/// Valid sections: server, logging, cache, metrics, limits, resilience
pub async fn update_config_section(
    State(state): State<AdminState>,
    Path(section): Path<String>,
    Json(request): Json<ConfigSectionUpdateRequest>,
) -> impl IntoResponse {
    use super::config::is_valid_section;

    // Check if section is valid
    if !is_valid_section(&section) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ConfigSectionUpdateResponse::invalid_section(&section)),
        );
    }

    // Check if watcher is available
    let Some(watcher) = &state.config_watcher else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ConfigSectionUpdateResponse::watcher_unavailable(&section)),
        );
    };

    // Get config path
    let Some(config_path) = &state.config_path else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ConfigSectionUpdateResponse::failure(
                &section,
                "Configuration file path not available",
            )),
        );
    };

    // Read current config file
    let config_content = match std::fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConfigSectionUpdateResponse::failure(
                    &section,
                    format!("Failed to read configuration file: {}", e),
                )),
            );
        }
    };

    // Parse as TOML
    let mut config_value: toml::Value = match toml::from_str(&config_content) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConfigSectionUpdateResponse::failure(
                    &section,
                    format!("Failed to parse configuration file: {}", e),
                )),
            );
        }
    };

    // Update the section
    if let Some(table) = config_value.as_table_mut() {
        // Convert JSON value to TOML value
        let toml_value = match json_to_toml(&request.values) {
            Some(v) => v,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ConfigSectionUpdateResponse::failure(
                        &section,
                        "Invalid configuration value format",
                    )),
                );
            }
        };

        // Merge with existing section or create new
        if let Some(existing) = table.get_mut(&section) {
            if let (Some(existing_table), Some(new_table)) =
                (existing.as_table_mut(), toml_value.as_table())
            {
                for (key, value) in new_table {
                    existing_table.insert(key.clone(), value.clone());
                }
            } else {
                table.insert(section.clone(), toml_value);
            }
        } else {
            table.insert(section.clone(), toml_value);
        }
    }

    // Write back to file
    let updated_content = match toml::to_string_pretty(&config_value) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConfigSectionUpdateResponse::failure(
                    &section,
                    format!("Failed to serialize configuration: {}", e),
                )),
            );
        }
    };

    if let Err(e) = std::fs::write(config_path, updated_content) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConfigSectionUpdateResponse::failure(
                &section,
                format!("Failed to write configuration file: {}", e),
            )),
        );
    }

    // Reload configuration
    match watcher.reload().await {
        Ok(new_config) => {
            let sanitized = SanitizedConfig::from_app_config(&new_config);
            (
                StatusCode::OK,
                Json(ConfigSectionUpdateResponse::success(&section, sanitized)),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConfigSectionUpdateResponse::failure(
                &section,
                format!("Configuration updated but reload failed: {}", e),
            )),
        ),
    }
}

/// Convert a JSON value to a TOML value
fn json_to_toml(json: &serde_json::Value) -> Option<toml::Value> {
    match json {
        serde_json::Value::Null => Some(toml::Value::String(String::new())),
        serde_json::Value::Bool(b) => Some(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Some(toml::Value::Float(f))
            } else {
                None
            }
        }
        serde_json::Value::String(s) => Some(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let toml_arr: Option<Vec<toml::Value>> = arr.iter().map(json_to_toml).collect();
            toml_arr.map(toml::Value::Array)
        }
        serde_json::Value::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (k, v) in obj {
                table.insert(k.clone(), json_to_toml(v)?);
            }
            Some(toml::Value::Table(table))
        }
    }
}
