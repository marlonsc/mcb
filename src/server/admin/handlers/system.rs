//! System status and dashboard handlers

use super::common::*;
use super::SearchQuery;
use crate::infrastructure::utils::{HealthUtils, IntoStatusCode};

/// Get system status
pub async fn get_status_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let system_info = state.admin_service.get_system_info().await.to_500()?;
    let providers = state.admin_service.get_providers().await.to_500()?;
    let indexing_status = state.admin_service.get_indexing_status().await.to_500()?;
    let performance = state
        .admin_service
        .get_performance_metrics()
        .await
        .to_500()?;

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
    let indexing_status = state.admin_service.get_indexing_status().await.to_500()?;
    let performance = state
        .admin_service
        .get_performance_metrics()
        .await
        .to_500()?;
    let providers = state.admin_service.get_providers().await.to_500()?;

    let active_providers = providers.iter().filter(|p| p.status == "available").count();

    let cpu = state
        .mcp_server
        .system_collector
        .collect_cpu_metrics()
        .await
        .map(|m| m.usage)
        .to_500()?;

    let memory = state
        .mcp_server
        .system_collector
        .collect_memory_metrics()
        .await
        .map(|m| m.usage_percent)
        .to_500()?;

    let system_info = state.admin_service.get_system_info().await.to_500()?;

    let health_status = HealthUtils::compute_status(cpu as f64, memory as f64);

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

/// Search handler
pub async fn search_handler(
    State(state): State<AdminState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<ApiResponse<crate::application::admin::types::SearchResults>>, StatusCode> {
    match state
        .admin_service
        .search(&params.q, None, params.limit)
        .await
    {
        Ok(results) => Ok(Json(ApiResponse::success(results))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Search failed: {}", e)))),
    }
}

/// Get activity feed
pub async fn get_activities_handler(
    State(state): State<AdminState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<
    Json<ApiResponse<Vec<crate::application::admin::helpers::activity::Activity>>>,
    StatusCode,
> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(50);

    let activities = state.activity_logger.get_activities(Some(limit)).await;
    Ok(Json(ApiResponse::success(activities)))
}
