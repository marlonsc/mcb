//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Admin control endpoints
//!
//! Provides endpoints for server control operations like shutdown.

use std::sync::Arc;
use std::time::Duration;

use axum::Json as AxumJson;
use axum::extract::State as AxumState;
use axum::http::StatusCode;
use mcb_domain::info;
use mcb_domain::ports::ShutdownCoordinator;
use serde::Serialize;

use crate::admin::auth::AxumAdminAuth;
use crate::admin::error::{AdminError, AdminStatusResult};
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

fn spawn_graceful_shutdown(coord: Arc<dyn ShutdownCoordinator>, timeout: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(timeout)).await;
        coord.signal_shutdown();
    });
}

/// Axum handler: initiate graceful server shutdown (protected).
///
/// # Errors
/// Returns `503` when the shutdown coordinator is unavailable and `409` when
/// a shutdown is already in progress.
pub async fn shutdown_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
    AxumJson(request): AxumJson<ShutdownRequest>,
) -> AdminStatusResult<ShutdownResponse> {
    info!("control", "shutdown called");

    let coordinator = require_service!(
        state,
        shutdown_coordinator,
        "Shutdown coordinator not available"
    );

    if coordinator.is_shutting_down() {
        return Err(AdminError::json(
            StatusCode::CONFLICT,
            &ShutdownResponse::error("Shutdown already in progress", state.shutdown_timeout_secs),
        ));
    }

    let timeout_secs = request.timeout_secs.unwrap_or(state.shutdown_timeout_secs);

    if request.immediate {
        info!("control", "Immediate shutdown requested");
        coordinator.signal_shutdown();
        return Ok((
            StatusCode::OK,
            AxumJson(ShutdownResponse::success("Immediate shutdown initiated", 0)),
        ));
    }

    info!("control", "Graceful shutdown requested", &timeout_secs);
    spawn_graceful_shutdown(Arc::clone(coordinator), timeout_secs);

    let msg = format!("Graceful shutdown initiated, server will stop in {timeout_secs} seconds");
    Ok((
        StatusCode::OK,
        AxumJson(ShutdownResponse::success(msg, timeout_secs)),
    ))
}
