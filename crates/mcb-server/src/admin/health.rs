//! Admin health check definition
//!
//! Provides endpoints for liveness, readiness, and extended health checks.

use mcb_domain::ports::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, PerformanceMetricsData,
};
use mcb_domain::utils::time as domain_time;
use mcb_domain::value_objects::OperationId;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get};

use crate::admin::auth::AdminAuth;
use crate::admin::handlers::AdminState;
use crate::admin::models::{AdminHealthResponse, LivenessResponse, ReadinessResponse};

/// Health check endpoint
#[get("/health")]
pub fn health_check(state: &State<AdminState>) -> Json<AdminHealthResponse> {
    tracing::info!("health_check called");
    let metrics = state.metrics.get_performance_metrics();
    let operations = state.indexing.get_operations();

    Json(AdminHealthResponse {
        status: "healthy",
        uptime_seconds: metrics.uptime_seconds,
        active_indexing_operations: operations.len(),
    })
}

/// Readiness check endpoint (for k8s/docker health checks)
#[get("/ready")]
pub fn readiness_check(state: &State<AdminState>) -> (Status, Json<ReadinessResponse>) {
    tracing::info!("readiness_check called");
    let metrics = state.metrics.get_performance_metrics();

    // Consider ready if server has been up for at least 1 second
    if metrics.uptime_seconds >= 1 {
        (
            Status::Ok,
            Json(ReadinessResponse {
                ready: true,
                uptime_seconds: metrics.uptime_seconds,
            }),
        )
    } else {
        (
            Status::ServiceUnavailable,
            Json(ReadinessResponse {
                ready: false,
                uptime_seconds: metrics.uptime_seconds,
            }),
        )
    }
}

/// Liveness check endpoint (for k8s/docker health checks)
#[get("/live")]
pub fn liveness_check(state: &State<AdminState>) -> (Status, Json<LivenessResponse>) {
    tracing::info!("liveness_check called");
    let metrics = state.metrics.get_performance_metrics();
    (
        Status::Ok,
        Json(LivenessResponse {
            alive: true,
            uptime_seconds: metrics.uptime_seconds,
        }),
    )
}

/// Extended health check with dependency status (protected)
#[get("/health/extended")]
pub fn extended_health_check(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Json<ExtendedHealthResponse> {
    tracing::info!("extended_health_check called");
    let metrics = state.metrics.get_performance_metrics();
    let operations = state.indexing.get_operations();
    let now = domain_time::epoch_secs_u64().unwrap_or(0);

    let dependencies = build_dependency_checks(&metrics, &operations, now);
    let dependencies_status = calculate_overall_health(&dependencies);

    let response = ExtendedHealthResponse {
        status: if dependencies_status == DependencyHealth::Unhealthy {
            "degraded"
        } else {
            "healthy"
        },
        uptime_seconds: metrics.uptime_seconds,
        active_indexing_operations: operations.len(),
        dependencies,
        dependencies_status,
    };

    Json(response)
}

fn build_dependency_checks(
    metrics: &PerformanceMetricsData,
    operations: &std::collections::HashMap<OperationId, mcb_domain::ports::IndexingOperation>,
    now: u64,
) -> Vec<DependencyHealthCheck> {
    vec![
        build_embedding_health(metrics, now),
        build_vector_store_health(operations, now),
        build_cache_health(metrics, now),
    ]
}

fn build_embedding_health(metrics: &PerformanceMetricsData, now: u64) -> DependencyHealthCheck {
    DependencyHealthCheck {
        name: "embedding_provider".to_owned(),
        status: match (metrics.total_queries, metrics.failed_queries) {
            (total, 0) if total > 0 => DependencyHealth::Healthy,
            (_, failed) if failed > 0 => DependencyHealth::Degraded,
            _ => DependencyHealth::Unknown,
        },
        message: Some(format!(
            "Total queries: {}, Failed: {}",
            metrics.total_queries, metrics.failed_queries
        )),
        latency_ms: Some(metrics.average_response_time_ms as u64),
        last_check: now,
    }
}

fn build_vector_store_health(
    operations: &std::collections::HashMap<OperationId, mcb_domain::ports::IndexingOperation>,
    now: u64,
) -> DependencyHealthCheck {
    DependencyHealthCheck {
        name: "vector_store".to_owned(),
        status: DependencyHealth::Healthy,
        message: Some(format!("Active indexing operations: {}", operations.len())),
        latency_ms: None,
        last_check: now,
    }
}

fn build_cache_health(metrics: &PerformanceMetricsData, now: u64) -> DependencyHealthCheck {
    DependencyHealthCheck {
        name: "cache".to_owned(),
        status: if metrics.cache_hit_rate > 0.0 {
            DependencyHealth::Healthy
        } else {
            DependencyHealth::Unknown
        },
        message: Some(format!(
            "Cache hit rate: {:.1}%",
            metrics.cache_hit_rate * 100.0
        )),
        latency_ms: None,
        last_check: now,
    }
}

fn calculate_overall_health(dependencies: &[DependencyHealthCheck]) -> DependencyHealth {
    let mut unhealthy_count = 0;
    let mut degraded_count = 0;

    for dep in dependencies {
        match dep.status {
            DependencyHealth::Unhealthy => unhealthy_count += 1,
            DependencyHealth::Degraded => degraded_count += 1,
            DependencyHealth::Healthy | DependencyHealth::Unknown => {}
        }
    }

    if unhealthy_count > 0 {
        DependencyHealth::Unhealthy
    } else if degraded_count > 0 {
        DependencyHealth::Degraded
    } else {
        DependencyHealth::Healthy
    }
}

/// Get performance metrics endpoint (protected)
#[get("/metrics")]
pub fn get_metrics(_auth: AdminAuth, state: &State<AdminState>) -> Json<PerformanceMetricsData> {
    tracing::info!("get_metrics called");
    let metrics = state.metrics.get_performance_metrics();
    Json(metrics)
}
