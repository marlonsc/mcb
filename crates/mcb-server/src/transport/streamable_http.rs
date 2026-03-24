//! HTTP header extraction utilities for workspace provenance.
//!
//! **Documentation**: [docs/modules/server.md](../../../docs/modules/server.md)
//!
//! Provides utilities for extracting and mapping custom HTTP headers to
//! execution context overrides for workspace provenance enforcement.

use std::collections::HashMap;

use axum::http::HeaderMap;
use mcb_utils::constants::headers::PROVENANCE_HEADER_MAPPINGS;
use mcb_utils::constants::protocol::HTTP_HEADER_EXECUTION_FLOW;

/// Extract a single header value, trimming whitespace.
pub fn extract_override(headers: &HeaderMap, header_name: &str) -> Option<String> {
    headers
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

/// Build a `HashMap` of header overrides from HTTP headers.
///
/// Maps custom headers to their corresponding context keys using the
/// centralized `PROVENANCE_HEADER_MAPPINGS` table plus execution flow.
#[must_use]
pub fn build_overrides(headers: &HeaderMap) -> HashMap<String, String> {
    let mut overrides = HashMap::new();

    for &(header_name, key) in PROVENANCE_HEADER_MAPPINGS {
        if let Some(value) = extract_override(headers, header_name) {
            overrides.insert(key.to_owned(), value);
        }
    }

    // Execution flow is transport-level, not provenance, but still overridden via header.
    if let Some(value) = extract_override(headers, HTTP_HEADER_EXECUTION_FLOW) {
        overrides.insert("execution_flow".to_owned(), value);
    }

    overrides
}
