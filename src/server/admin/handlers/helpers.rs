//! Centralized handler helper functions to reduce boilerplate
//!
//! This module provides unified, reusable patterns for:
//! - Error handling and conversion
//! - JSON response wrapping
//! - Service call handling
//! - Common handler patterns
//!
//! All helpers are designed for maximum code reuse across handlers.

use crate::server::admin::models::ApiResponse;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use std::fmt::Display;

// =============================================================================
// Centralized Error Handling
// =============================================================================

/// Convert any error type to HTTP 500 Internal Server Error
///
/// Generic error converter for use with `.map_err()` to standardize
/// error handling across all handlers.
#[inline]
pub fn internal_error<E: Display>(_error: E) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

/// Convert error with custom logging
#[inline]
pub fn internal_error_with_log<E: Display>(error: E) -> StatusCode {
    tracing::error!("[ADMIN_HANDLER] Internal error: {}", error);
    StatusCode::INTERNAL_SERVER_ERROR
}

// =============================================================================
// Centralized Response Creation
// =============================================================================

/// Success response wrapper - create successful JSON response
#[inline]
pub fn success<T: Serialize>(data: T) -> Result<Json<ApiResponse<T>>, StatusCode> {
    Ok(Json(ApiResponse::success(data)))
}

/// Error response wrapper - create error JSON response
#[inline]
pub fn error<T: Serialize + Default>(
    message: impl AsRef<str>,
) -> Result<Json<ApiResponse<T>>, StatusCode> {
    Ok(Json(ApiResponse::error(message.as_ref().to_string())))
}

/// Short aliases for quick usage (deprecated - use success/error directly)
#[deprecated(since = "0.1.0", note = "use `success()` or `error()` instead")]
pub fn ok_json<T: Serialize>(data: T) -> Result<Json<ApiResponse<T>>, StatusCode> {
    success(data)
}

/// Deprecated error response wrapper - use `error()` instead
#[deprecated(since = "0.1.0", note = "use `error()` instead")]
pub fn err_json<T: Serialize + Default>(
    message: impl AsRef<str>,
) -> Result<Json<ApiResponse<T>>, StatusCode> {
    error(message)
}

// =============================================================================
// Unified Service Call Pattern Handler
// =============================================================================

/// Handle a service call that returns a result, converting errors to HTTP responses
///
/// This is the core pattern used by almost all handlers:
/// 1. Call async service method
/// 2. Convert errors to HTTP 500
/// 3. Wrap success data in ApiResponse
///
/// # Example
///
/// ```rust,ignore
/// handle_service_call(state.admin_service.get_system_info())
///     .await
/// ```
pub async fn handle_service_call<T, E>(
    service_result: impl std::future::Future<Output = Result<T, E>>,
) -> Result<Json<ApiResponse<T>>, StatusCode>
where
    T: Serialize,
    E: Display,
{
    match service_result.await {
        Ok(data) => success(data),
        Err(e) => {
            tracing::error!("[ADMIN_HANDLER] Service call failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle service call with custom error message
pub async fn handle_service_call_with_msg<T, E>(
    service_result: impl std::future::Future<Output = Result<T, E>>,
    error_msg: impl Display,
) -> Result<Json<ApiResponse<T>>, StatusCode>
where
    T: Serialize,
    E: Display,
{
    match service_result.await {
        Ok(data) => success(data),
        Err(e) => {
            tracing::error!("[ADMIN_HANDLER] {}: {}", error_msg, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// =============================================================================
// Result Conversion Helpers
// =============================================================================

/// Convert a Result to a JSON response, using custom error message
#[inline]
pub fn result_to_response<T: Serialize, E: Display>(
    result: Result<T, E>,
    error_message: &str,
) -> Result<Json<ApiResponse<T>>, StatusCode> {
    match result {
        Ok(data) => success(data),
        Err(e) => {
            tracing::error!("[ADMIN_HANDLER] {}: {}", error_message, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Convert a Result directly to JSON response (auto error message)
#[inline]
pub fn result_to_json<T: Serialize, E: Display>(
    result: Result<T, E>,
) -> Result<Json<ApiResponse<T>>, StatusCode> {
    match result {
        Ok(data) => success(data),
        Err(e) => {
            tracing::error!("[ADMIN_HANDLER] Error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// =============================================================================
// Status Code Helpers
// =============================================================================

/// Map error to appropriate HTTP status code
#[inline]
pub fn error_to_status<E: Display>(error: E) -> StatusCode {
    tracing::error!("[ADMIN_HANDLER] {}", error);
    StatusCode::INTERNAL_SERVER_ERROR
}

/// Create response with specific status code
pub fn status_response<T: Serialize>(
    data: T,
    status: StatusCode,
) -> Result<(StatusCode, Json<ApiResponse<T>>), StatusCode> {
    Ok((status, Json(ApiResponse::success(data))))
}
