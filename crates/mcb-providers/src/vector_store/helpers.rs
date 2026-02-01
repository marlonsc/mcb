//! Vector Store Provider Helpers
//!
//! Shared utilities for vector store provider implementations (DRY principle).
//! Contains common HTTP error handling and response parsing patterns.

use std::time::Duration;

use mcb_domain::error::Error;

/// Handle HTTP request errors for vector store operations
///
/// Converts reqwest errors into domain errors with proper timeout detection.
///
/// # Arguments
/// * `error` - The reqwest error to handle
/// * `timeout` - The timeout duration for error messages
/// * `provider` - The provider name for error context
/// * `operation` - The operation being performed (e.g., "search", "upsert")
///
/// # Returns
/// Domain Error with appropriate message
pub fn handle_vector_request_error(
    error: reqwest::Error,
    timeout: Duration,
    provider: &str,
    operation: &str,
) -> Error {
    if error.is_timeout() {
        Error::vector_db(format!(
            "{} {} request timed out after {:?}",
            provider, operation, timeout
        ))
    } else {
        Error::vector_db(format!(
            "{} HTTP request for {} failed: {}",
            provider, operation, error
        ))
    }
}

/// Parse vector from JSON response
///
/// Extracts a vector array from a JSON value at the specified field.
///
/// # Arguments
/// * `item` - JSON value containing the vector
/// * `field` - Field name to extract
///
/// # Returns
/// Parsed vector or None if field is missing/invalid
pub fn parse_vector_from_json(item: &serde_json::Value, field: &str) -> Option<Vec<f32>> {
    item[field].as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_f64().map(|n| n as f32))
            .collect()
    })
}

/// Parse score from JSON response with fallback
///
/// Extracts a score value from JSON, handling different field names.
///
/// # Arguments
/// * `item` - JSON value containing the score
/// * `primary_field` - Primary field name to check
/// * `fallback_field` - Fallback field name if primary is missing
///
/// # Returns
/// Score as f64, or 0.0 if not found
pub fn parse_score_from_json(
    item: &serde_json::Value,
    primary_field: &str,
    fallback_field: &str,
) -> f64 {
    item[primary_field]
        .as_f64()
        .or_else(|| item[fallback_field].as_f64())
        .unwrap_or(0.0)
}
