//! MCP text extraction utilities.
//!
//! Provides functions for extracting text segments from MCP content values.
//! These are domain-level utilities used across the application for processing
//! MCP protocol responses.
//!
//! The functions accept `serde_json::Value` slices, keeping the domain layer
//! free from protocol-specific types like `rmcp::model::Content`.

use serde::Serialize;

/// Concatenate all text segments from a slice of JSON content values
/// using a custom separator.
///
/// Each value is expected to be an object containing a `"text"` field.
/// Non-text values (images, resources, etc.) are silently skipped.
#[must_use]
pub fn extract_text_with_sep(content: &[serde_json::Value], sep: &str) -> String {
    let mut result = String::new();
    let mut first = true;

    for value in content {
        if let Some(text) = value.get("text").and_then(|t| t.as_str()) {
            if !first {
                result.push_str(sep);
            }
            first = false;
            result.push_str(text);
        }
    }

    result
}

/// Concatenate all text segments from a slice of JSON content values,
/// separated by newlines.
#[must_use]
pub fn extract_text(content: &[serde_json::Value]) -> String {
    extract_text_with_sep(content, "\n")
}

/// Extract text from any serializable MCP content sequence, returning a
/// `Result` to surface serialization failures to callers.
///
/// Serializes the content to JSON, then extracts `"text"` fields.
/// This allows callers to pass protocol-specific types (e.g. `rmcp::model::Content`)
/// without the domain layer depending on those types directly.
#[must_use]
pub fn try_extract_text_from<T: Serialize>(
    content: &[T],
) -> Result<String, serde_json::Error> {
    let values: Result<Vec<serde_json::Value>, serde_json::Error> = content
        .iter()
        .map(serde_json::to_value)
        .collect();

    values.map(|vals| extract_text(&vals))
}

/// Extract text from any serializable MCP content sequence.
///
/// This is a convenience wrapper around [`try_extract_text_from`] that
/// logs serialization failures and returns an empty string on error.
#[must_use]
pub fn extract_text_from<T: Serialize>(content: &[T]) -> String {
    match try_extract_text_from(content) {
        Ok(text) => text,
        Err(err) => {
            eprintln!(
                "mcb-domain: failed to serialize MCP content for text extraction: {}",
                err
            );
            String::new()
        }
    }
}
