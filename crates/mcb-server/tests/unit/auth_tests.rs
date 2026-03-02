use axum::http::{HeaderMap, HeaderValue};
use mcb_server::auth::extract_api_key;
use rstest::rstest;

#[rstest]
#[test]
fn extract_api_key_reads_x_api_key() {
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_static("abc123"));
    assert_eq!(
        extract_api_key(&headers, "x-api-key").expect("api key"),
        "abc123"
    );
}

#[rstest]
#[test]
fn extract_api_key_reads_authorization_bearer() {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", HeaderValue::from_static("Bearer abc123"));
    assert_eq!(
        extract_api_key(&headers, "x-api-key").expect("api key"),
        "abc123"
    );
}

#[rstest]
#[test]
fn extract_api_key_rejects_missing_headers() {
    let headers = HeaderMap::new();
    assert!(extract_api_key(&headers, "x-api-key").is_err());
}
