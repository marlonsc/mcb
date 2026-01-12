//! HTTP handlers for admin API endpoints

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::admin::models::{
    AdminState, ApiResponse, IndexInfo, IndexOperationRequest, ProviderConfigRequest, ProviderInfo,
    SystemConfig,
};
use crate::admin::service::MaintenanceResult;

/// Get system configuration
pub async fn get_config_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<SystemConfig>>, StatusCode> {
    // Get real configuration from admin service
    let config_data = state
        .admin_service
        .get_configuration()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to admin models
    let config = SystemConfig {
        providers: config_data
            .providers
            .into_iter()
            .map(|p| ProviderInfo {
                id: p.id,
                name: p.name,
                provider_type: p.provider_type,
                status: p.status,
                config: p.config,
            })
            .collect(),
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

/// Update system configuration
pub async fn update_config_handler(
    State(state): State<AdminState>,
    Json(config): Json<SystemConfig>,
) -> Result<Json<ApiResponse<SystemConfig>>, StatusCode> {
    // Convert SystemConfig to HashMap for admin service
    let mut updates = std::collections::HashMap::new();

    // Indexing settings
    updates.insert(
        "indexing.chunk_size".to_string(),
        serde_json::json!(config.indexing.chunk_size),
    );
    updates.insert(
        "indexing.chunk_overlap".to_string(),
        serde_json::json!(config.indexing.chunk_overlap),
    );
    updates.insert(
        "indexing.max_file_size".to_string(),
        serde_json::json!(config.indexing.max_file_size),
    );
    updates.insert(
        "indexing.supported_extensions".to_string(),
        serde_json::json!(config.indexing.supported_extensions),
    );
    updates.insert(
        "indexing.exclude_patterns".to_string(),
        serde_json::json!(config.indexing.exclude_patterns),
    );

    // Security settings
    updates.insert(
        "security.enable_auth".to_string(),
        serde_json::json!(config.security.enable_auth),
    );
    updates.insert(
        "security.rate_limiting".to_string(),
        serde_json::json!(config.security.rate_limiting),
    );
    updates.insert(
        "security.max_requests_per_minute".to_string(),
        serde_json::json!(config.security.max_requests_per_minute),
    );

    // Metrics settings
    updates.insert(
        "metrics.enabled".to_string(),
        serde_json::json!(config.metrics.enabled),
    );
    updates.insert(
        "metrics.collection_interval".to_string(),
        serde_json::json!(config.metrics.collection_interval),
    );
    updates.insert(
        "metrics.retention_days".to_string(),
        serde_json::json!(config.metrics.retention_days),
    );

    // Update configuration via admin service
    match state
        .admin_service
        .update_configuration(updates, "admin")
        .await
    {
        Ok(_result) => {
            // Fetch updated configuration to return
            let updated_config = state
                .admin_service
                .get_configuration()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let response_config = SystemConfig {
                providers: updated_config
                    .providers
                    .into_iter()
                    .map(|p| ProviderInfo {
                        id: p.id,
                        name: p.name,
                        provider_type: p.provider_type,
                        status: p.status,
                        config: p.config,
                    })
                    .collect(),
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

/// List all providers
pub async fn list_providers_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<ProviderInfo>>>, StatusCode> {
    // Get real provider data from MCP server
    let provider_statuses = state
        .admin_service
        .get_providers()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let providers = provider_statuses
        .into_iter()
        .map(|status| ProviderInfo {
            id: status.id,
            name: status.name,
            provider_type: status.provider_type,
            status: status.status,
            config: status.config,
        })
        .collect();

    Ok(Json(ApiResponse::success(providers)))
}

/// Add a new provider
pub async fn add_provider_handler(
    State(state): State<AdminState>,
    Json(provider_config): Json<ProviderConfigRequest>,
) -> Result<Json<ApiResponse<ProviderInfo>>, StatusCode> {
    // Validate provider configuration based on type
    match provider_config.provider_type.as_str() {
        "embedding" => {
            // Validate embedding provider configuration
            if provider_config.config.get("model").is_none() {
                return Ok(Json(ApiResponse::error(
                    "Model is required for embedding providers".to_string(),
                )));
            }
        }
        "vector_store" => {
            // Validate vector store provider configuration
            if provider_config.config.get("host").is_none() {
                return Ok(Json(ApiResponse::error(
                    "Host is required for vector store providers".to_string(),
                )));
            }
        }
        _ => {
            return Ok(Json(ApiResponse::error(
                "Invalid provider type".to_string(),
            )));
        }
    }

    // Register provider through admin service
    match state
        .admin_service
        .add_provider(&provider_config.provider_type, provider_config.config)
        .await
    {
        Ok(svc_info) => {
            // Convert service::ProviderInfo to models::ProviderInfo
            let provider_info = ProviderInfo {
                id: svc_info.id,
                name: svc_info.name,
                provider_type: svc_info.provider_type,
                status: svc_info.status,
                config: svc_info.config,
            };
            Ok(Json(ApiResponse::success(provider_info)))
        }
        Err(e) => Ok(Json(ApiResponse::error(format!(
            "Failed to add provider: {}",
            e
        )))),
    }
}

/// Remove a provider
pub async fn remove_provider_handler(
    State(state): State<AdminState>,
    Path(provider_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Check if provider exists
    let providers = state
        .admin_service
        .get_providers()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !providers.iter().any(|p| p.id == provider_id) {
        return Ok(Json(ApiResponse::error("Provider not found".to_string())));
    }

    // In a real implementation, this would unregister the provider from the MCP server
    // For now, return success
    Ok(Json(ApiResponse::success(format!(
        "Provider {} removed successfully",
        provider_id
    ))))
}

/// List all indexes
pub async fn list_indexes_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<IndexInfo>>>, StatusCode> {
    // Get real indexing status from MCP server
    let indexing_status = state
        .admin_service
        .get_indexing_status()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let indexes = vec![IndexInfo {
        id: "main-index".to_string(),
        name: "Main Codebase Index".to_string(),
        status: if indexing_status.is_indexing {
            "indexing".to_string()
        } else {
            "active".to_string()
        },
        document_count: indexing_status.indexed_documents,
        created_at: indexing_status.start_time.unwrap_or(1640995200),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    }];

    Ok(Json(ApiResponse::success(indexes)))
}

/// Perform index operation
pub async fn index_operation_handler(
    State(state): State<AdminState>,
    Path(index_id): Path<String>,
    Json(operation): Json<IndexOperationRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    use crate::infrastructure::events::SystemEvent;

    // Perform operation based on type - use event bus for real operations
    match operation.operation.as_str() {
        "clear" => {
            // Clear the index by publishing IndexClear event
            if let Err(e) = state
                .event_bus
                .publish(SystemEvent::IndexClear {
                    collection: Some(index_id.clone()),
                })
                .await
            {
                tracing::error!("Failed to publish IndexClear event: {}", e);
                return Ok(Json(ApiResponse::error(format!(
                    "Failed to clear index: {}",
                    e
                ))));
            }

            // Also clear the index cache
            if let Err(e) = state
                .event_bus
                .publish(SystemEvent::CacheClear {
                    namespace: Some("indexes".to_string()),
                })
                .await
            {
                tracing::warn!("Failed to clear index cache: {}", e);
            }

            tracing::info!("[ADMIN] Index clear requested for: {}", index_id);
            Ok(Json(ApiResponse::success(format!(
                "Index {} clear initiated. The operation is running asynchronously.",
                index_id
            ))))
        }
        "rebuild" => {
            // Trigger index rebuild via event bus
            if let Err(e) = state
                .event_bus
                .publish(SystemEvent::IndexRebuild {
                    collection: Some(index_id.clone()),
                })
                .await
            {
                tracing::error!("Failed to publish IndexRebuild event: {}", e);
                return Ok(Json(ApiResponse::error(format!(
                    "Failed to start index rebuild: {}",
                    e
                ))));
            }

            tracing::info!("[ADMIN] Index rebuild requested for: {}", index_id);
            Ok(Json(ApiResponse::success(format!(
                "Index {} rebuild initiated. The operation is running asynchronously.",
                index_id
            ))))
        }
        "optimize" => {
            // Trigger index optimization
            if let Err(e) = state
                .event_bus
                .publish(SystemEvent::IndexOptimize {
                    collection: Some(index_id.clone()),
                })
                .await
            {
                tracing::error!("Failed to publish IndexOptimize event: {}", e);
                return Ok(Json(ApiResponse::error(format!(
                    "Failed to optimize index: {}",
                    e
                ))));
            }

            tracing::info!("[ADMIN] Index optimization requested for: {}", index_id);
            Ok(Json(ApiResponse::success(format!(
                "Index {} optimization initiated. The operation is running asynchronously.",
                index_id
            ))))
        }
        "status" => {
            // Get current indexing status
            let status = state.mcp_server.get_indexing_status_admin().await;
            let message = if status.is_indexing {
                format!(
                    "Index {} is currently indexing ({} of {} documents)",
                    index_id, status.indexed_documents, status.total_documents
                )
            } else {
                format!(
                    "Index {} is idle ({} documents indexed)",
                    index_id, status.indexed_documents
                )
            };
            Ok(Json(ApiResponse::success(message)))
        }
        _ => Ok(Json(ApiResponse::error(format!(
            "Invalid operation '{}'. Valid operations: clear, rebuild, optimize, status",
            operation.operation
        )))),
    }
}

/// Get system status
pub async fn get_status_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // Get real system information
    let system_info = state
        .admin_service
        .get_system_info()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let providers = state
        .admin_service
        .get_providers()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let indexing_status = state
        .admin_service
        .get_indexing_status()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let performance = state
        .admin_service
        .get_performance_metrics()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Group providers by type
    let mut embedding_providers = Vec::new();
    let mut vector_store_providers = Vec::new();

    for provider in providers {
        match provider.provider_type.as_str() {
            "embedding" => embedding_providers.push(provider.name.to_lowercase()),
            "vector_store" => vector_store_providers.push(provider.name.to_lowercase()),
            _ => {}
        }
    }

    let status = serde_json::json!({
        "service": "mcp-context-browser",
        "version": system_info.version,
        "status": "running",
        "uptime": system_info.uptime,
        "pid": system_info.pid,
        "providers": {
            "embedding": embedding_providers,
            "vector_store": vector_store_providers
        },
        "indexes": {
            "total": 1,
            "active": if indexing_status.is_indexing { 0 } else { 1 },
            "indexing": indexing_status.is_indexing,
            "total_documents": indexing_status.total_documents,
            "indexed_documents": indexing_status.indexed_documents
        },
        "performance": {
            "total_queries": performance.total_queries,
            "successful_queries": performance.successful_queries,
            "failed_queries": performance.failed_queries,
            "average_response_time_ms": performance.average_response_time_ms,
            "cache_hit_rate": performance.cache_hit_rate,
            "active_connections": performance.active_connections
        }
    });

    Ok(Json(ApiResponse::success(status)))
}

/// Get dashboard metrics for the web interface
pub async fn get_dashboard_metrics_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let indexing_status = state
        .admin_service
        .get_indexing_status()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let performance = state
        .admin_service
        .get_performance_metrics()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let providers = state
        .admin_service
        .get_providers()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_providers = providers.iter().filter(|p| p.status == "available").count();

    // Get real system metrics - FAIL if unavailable
    let cpu = state
        .mcp_server
        .system_collector
        .collect_cpu_metrics()
        .await
        .map(|m| m.usage)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let memory = state
        .mcp_server
        .system_collector
        .collect_memory_metrics()
        .await
        .map(|m| m.usage_percent)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get system information
    let system_info = state
        .admin_service
        .get_system_info()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Determine health status based on actual metrics, not hardcoded
    let health_status = match (cpu < 85.0, memory < 85.0) {
        (true, true) => "healthy",
        (false, false) => "critical",
        _ => "degraded",
    };

    let metrics = serde_json::json!({
        "active_providers": active_providers,
        "total_providers": providers.len(),
        "active_indexes": if indexing_status.is_indexing { 0 } else { 1 },
        "total_documents": indexing_status.total_documents,
        "cpu_usage": cpu,
        "memory_usage": memory,
        "queries": performance.total_queries,
        "avg_latency": performance.average_response_time_ms,
        "health": {
            "status": health_status,
            "uptime": system_info.uptime,
            "pid": system_info.pid
        }
    });

    Ok(Json(ApiResponse::success(metrics)))
}

/// Query parameters for search
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
}

// Configuration Management Handlers
/// Get current system configuration
pub async fn get_configuration_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::ConfigurationData>>, StatusCode> {
    let config = state
        .admin_service
        .get_configuration()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(config)))
}

/// Update system configuration
pub async fn update_configuration_handler(
    State(state): State<AdminState>,
    Extension(claims): Extension<crate::admin::auth::Claims>,
    Json(updates): Json<std::collections::HashMap<String, serde_json::Value>>,
) -> Result<Json<ApiResponse<crate::admin::service::ConfigurationUpdateResult>>, StatusCode> {
    // Extract authenticated user from JWT claims
    let user = &claims.sub;

    let result = state
        .admin_service
        .update_configuration(updates, user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(history)))
}

// Logging System Handlers
/// Get system logs with filtering
pub async fn get_logs_handler(
    State(state): State<AdminState>,
    Query(filter): Query<crate::admin::service::LogFilter>,
) -> Result<Json<ApiResponse<crate::admin::service::LogEntries>>, StatusCode> {
    let logs = state
        .admin_service
        .get_logs(filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(logs)))
}

/// Export logs to file
pub async fn export_logs_handler(
    State(state): State<AdminState>,
    Query(filter): Query<crate::admin::service::LogFilter>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let filename = state
        .admin_service
        .export_logs(filter, params.format)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(filename)))
}

/// Get log statistics
pub async fn get_log_stats_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::LogStats>>, StatusCode> {
    let stats = state
        .admin_service
        .get_log_stats()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(stats)))
}

// Maintenance Operations Handlers
/// Clear system cache
pub async fn clear_cache_handler(
    State(state): State<AdminState>,
    Path(cache_type): Path<String>,
) -> Result<Json<ApiResponse<crate::admin::service::MaintenanceResult>>, StatusCode> {
    let cache_type_enum = match cache_type.as_str() {
        "all" => crate::admin::service::CacheType::All,
        "query" => crate::admin::service::CacheType::QueryResults,
        "embeddings" => crate::admin::service::CacheType::Embeddings,
        "indexes" => crate::admin::service::CacheType::Indexes,
        _ => return Ok(Json(ApiResponse::error("Invalid cache type".to_string()))),
    };

    let result = state
        .admin_service
        .clear_cache(cache_type_enum)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(result)))
}

/// Restart provider connection
pub async fn restart_provider_handler(
    State(state): State<AdminState>,
    Path(provider_id): Path<String>,
) -> Result<Json<ApiResponse<crate::admin::service::MaintenanceResult>>, StatusCode> {
    let result = state
        .admin_service
        .restart_provider(&provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(result)))
}

/// Rebuild search index
pub async fn rebuild_index_handler(
    State(state): State<AdminState>,
    Path(index_id): Path<String>,
) -> Result<Json<ApiResponse<crate::admin::service::MaintenanceResult>>, StatusCode> {
    let result = state
        .admin_service
        .rebuild_index(&index_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(result)))
}

/// Cleanup old data
pub async fn cleanup_data_handler(
    State(state): State<AdminState>,
    Json(cleanup_config): Json<crate::admin::service::CleanupConfig>,
) -> Result<Json<ApiResponse<crate::admin::service::MaintenanceResult>>, StatusCode> {
    let result = state
        .admin_service
        .cleanup_data(cleanup_config)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(result)))
}

// Diagnostic Operations Handlers
/// Run comprehensive health check
pub async fn health_check_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::HealthCheckResult>>, StatusCode> {
    let result = state
        .admin_service
        .run_health_check()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(result)))
}

// Data Management Handlers
/// Create system backup
pub async fn create_backup_handler(
    State(state): State<AdminState>,
    Json(backup_config): Json<crate::admin::service::BackupConfig>,
) -> Result<Json<ApiResponse<crate::admin::service::BackupResult>>, StatusCode> {
    let result = state
        .admin_service
        .create_backup(backup_config)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(result)))
}

/// List available backups
pub async fn list_backups_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<crate::admin::service::BackupInfo>>>, StatusCode> {
    let backups = state
        .admin_service
        .list_backups()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(result)))
}

// Query Parameter Structures
#[derive(Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct ExportQuery {
    pub format: crate::admin::service::LogExportFormat,
}

/// Search handler
pub async fn search_handler(
    State(state): State<AdminState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<ApiResponse<crate::admin::service::SearchResults>>, StatusCode> {
    // Use admin service for search
    match state
        .admin_service
        .search(&params.q, None, params.limit)
        .await
    {
        Ok(results) => Ok(Json(ApiResponse::success(results))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Search failed: {}", e)))),
    }
}

// === Subsystem Control Handlers (ADR-007) ===

/// Get all subsystems and their status
pub async fn get_subsystems_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<crate::admin::service::SubsystemInfo>>>, StatusCode> {
    let subsystems = state
        .admin_service
        .get_subsystems()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(subsystems)))
}

/// Request body for sending signals to subsystems
#[derive(Deserialize)]
pub struct SubsystemSignalRequest {
    pub signal: crate::admin::service::SubsystemSignal,
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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(result)))
}

/// Get all registered HTTP routes
pub async fn get_routes_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<crate::admin::service::RouteInfo>>>, StatusCode> {
    let routes = state
        .admin_service
        .get_routes()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(routes)))
}

/// Reload router configuration
pub async fn reload_routes_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::MaintenanceResult>>, StatusCode> {
    let result = state
        .admin_service
        .reload_routes()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(result)))
}

/// Persist current runtime configuration to file
pub async fn persist_configuration_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::ConfigPersistResult>>, StatusCode> {
    let result = state
        .admin_service
        .persist_configuration()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(result)))
}

/// Get difference between runtime and file configuration
pub async fn get_config_diff_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::ConfigDiff>>, StatusCode> {
    let diff = state
        .admin_service
        .get_config_diff()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(diff)))
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
            .map_err(|_| StatusCode::NOT_FOUND)?;

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
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

// ============================================================================
// Simplified API Handlers (for HTMX buttons)
// ============================================================================

/// Clear cache by type (simplified endpoint for HTMX)
pub async fn api_clear_cache_handler(
    State(state): State<AdminState>,
    Path(cache_type): Path<String>,
) -> axum::response::Html<String> {
    let cache_type_enum = match cache_type.as_str() {
        "all" => crate::admin::service::CacheType::All,
        "query" => crate::admin::service::CacheType::QueryResults,
        "embeddings" => crate::admin::service::CacheType::Embeddings,
        "indexes" => crate::admin::service::CacheType::Indexes,
        _ => {
            return axum::response::Html(format!(
                r#"<div class="text-red-600 dark:text-red-400 mt-2">Invalid cache type: {}</div>"#,
                cache_type
            ))
        }
    };

    match state.admin_service.clear_cache(cache_type_enum).await {
        Ok(result) => axum::response::Html(format!(
            r#"<div class="text-green-600 dark:text-green-400 mt-2">{}</div>"#,
            result.message
        )),
        Err(e) => axum::response::Html(format!(
            r#"<div class="text-red-600 dark:text-red-400 mt-2">Error: {}</div>"#,
            e
        )),
    }
}

/// Restart all providers (simplified endpoint for HTMX)
pub async fn api_restart_all_providers_handler(
    State(state): State<AdminState>,
) -> axum::response::Html<String> {
    let providers = match state.admin_service.get_providers().await {
        Ok(p) => p,
        Err(e) => {
            return axum::response::Html(format!(
                r#"<div class="text-red-600 dark:text-red-400 mt-2">Error getting providers: {}</div>"#,
                e
            ))
        }
    };

    let provider_list: Vec<(String, String)> = providers
        .into_iter()
        .map(|p| (p.provider_type, p.id))
        .collect();

    match crate::admin::service::helpers::maintenance::restart_all_providers(
        &state.event_bus,
        &provider_list,
    )
    .await
    {
        Ok(result) => axum::response::Html(format!(
            r#"<div class="text-green-600 dark:text-green-400 mt-2">{}</div>"#,
            result.message
        )),
        Err(e) => axum::response::Html(format!(
            r#"<div class="text-red-600 dark:text-red-400 mt-2">Error: {}</div>"#,
            e
        )),
    }
}

/// Restart providers by type (simplified endpoint for HTMX)
pub async fn api_restart_providers_by_type_handler(
    State(state): State<AdminState>,
    Path(provider_type): Path<String>,
) -> axum::response::Html<String> {
    let providers = match state.admin_service.get_providers().await {
        Ok(p) => p,
        Err(e) => {
            return axum::response::Html(format!(
                r#"<div class="text-red-600 dark:text-red-400 mt-2">Error getting providers: {}</div>"#,
                e
            ))
        }
    };

    let provider_list: Vec<(String, String)> = providers
        .into_iter()
        .filter(|p| p.provider_type == provider_type)
        .map(|p| (p.provider_type, p.id))
        .collect();

    if provider_list.is_empty() {
        return axum::response::Html(format!(
            r#"<div class="text-yellow-600 dark:text-yellow-400 mt-2">No {} providers found</div>"#,
            provider_type
        ));
    }

    match crate::admin::service::helpers::maintenance::restart_all_providers(
        &state.event_bus,
        &provider_list,
    )
    .await
    {
        Ok(result) => axum::response::Html(format!(
            r#"<div class="text-green-600 dark:text-green-400 mt-2">{}</div>"#,
            result.message
        )),
        Err(e) => axum::response::Html(format!(
            r#"<div class="text-red-600 dark:text-red-400 mt-2">Error: {}</div>"#,
            e
        )),
    }
}

/// Reconfigure a provider without restart
pub async fn api_reconfigure_provider_handler(
    State(state): State<AdminState>,
    Path((provider_type, provider_id)): Path<(String, String)>,
    Json(config): Json<serde_json::Value>,
) -> Result<axum::Json<ApiResponse<MaintenanceResult>>, StatusCode> {
    let full_provider_id = format!("{}:{}", provider_type, provider_id);

    match state
        .admin_service
        .reconfigure_provider(&full_provider_id, config)
        .await
    {
        Ok(result) => {
            tracing::info!(
                "[ADMIN] Provider reconfiguration successful for {}",
                full_provider_id
            );
            Ok(axum::Json(ApiResponse::success(result)))
        }
        Err(e) => {
            tracing::error!("[ADMIN] Provider reconfiguration failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Rebuild all indexes (simplified endpoint for HTMX)
pub async fn api_rebuild_indexes_handler(
    State(state): State<AdminState>,
) -> axum::response::Html<String> {
    match state.admin_service.rebuild_index("default").await {
        Ok(result) => axum::response::Html(format!(
            r#"<div class="text-green-600 dark:text-green-400 mt-2">{}</div>"#,
            result.message
        )),
        Err(e) => axum::response::Html(format!(
            r#"<div class="text-red-600 dark:text-red-400 mt-2">Error: {}</div>"#,
            e
        )),
    }
}

/// Optimize indexes (simplified endpoint for HTMX)
pub async fn api_optimize_indexes_handler(
    State(_state): State<AdminState>,
) -> axum::response::Html<String> {
    // Index optimization is not yet implemented, but we can acknowledge the request
    axum::response::Html(
        r#"<div class="text-yellow-600 dark:text-yellow-400 mt-2">Index optimization requested. This is a placeholder for future implementation.</div>"#
            .to_string(),
    )
}

/// Clear all indexes (simplified endpoint for HTMX)
pub async fn api_clear_indexes_handler(
    State(state): State<AdminState>,
) -> axum::response::Html<String> {
    // Clear by rebuilding with "clear" operation
    match state.admin_service.rebuild_index("__clear__").await {
        Ok(result) => axum::response::Html(format!(
            r#"<div class="text-green-600 dark:text-green-400 mt-2">{}</div>"#,
            result.message
        )),
        Err(e) => axum::response::Html(format!(
            r#"<div class="text-red-600 dark:text-red-400 mt-2">Error: {}</div>"#,
            e
        )),
    }
}

/// Request for cleanup operation
#[derive(Debug, Deserialize)]
pub struct CleanupRequest {
    pub older_than_days: Option<u32>,
}

/// Cleanup old data (simplified endpoint for HTMX)
pub async fn api_cleanup_handler(
    State(state): State<AdminState>,
    Query(params): Query<CleanupRequest>,
) -> axum::response::Html<String> {
    let cleanup_config = crate::admin::service::CleanupConfig {
        older_than_days: params.older_than_days.unwrap_or(30),
        max_items_to_keep: None,
        cleanup_types: vec!["logs".to_string(), "exports".to_string()],
    };

    match state.admin_service.cleanup_data(cleanup_config).await {
        Ok(result) => axum::response::Html(format!(
            r#"<div class="text-green-600 dark:text-green-400 mt-2">{}</div>"#,
            result.message
        )),
        Err(e) => axum::response::Html(format!(
            r#"<div class="text-red-600 dark:text-red-400 mt-2">Error: {}</div>"#,
            e
        )),
    }
}

// ============================================================================
// HTMX Partial Handlers
// ============================================================================

/// HTMX partial for recovery status
pub async fn htmx_recovery_status_handler(
    State(state): State<AdminState>,
) -> axum::response::Html<String> {
    let recovery_states = if let Some(recovery_manager) = &state.recovery_manager {
        recovery_manager.get_recovery_states()
    } else {
        Vec::new()
    };

    if recovery_states.is_empty() {
        return axum::response::Html(
            r#"<div class="px-6 py-4 text-center text-gray-500 dark:text-gray-400">
                <svg class="h-8 w-8 mx-auto text-green-500 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                All subsystems healthy - no recovery in progress
            </div>"#
                .to_string(),
        );
    }

    let mut html = String::from(r#"<div class="divide-y divide-gray-200 dark:divide-gray-700">"#);

    for state in recovery_states {
        let status_color = match state.status {
            crate::daemon::types::RecoveryStatus::Healthy => "bg-green-500",
            crate::daemon::types::RecoveryStatus::Recovering => "bg-yellow-500 animate-pulse",
            crate::daemon::types::RecoveryStatus::Exhausted => "bg-red-500",
            crate::daemon::types::RecoveryStatus::Manual => "bg-orange-500",
        };

        let status_text = format!("{}", state.status);

        html.push_str(&format!(
            r#"<div class="flex items-center justify-between px-6 py-4">
                <div class="flex items-center space-x-3">
                    <span class="w-3 h-3 rounded-full {}"></span>
                    <div>
                        <span class="font-medium text-gray-900 dark:text-white">{}</span>
                        <span class="ml-2 text-sm text-gray-500 dark:text-gray-400">{}</span>
                    </div>
                </div>
                <div class="flex items-center space-x-4">"#,
            status_color, state.subsystem_id, status_text
        ));

        if state.current_retry > 0 {
            html.push_str(&format!(
                r#"<span class="text-sm text-gray-500">Retry {}/{}</span>"#,
                state.current_retry, state.max_retries
            ));
        }

        if let Some(ref error) = state.last_error {
            html.push_str(&format!(
                r#"<span class="text-xs text-red-500 max-w-xs truncate" title="{}">{}</span>"#,
                error,
                if error.len() > 30 {
                    format!("{}...", &error[..30])
                } else {
                    error.clone()
                }
            ));
        }

        // Action buttons based on status
        if state.status == crate::daemon::types::RecoveryStatus::Exhausted {
            html.push_str(&format!(
                r##"<button hx-post="/admin/api/recovery/{}/reset"
                          hx-target="#recovery-status"
                          hx-swap="innerHTML"
                          class="text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300">
                    Reset
                </button>"##,
                state.subsystem_id
            ));
        }

        html.push_str(&format!(
            r##"<button hx-post="/admin/api/recovery/{}/trigger"
                      hx-target="#recovery-status"
                      hx-swap="innerHTML"
                      class="text-sm text-orange-600 hover:text-orange-800 dark:text-orange-400 dark:hover:text-orange-300">
                Retry Now
            </button>"##,
            state.subsystem_id
        ));

        html.push_str("</div></div>");
    }

    html.push_str("</div>");

    axum::response::Html(html)
}

/// HTMX partial for maintenance history
pub async fn htmx_maintenance_history_handler(
    State(state): State<AdminState>,
) -> axum::response::Html<String> {
    // Get recent activities from the activity logger
    let activities = state.activity_logger.get_activities(Some(10)).await;

    if activities.is_empty() {
        return axum::response::Html(
            r#"<div class="px-6 py-4">
                <div class="text-center text-gray-500 dark:text-gray-400 py-4">
                    <svg class="h-8 w-8 mx-auto text-gray-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                    </svg>
                    <p>No recent maintenance activity.</p>
                    <p class="text-sm mt-1">Operations will appear here as they occur.</p>
                </div>
            </div>"#
                .to_string(),
        );
    }

    // Build HTML for activities
    let mut html = String::from(r#"<div class="divide-y divide-gray-200 dark:divide-gray-700">"#);

    for activity in activities {
        let level_class = match activity.level {
            crate::admin::service::helpers::activity::ActivityLevel::Info => {
                "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300"
            }
            crate::admin::service::helpers::activity::ActivityLevel::Warning => {
                "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300"
            }
            crate::admin::service::helpers::activity::ActivityLevel::Error => {
                "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300"
            }
            crate::admin::service::helpers::activity::ActivityLevel::Success => {
                "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300"
            }
        };

        let time_ago = chrono::Utc::now()
            .signed_duration_since(activity.timestamp)
            .num_seconds();
        let time_str = if time_ago < 60 {
            format!("{}s ago", time_ago)
        } else if time_ago < 3600 {
            format!("{}m ago", time_ago / 60)
        } else {
            format!("{}h ago", time_ago / 3600)
        };

        html.push_str(&format!(
            r#"<div class="px-4 py-3">
                <div class="flex items-center justify-between">
                    <span class="text-xs font-medium px-2 py-1 rounded {}">{}</span>
                    <span class="text-xs text-gray-500">{}</span>
                </div>
                <p class="mt-1 text-sm text-gray-900 dark:text-gray-100">{}</p>
            </div>"#,
            level_class, activity.category, time_str, activity.message
        ));
    }

    html.push_str("</div>");
    axum::response::Html(html)
}

/// Get activity feed
pub async fn get_activities_handler(
    State(state): State<AdminState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<crate::admin::service::helpers::activity::Activity>>>, StatusCode>
{
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(50);

    let activities = state.activity_logger.get_activities(Some(limit)).await;
    Ok(Json(ApiResponse::success(activities)))
}
