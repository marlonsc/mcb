//! HTTP header extraction utilities for workspace provenance.
//!
//! **Documentation**: [docs/modules/server.md](../../../docs/modules/server.md)
//!
//! Provides utilities for extracting and mapping custom HTTP headers to
//! execution context overrides for workspace provenance enforcement.

use std::collections::HashMap;

use axum::http::HeaderMap;

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
/// Maps custom headers to their corresponding context keys:
/// - X-Workspace-Root → `workspace_root`
/// - X-Repo-Path → `repo_path`
/// - X-Repo-Id → `repo_id`
/// - X-Session-Id → `session_id`
/// - X-Parent-Session-Id → `parent_session_id`
/// - X-Project-Id → `project_id`
/// - X-Worktree-Id → `worktree_id`
/// - X-Operator-Id → `operator_id`
/// - X-Machine-Id → `machine_id`
/// - X-Agent-Program → `agent_program`
/// - X-Model-Id → `model_id`
/// - X-Delegated → delegated
/// - X-Execution-Flow → `execution_flow`
#[must_use]
pub fn build_overrides(headers: &HeaderMap) -> HashMap<String, String> {
    let mut overrides = HashMap::new();
    let mappings = [
        ("X-Workspace-Root", "workspace_root"),
        ("X-Repo-Path", "repo_path"),
        ("X-Repo-Id", "repo_id"),
        ("X-Session-Id", "session_id"),
        ("X-Parent-Session-Id", "parent_session_id"),
        ("X-Project-Id", "project_id"),
        ("X-Worktree-Id", "worktree_id"),
        ("X-Operator-Id", "operator_id"),
        ("X-Machine-Id", "machine_id"),
        ("X-Agent-Program", "agent_program"),
        ("X-Model-Id", "model_id"),
        ("X-Delegated", "delegated"),
        ("X-Execution-Flow", "execution_flow"),
    ];

    for (header_name, key) in mappings {
        if let Some(value) = extract_override(headers, header_name) {
            overrides.insert(key.to_owned(), value);
        }
    }

    overrides
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_override_present() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Workspace-Root", HeaderValue::from_static("/workspace"));
        let result = extract_override(&headers, "X-Workspace-Root");
        assert_eq!(result, Some("/workspace".to_owned()));
    }

    #[test]
    fn test_extract_override_missing() {
        let headers = HeaderMap::new();
        let result = extract_override(&headers, "X-Workspace-Root");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_override_whitespace_trimmed() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Workspace-Root",
            HeaderValue::from_static("  /workspace  "),
        );
        let result = extract_override(&headers, "X-Workspace-Root");
        assert_eq!(result, Some("/workspace".to_owned()));
    }

    #[test]
    fn test_build_overrides_multiple_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Workspace-Root", HeaderValue::from_static("/workspace"));
        headers.insert("X-Repo-Path", HeaderValue::from_static("/repo"));
        headers.insert("X-Session-Id", HeaderValue::from_static("sess-123"));

        let overrides = build_overrides(&headers);
        assert_eq!(
            overrides.get("workspace_root"),
            Some(&"/workspace".to_owned())
        );
        assert_eq!(overrides.get("repo_path"), Some(&"/repo".to_owned()));
        assert_eq!(overrides.get("session_id"), Some(&"sess-123".to_owned()));
    }

    #[test]
    fn test_build_overrides_empty_headers() {
        let headers = HeaderMap::new();
        let overrides = build_overrides(&headers);
        assert!(overrides.is_empty());
    }
}
