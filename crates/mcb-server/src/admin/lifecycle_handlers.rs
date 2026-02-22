//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Service Lifecycle HTTP Handlers
//!
//! HTTP handlers for service lifecycle management endpoints.
//! These endpoints allow starting, stopping, and restarting
//! services via the `ServiceManager`.
//!
//! ## Endpoints
//!
//! | Path | Method | Description |
//! | ------ | -------- | ------------- |
//! | `/services` | GET | List all registered services and their states (protected) |
//! | `/services/{name}/start` | POST | Start a specific service (protected) |
//! | `/services/{name}/stop` | POST | Stop a specific service (protected) |
//! | `/services/{name}/restart` | POST | Restart a specific service (protected) |

use std::sync::Arc;

use axum::Json as AxumJson;
use axum::extract::{Path, State as AxumState};
use axum::http::StatusCode;
use serde_json::json;

use mcb_domain::info;

use super::auth::AxumAdminAuth;
use super::handlers::AdminState;
pub use super::lifecycle_models::{
    ServiceActionResponse, ServiceErrorResponse, ServiceInfoResponse, ServiceListResponse,
    ServicesHealthResponse,
};

enum ServiceAction {
    Start,
    Stop,
    Restart,
}

impl ServiceAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Start => "started",
            Self::Stop => "stopped",
            Self::Restart => "restarted",
        }
    }
}

use crate::admin::error::{AdminError, AdminResult, AdminStatusResult};

async fn execute_service_action_axum(
    state: &AdminState,
    name: &str,
    action: ServiceAction,
) -> AdminStatusResult<ServiceActionResponse> {
    let service_manager = require_service!(state, service_manager, "Service manager not available");

    let result = match action {
        ServiceAction::Start => service_manager.start(name).await,
        ServiceAction::Stop => service_manager.stop(name).await,
        ServiceAction::Restart => service_manager.restart(name).await,
    };

    match result {
        Ok(()) => Ok((
            StatusCode::OK,
            AxumJson(ServiceActionResponse {
                status: action.label().to_owned(),
                service: name.to_owned(),
            }),
        )),
        Err(error) => Err(AdminError::bad_request(error.to_string())),
    }
}

/// Axum handler: list all registered services and their states (protected).
///
/// # Errors
/// Returns `503` when service manager is unavailable.
pub async fn list_services_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
) -> AdminResult<ServiceListResponse> {
    info!("lifecycle", "list_services called");
    let service_manager = require_service!(state, service_manager, "Service manager not available");

    let services = service_manager
        .list()
        .into_iter()
        .map(|info| ServiceInfoResponse {
            name: info.name,
            state: format!("{:?}", info.state),
        })
        .collect::<Vec<_>>();

    Ok(AxumJson(ServiceListResponse {
        count: services.len(),
        services,
    }))
}

/// Axum handler: start a specific service (protected).
///
/// # Errors
/// Returns `503` when service manager is unavailable and `400` on action errors.
pub async fn start_service_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
    Path(name): Path<String>,
) -> AdminStatusResult<ServiceActionResponse> {
    info!("lifecycle", "start_service called");
    execute_service_action_axum(&state, &name, ServiceAction::Start).await
}

/// Axum handler: stop a specific service (protected).
///
/// # Errors
/// Returns `503` when service manager is unavailable and `400` on action errors.
pub async fn stop_service_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
    Path(name): Path<String>,
) -> AdminStatusResult<ServiceActionResponse> {
    info!("lifecycle", "stop_service called");
    execute_service_action_axum(&state, &name, ServiceAction::Stop).await
}

/// Axum handler: restart a specific service (protected).
///
/// # Errors
/// Returns `503` when service manager is unavailable and `400` on action errors.
pub async fn restart_service_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
    Path(name): Path<String>,
) -> AdminStatusResult<ServiceActionResponse> {
    info!("lifecycle", "restart_service called");
    execute_service_action_axum(&state, &name, ServiceAction::Restart).await
}

/// Axum handler: get health check for all services (protected).
///
/// # Errors
/// Returns `503` when service manager is unavailable.
pub async fn services_health_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
) -> AdminResult<ServicesHealthResponse> {
    info!("lifecycle", "services_health called");
    let service_manager = require_service!(state, service_manager, "Service manager not available");

    let checks = service_manager.health_check_all().await;
    let checks_json = checks
        .iter()
        .map(|c| serde_json::to_value(c).unwrap_or(json!({})))
        .collect::<Vec<_>>();

    Ok(AxumJson(ServicesHealthResponse {
        count: checks.len(),
        checks: checks_json,
    }))
}
