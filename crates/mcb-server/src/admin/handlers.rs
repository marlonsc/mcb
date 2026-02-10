//! Admin request handlers
//!
//! HTTP handlers for admin API endpoints including health checks,
//! performance metrics, indexing status, and runtime configuration management.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).
//! Authentication guards added in v0.1.2.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mcb_domain::ports::admin::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, IndexingOperation,
    IndexingOperationsInterface, PerformanceMetricsData, PerformanceMetricsInterface,
    ShutdownCoordinator,
};
use mcb_domain::ports::infrastructure::EventBusProvider;
use mcb_domain::ports::jobs::{Job, JobStatus, JobType};
use mcb_domain::ports::providers::CacheProvider;
use mcb_domain::ports::services::{ProjectServiceInterface, VcsEntityServiceInterface};
use mcb_domain::value_objects::OperationId;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::config::watcher::ConfigWatcher;
use mcb_infrastructure::infrastructure::ServiceManager;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get, post};
use serde::Serialize;
use tracing::info;

use super::auth::AdminAuth;

/// Admin handler state containing shared service references
#[derive(Clone)]
pub struct AdminState {
    /// Performance metrics tracker
    pub metrics: Arc<dyn PerformanceMetricsInterface>,
    /// Indexing operations tracker
    pub indexing: Arc<dyn IndexingOperationsInterface>,
    /// Configuration watcher for hot-reload support
    pub config_watcher: Option<Arc<ConfigWatcher>>,
    /// Current configuration snapshot (read-only fallback if watcher unavailable)
    pub current_config: AppConfig,
    /// Configuration file path (for updates)
    pub config_path: Option<PathBuf>,
    /// Shutdown coordinator for graceful shutdown
    pub shutdown_coordinator: Option<Arc<dyn ShutdownCoordinator>>,
    /// Default shutdown timeout in seconds
    pub shutdown_timeout_secs: u64,
    /// Event bus for SSE streaming
    pub event_bus: Arc<dyn EventBusProvider>,
    /// Service manager for lifecycle control
    pub service_manager: Option<Arc<ServiceManager>>,
    /// Cache provider for stats
    pub cache: Option<Arc<dyn CacheProvider>>,
    /// Project workflow service for project/phase/issue navigation
    pub project_workflow: Option<Arc<dyn ProjectServiceInterface>>,
    /// VCS entity service for repository/branch/worktree navigation
    pub vcs_entity: Option<Arc<dyn VcsEntityServiceInterface>>,
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
#[get("/health")]
pub fn health_check(state: &State<AdminState>) -> Json<AdminHealthResponse> {
    let metrics = state.metrics.get_performance_metrics();
    let operations = state.indexing.get_operations();

    Json(AdminHealthResponse {
        status: "healthy",
        uptime_seconds: metrics.uptime_seconds,
        active_indexing_operations: operations.len(),
    })
}

/// Get performance metrics endpoint (protected)
#[get("/metrics")]
pub fn get_metrics(_auth: AdminAuth, state: &State<AdminState>) -> Json<PerformanceMetricsData> {
    let metrics = state.metrics.get_performance_metrics();
    Json(metrics)
}

/// Jobs status response (unified job tracking)
#[derive(Serialize)]
pub struct JobsStatusResponse {
    /// Total number of tracked jobs
    pub total: usize,
    /// Number of currently running jobs
    pub running: usize,
    /// Number of queued jobs
    pub queued: usize,
    /// Job details
    pub jobs: Vec<Job>,
}

/// List all background jobs
#[get("/jobs")]
pub fn get_jobs_status(state: &State<AdminState>) -> Json<JobsStatusResponse> {
    let operations = state.indexing.get_operations();

    let jobs: Vec<Job> = operations
        .values()
        .map(|op| {
            let progress = if op.total_files > 0 {
                ((op.processed_files as f64 / op.total_files as f64) * 100.0) as u8
            } else {
                0
            };
            Job {
                id: op.id.clone(),
                job_type: JobType::Indexing,
                label: op.collection.to_string(),
                status: JobStatus::Running,
                progress_percent: progress,
                processed_items: op.processed_files,
                total_items: op.total_files,
                current_item: op.current_file.clone(),
                created_at: op.started_at,
                started_at: Some(op.started_at),
                completed_at: None,
                result: None,
            }
        })
        .collect();

    let running = jobs.len();
    Json(JobsStatusResponse {
        total: running,
        running,
        queued: 0,
        jobs,
    })
}

/// Projects list response for browse entity navigation
#[derive(Serialize)]
pub struct ProjectsBrowseResponse {
    /// List of projects
    pub projects: Vec<mcb_domain::entities::project::Project>,
    /// Total number of projects
    pub total: usize,
}

/// List workflow projects for browse entity graph
#[get("/projects")]
pub async fn list_browse_projects(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<ProjectsBrowseResponse>, (Status, Json<CacheErrorResponse>)> {
    let Some(project_workflow) = &state.project_workflow else {
        return Err((
            Status::ServiceUnavailable,
            Json(CacheErrorResponse {
                error: "Project workflow service not available".to_string(),
            }),
        ));
    };

    // TODO(phase-1): extract org_id from admin auth context
    match project_workflow
        .list_projects(mcb_domain::constants::keys::DEFAULT_ORG_ID)
        .await
    {
        Ok(projects) => {
            let total = projects.len();
            Ok(Json(ProjectsBrowseResponse { projects, total }))
        }
        Err(e) => Err((
            Status::InternalServerError,
            Json(CacheErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Serialize)]
pub struct RepositoriesBrowseResponse {
    pub repositories: Vec<mcb_domain::entities::repository::Repository>,
    pub total: usize,
}

#[get("/repositories?<project_id>")]
pub async fn list_browse_repositories(
    _auth: AdminAuth,
    state: &State<AdminState>,
    project_id: Option<String>,
) -> Result<Json<RepositoriesBrowseResponse>, (Status, Json<CacheErrorResponse>)> {
    let Some(vcs_entity) = &state.vcs_entity else {
        return Err((
            Status::ServiceUnavailable,
            Json(CacheErrorResponse {
                error: "VCS entity service not available".to_string(),
            }),
        ));
    };

    let org_id = mcb_domain::constants::keys::DEFAULT_ORG_ID;
    let pid = project_id.as_deref().unwrap_or("");

    match vcs_entity.list_repositories(org_id, pid).await {
        Ok(repositories) => {
            let total = repositories.len();
            Ok(Json(RepositoriesBrowseResponse {
                repositories,
                total,
            }))
        }
        Err(e) => Err((
            Status::InternalServerError,
            Json(CacheErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

/// Readiness response
#[derive(Serialize)]
pub struct ReadinessResponse {
    /// Whether the server is ready to accept requests
    pub ready: bool,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
}

/// Liveness response
#[derive(Serialize)]
pub struct LivenessResponse {
    /// Whether the server process is alive and responding
    pub alive: bool,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
}

/// Readiness check endpoint (for k8s/docker health checks)
#[get("/ready")]
pub fn readiness_check(state: &State<AdminState>) -> (Status, Json<ReadinessResponse>) {
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
    let metrics = state.metrics.get_performance_metrics();
    (
        Status::Ok,
        Json(LivenessResponse {
            alive: true,
            uptime_seconds: metrics.uptime_seconds,
        }),
    )
}

// ============================================================================
// Service Control Endpoints
// ============================================================================

/// Shutdown request body
#[derive(serde::Deserialize, Default)]
pub struct ShutdownRequest {
    /// Custom timeout in seconds (optional, uses default if not provided)
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    /// Immediate shutdown without graceful period (default: false)
    #[serde(default)]
    pub immediate: bool,
}

/// Shutdown response
#[derive(Serialize)]
pub struct ShutdownResponse {
    /// Whether shutdown was initiated
    pub initiated: bool,
    /// Message describing the shutdown status
    pub message: String,
    /// Timeout being used for graceful shutdown
    pub timeout_secs: u64,
}

impl ShutdownResponse {
    fn error(message: impl Into<String>, timeout: u64) -> Self {
        Self {
            initiated: false,
            message: message.into(),
            timeout_secs: timeout,
        }
    }

    fn success(message: impl Into<String>, timeout: u64) -> Self {
        Self {
            initiated: true,
            message: message.into(),
            timeout_secs: timeout,
        }
    }
}

/// Initiate graceful server shutdown (protected)
///
/// Signals all components to begin shutdown. The server will attempt
/// to complete in-flight requests before terminating.
///
/// # Request Body
///
/// - `timeout_secs`: Optional custom timeout (default: 30s)
/// - `immediate`: Skip graceful shutdown period (default: false)
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[post("/shutdown", format = "json", data = "<request>")]
pub fn shutdown(
    _auth: AdminAuth,
    state: &State<AdminState>,
    request: Json<ShutdownRequest>,
) -> (Status, Json<ShutdownResponse>) {
    let request = request.into_inner();

    let Some(coordinator) = &state.shutdown_coordinator else {
        return (
            Status::ServiceUnavailable,
            Json(ShutdownResponse::error(
                "Shutdown coordinator not available",
                0,
            )),
        );
    };

    if coordinator.is_shutting_down() {
        return (
            Status::Conflict,
            Json(ShutdownResponse::error(
                "Shutdown already in progress",
                state.shutdown_timeout_secs,
            )),
        );
    }

    let timeout_secs = request.timeout_secs.unwrap_or(state.shutdown_timeout_secs);

    if request.immediate {
        info!("Immediate shutdown requested");
        coordinator.signal_shutdown();
        return (
            Status::Ok,
            Json(ShutdownResponse::success("Immediate shutdown initiated", 0)),
        );
    }

    info!(timeout_secs = timeout_secs, "Graceful shutdown requested");
    spawn_graceful_shutdown(Arc::clone(coordinator), timeout_secs);

    let msg = format!(
        "Graceful shutdown initiated, server will stop in {} seconds",
        timeout_secs
    );
    (
        Status::Ok,
        Json(ShutdownResponse::success(msg, timeout_secs)),
    )
}

fn spawn_graceful_shutdown(coord: Arc<dyn ShutdownCoordinator>, timeout: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(timeout)).await;
        coord.signal_shutdown();
    });
}

/// Extended health check with dependency status (protected)
///
/// Returns detailed health information including the status of
/// all service dependencies (embedding provider, vector store, cache).
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/health/extended")]
pub fn extended_health_check(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Json<ExtendedHealthResponse> {
    let metrics = state.metrics.get_performance_metrics();
    let operations = state.indexing.get_operations();
    let now = current_timestamp();

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

/// Get current timestamp in seconds since UNIX epoch
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Build dependency health checks from metrics and operations
fn build_dependency_checks(
    metrics: &PerformanceMetricsData,
    operations: &std::collections::HashMap<OperationId, IndexingOperation>,
    now: u64,
) -> Vec<DependencyHealthCheck> {
    vec![
        build_embedding_health(metrics, now),
        build_vector_store_health(operations, now),
        build_cache_health(metrics, now),
    ]
}

/// Build embedding provider health check
fn build_embedding_health(metrics: &PerformanceMetricsData, now: u64) -> DependencyHealthCheck {
    DependencyHealthCheck {
        name: "embedding_provider".to_string(),
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

/// Build vector store health check
fn build_vector_store_health(
    operations: &std::collections::HashMap<OperationId, IndexingOperation>,
    now: u64,
) -> DependencyHealthCheck {
    DependencyHealthCheck {
        name: "vector_store".to_string(),
        status: DependencyHealth::Healthy,
        message: Some(format!("Active indexing operations: {}", operations.len())),
        latency_ms: None,
        last_check: now,
    }
}

/// Build cache health check
fn build_cache_health(metrics: &PerformanceMetricsData, now: u64) -> DependencyHealthCheck {
    DependencyHealthCheck {
        name: "cache".to_string(),
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

/// Calculate overall health status from individual dependency checks
fn calculate_overall_health(dependencies: &[DependencyHealthCheck]) -> DependencyHealth {
    let mut unhealthy_count = 0;
    let mut degraded_count = 0;

    for dep in dependencies {
        match dep.status {
            DependencyHealth::Unhealthy => unhealthy_count += 1,
            DependencyHealth::Degraded => degraded_count += 1,
            DependencyHealth::Healthy | DependencyHealth::Unknown => {
                // Healthy/Unknown dependencies don't need counting
            }
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

// ============================================================================
// Cache Stats Endpoint
// ============================================================================

/// Cache error response
#[derive(Serialize)]
pub struct CacheErrorResponse {
    /// Error message describing the cache operation failure
    pub error: String,
}

/// Get cache statistics (protected)
///
/// Returns cache hit/miss rates, entry counts, and other metrics.
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/cache/stats")]
pub async fn get_cache_stats(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<mcb_domain::ports::providers::cache::CacheStats>, (Status, Json<CacheErrorResponse>)>
{
    let Some(cache) = &state.cache else {
        return Err((
            Status::ServiceUnavailable,
            Json(CacheErrorResponse {
                error: "Cache provider not available".to_string(),
            }),
        ));
    };

    match cache.stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((
            Status::InternalServerError,
            Json(CacheErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}
