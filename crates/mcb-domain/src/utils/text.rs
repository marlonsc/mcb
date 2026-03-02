//! MCP text extraction utilities.
//!
//! Provides functions for extracting text segments from MCP `Content` slices.
//! These are domain-level utilities used across the application for processing
//! MCP protocol responses.

use rmcp::model::Content;

/// Concatenate all text segments from an MCP `Content` slice using a custom separator.
#[must_use]
pub fn extract_text_with_sep(content: &[Content], sep: &str) -> String {
    content
        .iter()
        .filter_map(|c| {
            if let Ok(v) = serde_json::to_value(c) {
                v.get("text")
                    .and_then(|t| t.as_str())
                    .map(ToOwned::to_owned)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(sep)
}

/// Concatenate all text segments from an MCP `Content` slice, separated by newlines.
#[must_use]
pub fn extract_text(content: &[Content]) -> String {
    extract_text_with_sep(content, "\n")
}
