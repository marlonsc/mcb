//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use serde::Serialize;
use serde_with::skip_serializing_none;

/// Response for service list endpoint.
#[derive(Serialize)]
pub struct ServiceListResponse {
    /// Number of registered services.
    pub count: usize,
    /// List of services with their states.
    pub services: Vec<ServiceInfoResponse>,
}

/// Individual service info in the list.
#[derive(Serialize)]
pub struct ServiceInfoResponse {
    /// Service name.
    pub name: String,
    /// Current state as string.
    pub state: String,
}

/// Service error response.
#[skip_serializing_none]
#[derive(Serialize)]
pub struct ServiceErrorResponse {
    /// Error message describing what went wrong.
    pub error: String,
    /// Name of the service involved in the error (optional).
    pub service: Option<String>,
    /// Number of services (optional, used in list responses).
    pub count: Option<usize>,
    /// List of services (optional, used in list responses).
    pub services: Option<Vec<ServiceInfoResponse>>,
}

/// Service action response.
#[derive(Serialize)]
pub struct ServiceActionResponse {
    /// Status of the action (e.g., started, stopped, restarted).
    pub status: String,
    /// Name of the service that was acted upon.
    pub service: String,
}

/// Services health response.
#[derive(Serialize)]
pub struct ServicesHealthResponse {
    /// Number of services checked.
    pub count: usize,
    /// Health check results for each service.
    pub checks: Vec<serde_json::Value>,
}
