//! Handler helper functions to reduce boilerplate
//!
//! Provides common error handling, response wrapping, and utility functions
//! used across all admin API handlers.

use crate::admin::models::ApiResponse;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

/// Convert any error to INTERNAL_SERVER_ERROR
///
/// This function is used with `.map_err()` to convert any error type
/// into an HTTP 500 Internal Server Error response.
///
/// # Example
///
/// ```rust,ignore
/// let result = state.admin_service.some_operation()
///     .await
///     .map_err(internal_error)?;
/// ```
#[inline]
pub fn internal_error<E>(_error: E) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

/// Create a successful JSON response with data
///
/// Wraps the given data in an `ApiResponse::success()` and returns it
/// as a JSON response.
///
/// # Example
///
/// ```rust,ignore
/// let data = state.admin_service.get_system_info().await?;
/// ok_json(data)
/// ```
#[inline]
pub fn ok_json<T: Serialize>(data: T) -> Result<Json<ApiResponse<T>>, StatusCode> {
    Ok(Json(ApiResponse::success(data)))
}

/// Create an error JSON response
///
/// Wraps the given error message in an `ApiResponse::error()` and returns it
/// as a JSON response.
///
/// # Example
///
/// ```rust,ignore
/// if let Err(e) = some_operation() {
///     return err_json::<SystemInfo>("Failed to update system config");
/// }
/// ```
#[inline]
pub fn err_json<T: Serialize + Default>(
    message: impl AsRef<str>,
) -> Result<Json<ApiResponse<T>>, StatusCode> {
    Ok(Json(ApiResponse::error(message.as_ref().to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_error() {
        let result = internal_error::<String>("some error".to_string());
        assert_eq!(result, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_ok_json() {
        #[derive(Serialize)]
        struct TestData {
            value: i32,
        }

        let data = TestData { value: 42 };
        let result = ok_json(data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_err_json() {
        #[derive(Serialize, Default)]
        struct TestData;

        let result = err_json::<TestData>("test error message");
        assert!(result.is_ok());
    }
}
