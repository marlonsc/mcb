//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Declarative error type for Axum admin handlers.
//!
//! Replaces verbose `(StatusCode, Json<SomeErrorResponse>)` tuples with a
//! single `AdminError` that implements [`IntoResponse`].

use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;

/// Unified error type for all Axum admin handlers.
///
/// Implements [`IntoResponse`] so handlers can return
/// `Result<Json<T>, AdminError>` directly.
pub struct AdminError {
    status: StatusCode,
    body: serde_json::Value,
}

impl AdminError {
    /// 503 Service Unavailable with a plain message.
    pub fn unavailable(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: serde_json::json!({ "error": msg.into() }),
        }
    }

    /// 400 Bad Request with a plain message.
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: serde_json::json!({ "error": msg.into() }),
        }
    }

    /// 500 Internal Server Error with a plain message.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: serde_json::json!({ "error": msg.into() }),
        }
    }

    /// 409 Conflict with a plain message.
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: serde_json::json!({ "error": msg.into() }),
        }
    }

    /// 404 Not Found with a plain message.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: serde_json::json!({ "error": msg.into() }),
        }
    }

    /// Arbitrary status with a custom serializable body.
    pub fn json<T: Serialize>(status: StatusCode, body: &T) -> Self {
        Self {
            status,
            body: serde_json::to_value(body)
                .unwrap_or(serde_json::json!({ "error": "serialization failed" })),
        }
    }
}

impl IntoResponse for AdminError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.body)).into_response()
    }
}

/// Convenience alias: handler returns `Json<T>` on success, `AdminError` on failure.
pub type AdminResult<T> = Result<Json<T>, AdminError>;

/// Convenience alias: handler returns `(StatusCode, Json<T>)` on success, `AdminError` on failure.
pub type AdminStatusResult<T> = Result<(StatusCode, Json<T>), AdminError>;
