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
