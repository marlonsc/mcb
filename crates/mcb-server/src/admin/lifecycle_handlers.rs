//! Service Lifecycle HTTP Handlers
//!
//! HTTP handlers for service lifecycle management endpoints.
//! These endpoints allow starting, stopping, and restarting
//! services via the ServiceManager.
//!
//! ## Endpoints
//!
//! | Path | Method | Description |
//! |------|--------|-------------|
//! | `/services` | GET | List all registered services and their states |
//! | `/services/{name}/start` | POST | Start a specific service |
//! | `/services/{name}/stop` | POST | Stop a specific service |
//! | `/services/{name}/restart` | POST | Restart a specific service |

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::json;

use super::handlers::AdminState;

/// Response for service list endpoint
#[derive(Serialize)]
pub struct ServiceListResponse {
    /// Number of registered services
    pub count: usize,
    /// List of services with their states
    pub services: Vec<ServiceInfoResponse>,
}

/// Individual service info in the list
#[derive(Serialize)]
pub struct ServiceInfoResponse {
    /// Service name
    pub name: String,
    /// Current state as string
    pub state: String,
}

/// List all registered services and their states
///
/// GET /admin/services
pub async fn list_services(State(state): State<AdminState>) -> impl IntoResponse {
    let Some(service_manager) = &state.service_manager else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "Service manager not available",
                "count": 0,
                "services": []
            })),
        )
            .into_response();
    };

    let services: Vec<ServiceInfoResponse> = service_manager
        .list()
        .into_iter()
        .map(|info| ServiceInfoResponse {
            name: info.name,
            state: format!("{:?}", info.state),
        })
        .collect();

    Json(ServiceListResponse {
        count: services.len(),
        services,
    })
    .into_response()
}

/// Start a specific service
///
/// POST /admin/services/{name}/start
pub async fn start_service(
    State(state): State<AdminState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let Some(service_manager) = &state.service_manager else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "Service manager not available"
            })),
        );
    };

    match service_manager.start(&name).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "status": "started",
                "service": name
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e.to_string(),
                "service": name
            })),
        ),
    }
}

/// Stop a specific service
///
/// POST /admin/services/{name}/stop
pub async fn stop_service(
    State(state): State<AdminState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let Some(service_manager) = &state.service_manager else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "Service manager not available"
            })),
        );
    };

    match service_manager.stop(&name).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "status": "stopped",
                "service": name
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e.to_string(),
                "service": name
            })),
        ),
    }
}

/// Restart a specific service
///
/// POST /admin/services/{name}/restart
pub async fn restart_service(
    State(state): State<AdminState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let Some(service_manager) = &state.service_manager else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "Service manager not available"
            })),
        );
    };

    match service_manager.restart(&name).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "status": "restarted",
                "service": name
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e.to_string(),
                "service": name
            })),
        ),
    }
}

/// Get health check for all services
///
/// GET /admin/services/health
pub async fn services_health(State(state): State<AdminState>) -> impl IntoResponse {
    let Some(service_manager) = &state.service_manager else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "Service manager not available"
            })),
        )
            .into_response();
    };

    let checks = service_manager.health_check_all().await;
    Json(json!({
        "count": checks.len(),
        "checks": checks
    }))
    .into_response()
}
