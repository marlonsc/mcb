//! Admin control endpoints
//!
//! Provides endpoints for server control operations like shutdown.

use std::sync::Arc;
use std::time::Duration;

use mcb_domain::ports::admin::ShutdownCoordinator;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, post};
use serde::Serialize;
use tracing::info;

use crate::admin::auth::AdminAuth;
use crate::admin::handlers::AdminState;

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
#[post("/shutdown", format = "json", data = "<request>")]
pub fn shutdown(
    _auth: AdminAuth,
    state: &State<AdminState>,
    request: Json<ShutdownRequest>,
) -> (Status, Json<ShutdownResponse>) {
    tracing::info!("shutdown called");
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

    let msg = format!("Graceful shutdown initiated, server will stop in {timeout_secs} seconds");
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
