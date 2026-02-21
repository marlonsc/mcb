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

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get, post};
use serde_json::json;

use super::auth::AdminAuth;
use super::handlers::AdminState;
pub use super::lifecycle_models::{
    ServiceActionResponse, ServiceErrorResponse, ServiceInfoResponse, ServiceListResponse,
    ServicesHealthResponse,
};

fn service_manager_unavailable(
    include_list_defaults: bool,
) -> (Status, Json<ServiceErrorResponse>) {
    (
        Status::ServiceUnavailable,
        Json(ServiceErrorResponse {
            error: "Service manager not available".to_owned(),
            service: None,
            count: include_list_defaults.then_some(0),
            services: include_list_defaults.then_some(vec![]),
        }),
    )
}

fn service_action_success(action: &str, name: &str) -> (Status, Json<ServiceActionResponse>) {
    (
        Status::Ok,
        Json(ServiceActionResponse {
            status: action.to_owned(),
            service: name.to_owned(),
        }),
    )
}

fn service_action_error(
    name: &str,
    error: impl std::fmt::Display,
) -> (Status, Json<ServiceErrorResponse>) {
    (
        Status::BadRequest,
        Json(ServiceErrorResponse {
            error: error.to_string(),
            service: Some(name.to_owned()),
            count: None,
            services: None,
        }),
    )
}

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

async fn execute_service_action(
    state: &State<AdminState>,
    name: &str,
    action: ServiceAction,
) -> Result<(Status, Json<ServiceActionResponse>), (Status, Json<ServiceErrorResponse>)> {
    let Some(service_manager) = &state.service_manager else {
        return Err(service_manager_unavailable(false));
    };

    let result = match action {
        ServiceAction::Start => service_manager.start(name).await,
        ServiceAction::Stop => service_manager.stop(name).await,
        ServiceAction::Restart => service_manager.restart(name).await,
    };

    match result {
        Ok(()) => Ok(service_action_success(action.label(), name)),
        Err(error) => Err(service_action_error(name, error)),
    }
}

/// List all registered services and their states (protected)
///
/// GET /admin/services
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
///
/// # Errors
/// Returns `503` when service manager is unavailable.
#[get("/services")]
pub fn list_services(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<ServiceListResponse>, (Status, Json<ServiceErrorResponse>)> {
    tracing::info!("list_services called");
    let Some(service_manager) = &state.service_manager else {
        return Err(service_manager_unavailable(true));
    };

    let services = service_manager
        .list()
        .into_iter()
        .map(|info| ServiceInfoResponse {
            name: info.name,
            state: format!("{:?}", info.state),
        })
        .collect::<Vec<_>>();

    Ok(Json(ServiceListResponse {
        count: services.len(),
        services,
    }))
}

/// Start a specific service (protected)
///
/// POST /admin/services/{name}/start
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
///
/// # Errors
/// Returns `503` when service manager is unavailable and `400` on action errors.
#[post("/services/<name>/start")]
pub async fn start_service(
    _auth: AdminAuth,
    state: &State<AdminState>,
    name: &str,
) -> Result<(Status, Json<ServiceActionResponse>), (Status, Json<ServiceErrorResponse>)> {
    tracing::info!("start_service called");
    execute_service_action(state, name, ServiceAction::Start).await
}

/// Stop a specific service (protected)
///
/// POST /admin/services/{name}/stop
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
///
/// # Errors
/// Returns `503` when service manager is unavailable and `400` on action errors.
#[post("/services/<name>/stop")]
pub async fn stop_service(
    _auth: AdminAuth,
    state: &State<AdminState>,
    name: &str,
) -> Result<(Status, Json<ServiceActionResponse>), (Status, Json<ServiceErrorResponse>)> {
    tracing::info!("stop_service called");
    execute_service_action(state, name, ServiceAction::Stop).await
}

/// Restart a specific service (protected)
///
/// POST /admin/services/{name}/restart
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
///
/// # Errors
/// Returns `503` when service manager is unavailable and `400` on action errors.
#[post("/services/<name>/restart")]
pub async fn restart_service(
    _auth: AdminAuth,
    state: &State<AdminState>,
    name: &str,
) -> Result<(Status, Json<ServiceActionResponse>), (Status, Json<ServiceErrorResponse>)> {
    tracing::info!("restart_service called");
    execute_service_action(state, name, ServiceAction::Restart).await
}

/// Get health check for all services (protected)
///
/// GET /admin/services/health
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
///
/// # Errors
/// Returns `503` when service manager is unavailable.
#[get("/services/health")]
pub async fn services_health(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<ServicesHealthResponse>, (Status, Json<ServiceErrorResponse>)> {
    tracing::info!("services_health called");
    let Some(service_manager) = &state.service_manager else {
        return Err(service_manager_unavailable(false));
    };

    let checks = service_manager.health_check_all().await;
    let checks_json = checks
        .iter()
        .map(|c| serde_json::to_value(c).unwrap_or(json!({})))
        .collect::<Vec<_>>();

    Ok(Json(ServicesHealthResponse {
        count: checks.len(),
        checks: checks_json,
    }))
}

// ---------------------------------------------------------------------------
// Axum handlers
// ---------------------------------------------------------------------------

fn service_manager_unavailable_axum(
    include_list_defaults: bool,
) -> (axum::http::StatusCode, axum::Json<ServiceErrorResponse>) {
    (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        axum::Json(ServiceErrorResponse {
            error: "Service manager not available".to_owned(),
            service: None,
            count: include_list_defaults.then_some(0),
            services: include_list_defaults.then_some(vec![]),
        }),
    )
}

fn service_action_success_axum(
    action: &str,
    name: &str,
) -> (axum::http::StatusCode, axum::Json<ServiceActionResponse>) {
    (
        axum::http::StatusCode::OK,
        axum::Json(ServiceActionResponse {
            status: action.to_owned(),
            service: name.to_owned(),
        }),
    )
}

fn service_action_error_axum(
    name: &str,
    error: impl std::fmt::Display,
) -> (axum::http::StatusCode, axum::Json<ServiceErrorResponse>) {
    (
        axum::http::StatusCode::BAD_REQUEST,
        axum::Json(ServiceErrorResponse {
            error: error.to_string(),
            service: Some(name.to_owned()),
            count: None,
            services: None,
        }),
    )
}

async fn execute_service_action_axum(
    state: &AdminState,
    name: &str,
    action: ServiceAction,
) -> Result<
    (axum::http::StatusCode, axum::Json<ServiceActionResponse>),
    (axum::http::StatusCode, axum::Json<ServiceErrorResponse>),
> {
    let Some(service_manager) = &state.service_manager else {
        return Err(service_manager_unavailable_axum(false));
    };

    let result = match action {
        ServiceAction::Start => service_manager.start(name).await,
        ServiceAction::Stop => service_manager.stop(name).await,
        ServiceAction::Restart => service_manager.restart(name).await,
    };

    match result {
        Ok(()) => Ok(service_action_success_axum(action.label(), name)),
        Err(error) => Err(service_action_error_axum(name, error)),
    }
}

/// Axum handler: list all registered services and their states (protected).
///
/// # Errors
/// Returns `503` when service manager is unavailable.
pub fn list_services_axum(
    _auth: crate::admin::auth::AxumAdminAuth,
    axum::extract::State(state): axum::extract::State<Arc<AdminState>>,
) -> Result<
    axum::Json<ServiceListResponse>,
    (axum::http::StatusCode, axum::Json<ServiceErrorResponse>),
> {
    tracing::info!("list_services called");
    let Some(service_manager) = &state.service_manager else {
        return Err(service_manager_unavailable_axum(true));
    };

    let services = service_manager
        .list()
        .into_iter()
        .map(|info| ServiceInfoResponse {
            name: info.name,
            state: format!("{:?}", info.state),
        })
        .collect::<Vec<_>>();

    Ok(axum::Json(ServiceListResponse {
        count: services.len(),
        services,
    }))
}

/// Axum handler: start a specific service (protected).
///
/// # Errors
/// Returns `503` when service manager is unavailable and `400` on action errors.
pub async fn start_service_axum(
    _auth: crate::admin::auth::AxumAdminAuth,
    axum::extract::State(state): axum::extract::State<Arc<AdminState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<
    (axum::http::StatusCode, axum::Json<ServiceActionResponse>),
    (axum::http::StatusCode, axum::Json<ServiceErrorResponse>),
> {
    tracing::info!("start_service called");
    execute_service_action_axum(&state, &name, ServiceAction::Start).await
}

/// Axum handler: stop a specific service (protected).
///
/// # Errors
/// Returns `503` when service manager is unavailable and `400` on action errors.
pub async fn stop_service_axum(
    _auth: crate::admin::auth::AxumAdminAuth,
    axum::extract::State(state): axum::extract::State<Arc<AdminState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<
    (axum::http::StatusCode, axum::Json<ServiceActionResponse>),
    (axum::http::StatusCode, axum::Json<ServiceErrorResponse>),
> {
    tracing::info!("stop_service called");
    execute_service_action_axum(&state, &name, ServiceAction::Stop).await
}

/// Axum handler: restart a specific service (protected).
///
/// # Errors
/// Returns `503` when service manager is unavailable and `400` on action errors.
pub async fn restart_service_axum(
    _auth: crate::admin::auth::AxumAdminAuth,
    axum::extract::State(state): axum::extract::State<Arc<AdminState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<
    (axum::http::StatusCode, axum::Json<ServiceActionResponse>),
    (axum::http::StatusCode, axum::Json<ServiceErrorResponse>),
> {
    tracing::info!("restart_service called");
    execute_service_action_axum(&state, &name, ServiceAction::Restart).await
}

/// Axum handler: get health check for all services (protected).
///
/// # Errors
/// Returns `503` when service manager is unavailable.
pub async fn services_health_axum(
    _auth: crate::admin::auth::AxumAdminAuth,
    axum::extract::State(state): axum::extract::State<Arc<AdminState>>,
) -> Result<
    axum::Json<ServicesHealthResponse>,
    (axum::http::StatusCode, axum::Json<ServiceErrorResponse>),
> {
    tracing::info!("services_health called");
    let Some(service_manager) = &state.service_manager else {
        return Err(service_manager_unavailable_axum(false));
    };

    let checks = service_manager.health_check_all().await;
    let checks_json = checks
        .iter()
        .map(|c| serde_json::to_value(c).unwrap_or(json!({})))
        .collect::<Vec<_>>();

    Ok(axum::Json(ServicesHealthResponse {
        count: checks.len(),
        checks: checks_json,
    }))
}
