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

/// Extract text from any serializable MCP content sequence.
///
/// Serializes the content to JSON, then extracts `"text"` fields.
/// This allows callers to pass protocol-specific types (e.g. `rmcp::model::Content`)
/// without the domain layer depending on those types directly.
#[must_use]
pub fn extract_text_from<T: Serialize>(content: &[T]) -> String {
    let values: Vec<serde_json::Value> = content
        .iter()
        .filter_map(|c| serde_json::to_value(c).ok())
        .collect();
    extract_text(&values)
}
