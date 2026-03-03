//! MCP text extraction utilities.
//!
//! Provides functions for extracting text segments from MCP content values.
//! These are domain-level utilities used across the application for processing
//! MCP protocol responses.
//!
//! The functions accept `serde_json::Value` slices, keeping the domain layer
//! free from protocol-specific types like `rmcp::model::Content`.

/// Concatenate all text segments from a slice of JSON content values
/// using a custom separator.
///
/// Each value is expected to be an object containing a `"text"` field.
/// Non-text values (images, resources, etc.) are silently skipped.
#[must_use]
pub fn extract_text_with_sep(content: &[serde_json::Value], sep: &str) -> String {
    content
        .iter()
        .filter_map(|v| {
            v.get("text")
                .and_then(|t| t.as_str())
                .map(ToOwned::to_owned)
        })
        .collect::<Vec<_>>()
        .join(sep)
}

/// Concatenate all text segments from a slice of JSON content values,
/// separated by newlines.
#[must_use]
pub fn extract_text(content: &[serde_json::Value]) -> String {
    extract_text_with_sep(content, "\n")
}
