//! Configuration management handlers

use super::common::*;
use super::HistoryQuery;
use crate::admin::config_keys;
use crate::infrastructure::utils::IntoStatusCode;

/// Get system configuration (legacy endpoint)
pub async fn get_config_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<SystemConfig>>, StatusCode> {
    let config_data = state.admin_service.get_configuration().await.to_500()?;

    let config = SystemConfig {
        providers: config_data.providers,
        indexing: crate::admin::models::IndexingConfig {
            chunk_size: config_data.indexing.chunk_size,
            chunk_overlap: config_data.indexing.chunk_overlap,
            max_file_size: config_data.indexing.max_file_size,
            supported_extensions: config_data.indexing.supported_extensions,
            exclude_patterns: config_data.indexing.exclude_patterns,
        },
        security: crate::admin::models::SecurityConfig {
            enable_auth: config_data.security.enable_auth,
            rate_limiting: config_data.security.rate_limiting,
            max_requests_per_minute: config_data.security.max_requests_per_minute,
        },
        metrics: crate::admin::models::MetricsConfig {
            enabled: config_data.metrics.enabled,
            collection_interval: config_data.metrics.collection_interval,
            retention_days: config_data.metrics.retention_days,
        },
    };

    Ok(Json(ApiResponse::success(config)))
}

/// Update system configuration (legacy endpoint)
pub async fn update_config_handler(
    State(state): State<AdminState>,
    Json(config): Json<SystemConfig>,
) -> Result<Json<ApiResponse<SystemConfig>>, StatusCode> {
    let mut updates = std::collections::HashMap::new();

    // Indexing settings
    updates.insert(
        config_keys::indexing::CHUNK_SIZE.to_string(),
        serde_json::json!(config.indexing.chunk_size),
    );
    updates.insert(
        config_keys::indexing::CHUNK_OVERLAP.to_string(),
        serde_json::json!(config.indexing.chunk_overlap),
    );
    updates.insert(
        config_keys::indexing::MAX_FILE_SIZE.to_string(),
        serde_json::json!(config.indexing.max_file_size),
    );
    updates.insert(
        config_keys::indexing::SUPPORTED_EXTENSIONS.to_string(),
        serde_json::json!(config.indexing.supported_extensions),
    );
    updates.insert(
        config_keys::indexing::EXCLUDE_PATTERNS.to_string(),
        serde_json::json!(config.indexing.exclude_patterns),
    );

    // Security settings
    updates.insert(
        config_keys::security::ENABLE_AUTH.to_string(),
        serde_json::json!(config.security.enable_auth),
    );
    updates.insert(
        config_keys::security::RATE_LIMITING.to_string(),
        serde_json::json!(config.security.rate_limiting),
    );
    updates.insert(
        config_keys::security::MAX_REQUESTS_PER_MINUTE.to_string(),
        serde_json::json!(config.security.max_requests_per_minute),
    );

    // Metrics settings
    updates.insert(
        config_keys::metrics::ENABLED.to_string(),
        serde_json::json!(config.metrics.enabled),
    );
    updates.insert(
        config_keys::metrics::COLLECTION_INTERVAL.to_string(),
        serde_json::json!(config.metrics.collection_interval),
    );
    updates.insert(
        config_keys::metrics::RETENTION_DAYS.to_string(),
        serde_json::json!(config.metrics.retention_days),
    );

    match state
        .admin_service
        .update_configuration(updates, "admin")
        .await
    {
        Ok(_result) => {
            let updated_config = state.admin_service.get_configuration().await.to_500()?;

            let response_config = SystemConfig {
                providers: updated_config.providers,
                indexing: crate::admin::models::IndexingConfig {
                    chunk_size: updated_config.indexing.chunk_size,
                    chunk_overlap: updated_config.indexing.chunk_overlap,
                    max_file_size: updated_config.indexing.max_file_size,
                    supported_extensions: updated_config.indexing.supported_extensions,
                    exclude_patterns: updated_config.indexing.exclude_patterns,
                },
                security: crate::admin::models::SecurityConfig {
                    enable_auth: updated_config.security.enable_auth,
                    rate_limiting: updated_config.security.rate_limiting,
                    max_requests_per_minute: updated_config.security.max_requests_per_minute,
                },
                metrics: crate::admin::models::MetricsConfig {
                    enabled: updated_config.metrics.enabled,
                    collection_interval: updated_config.metrics.collection_interval,
                    retention_days: updated_config.metrics.retention_days,
                },
            };

            Ok(Json(ApiResponse::success(response_config)))
        }
        Err(e) => Ok(Json(ApiResponse::error(format!(
            "Failed to update configuration: {}",
            e
        )))),
    }
}

/// Get current system configuration
pub async fn get_configuration_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::ConfigurationData>>, StatusCode> {
    let config = state.admin_service.get_configuration().await.to_500()?;

    Ok(Json(ApiResponse::success(config)))
}

/// Update system configuration
pub async fn update_configuration_handler(
    State(state): State<AdminState>,
    Extension(claims): Extension<crate::admin::auth::Claims>,
    Json(updates): Json<std::collections::HashMap<String, serde_json::Value>>,
) -> Result<Json<ApiResponse<crate::admin::service::ConfigurationUpdateResult>>, StatusCode> {
    let user = &claims.sub;

    let result = state
        .admin_service
        .update_configuration(updates, user)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

/// Validate configuration changes
pub async fn validate_configuration_handler(
    State(state): State<AdminState>,
    Json(updates): Json<std::collections::HashMap<String, serde_json::Value>>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    let warnings = state
        .admin_service
        .validate_configuration(&updates)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(warnings)))
}

/// Get configuration change history
pub async fn get_configuration_history_handler(
    State(state): State<AdminState>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<ApiResponse<Vec<crate::admin::service::ConfigurationChange>>>, StatusCode> {
    let history = state
        .admin_service
        .get_configuration_history(params.limit)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(history)))
}

/// Persist current runtime configuration to file
pub async fn persist_configuration_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::ConfigPersistResult>>, StatusCode> {
    let result = state.admin_service.persist_configuration().await.to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

/// Get difference between runtime and file configuration
pub async fn get_config_diff_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::ConfigDiff>>, StatusCode> {
    let diff = state.admin_service.get_config_diff().await.to_500()?;

    Ok(Json(ApiResponse::success(diff)))
}
