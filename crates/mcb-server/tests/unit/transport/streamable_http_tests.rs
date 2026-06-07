use axum::http::{HeaderMap, HeaderValue};
use mcb_server::transport::streamable_http::{build_overrides, extract_override};
use mcb_utils::constants::headers::{HEADER_REPO_PATH, HEADER_SESSION_ID, HEADER_WORKSPACE_ROOT};
use rstest::rstest;

#[rstest]
fn test_extract_override_present() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HEADER_WORKSPACE_ROOT,
        HeaderValue::from_static("/workspace"),
    );
    let result = extract_override(&headers, HEADER_WORKSPACE_ROOT);
    assert_eq!(result, Some("/workspace".to_owned()));
}

#[rstest]
fn test_extract_override_missing() {
    let headers = HeaderMap::new();
    let result = extract_override(&headers, HEADER_WORKSPACE_ROOT);
    assert_eq!(result, None);
}

#[rstest]
fn test_extract_override_whitespace_trimmed() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HEADER_WORKSPACE_ROOT,
        HeaderValue::from_static("  /workspace  "),
    );
    let result = extract_override(&headers, HEADER_WORKSPACE_ROOT);
    assert_eq!(result, Some("/workspace".to_owned()));
}

#[rstest]
fn test_build_overrides_multiple_headers() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HEADER_WORKSPACE_ROOT,
        HeaderValue::from_static("/workspace"),
    );
    headers.insert(HEADER_REPO_PATH, HeaderValue::from_static("/repo"));
    headers.insert(HEADER_SESSION_ID, HeaderValue::from_static("sess-123"));

    let overrides = build_overrides(&headers);
    assert_eq!(
        overrides.get("workspace_root"),
        Some(&"/workspace".to_owned())
    );
    assert_eq!(overrides.get("repo_path"), Some(&"/repo".to_owned()));
    assert_eq!(overrides.get("session_id"), Some(&"sess-123".to_owned()));
}

#[rstest]
fn test_build_overrides_empty_headers() {
    let headers = HeaderMap::new();
    let overrides = build_overrides(&headers);
    assert!(overrides.is_empty());
}
