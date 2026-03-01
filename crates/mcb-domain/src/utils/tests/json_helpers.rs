//! JSON parsing helpers for test assertions.
//!
//! Centralized in `mcb-domain` so all test crates share the same JSON utilities
//! without duplicating code.

/// Parse a JSON string into a `serde_json::Value`, returning `None` on failure.
#[must_use]
pub fn parse_json_text(text: &str) -> Option<serde_json::Value> {
    serde_json::from_str(text).ok()
}

/// Parse a JSON string into a typed value.
///
/// # Errors
///
/// Returns a `serde_json::Error` if the input is not valid JSON for type `T`.
pub fn parse_json<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(text)
}

/// Extract a `count` field from a JSON string, defaulting to `0`.
#[must_use]
pub fn parse_count_from_json_text(text: &str) -> usize {
    parse_json_text(text)
        .and_then(|v| v.get("count").and_then(serde_json::Value::as_u64))
        .map_or(0, |v| v as usize)
}
