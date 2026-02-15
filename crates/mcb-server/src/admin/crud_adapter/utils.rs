//! Utility functions for CRUD adapter.

use rmcp::model::Content;

/// Concatenate all text segments from an MCP `Content` slice.
pub fn extract_text_content(content: &[Content]) -> String {
    content
        .iter()
        .filter_map(|c| {
            if let Ok(v) = serde_json::to_value(c) {
                v.get("text").and_then(|t| t.as_str()).map(str::to_string)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
