//! Embedding Provider Helpers Tests

use mcb_infrastructure::adapters::providers::embedding::helpers::constructor;

#[test]
fn test_validate_api_key() {
    assert_eq!(constructor::validate_api_key("  key  "), "key");
    assert_eq!(constructor::validate_api_key("key"), "key");
}

#[test]
fn test_validate_url() {
    assert_eq!(
        constructor::validate_url(Some("  https://api.example.com  ".to_string())),
        Some("https://api.example.com".to_string())
    );
    assert_eq!(constructor::validate_url(None), None);
}

#[test]
fn test_get_effective_url() {
    assert_eq!(
        constructor::get_effective_url(Some("https://custom.com"), "https://default.com"),
        "https://custom.com"
    );
    assert_eq!(
        constructor::get_effective_url(None, "https://default.com"),
        "https://default.com"
    );
}

#[test]
fn test_default_timeout() {
    let timeout = constructor::default_timeout();
    assert_eq!(timeout.as_secs(), 30);
}
