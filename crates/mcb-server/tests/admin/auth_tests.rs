//! Tests for AdminAuthConfig
//!
//! Tests admin authentication configuration and validation.

use mcb_server::admin::auth::{AdminAuthConfig, is_unauthenticated_route};
use rstest::rstest;

#[test]
fn test_admin_auth_config_default() {
    let config = AdminAuthConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.header_name, "X-Admin-Key");
    assert!(config.api_key.is_none());
}

#[rstest]
#[case(Some("secret-key"), "secret-key", true, true)]
#[case(Some("secret-key"), "wrong-key", false, true)]
#[case(None, "any-key", false, false)]
#[test]
fn test_admin_auth_key_validation(
    #[case] api_key: Option<&str>,
    #[case] candidate_key: &str,
    #[case] expected_valid: bool,
    #[case] expected_configured: bool,
) {
    let config = AdminAuthConfig {
        enabled: true,
        header_name: "X-Admin-Key".to_string(),
        api_key: api_key.map(std::string::ToString::to_string),
    };

    assert_eq!(config.validate_key(candidate_key), expected_valid);
    assert_eq!(config.is_configured(), expected_configured);
}

#[rstest]
#[case("/live", true)]
#[case("/ready", true)]
#[case("/health", false)]
#[case("/config", false)]
#[case("/metrics", false)]
#[case("/shutdown", false)]
#[test]
fn test_is_unauthenticated_route(#[case] path: &str, #[case] expected: bool) {
    assert_eq!(is_unauthenticated_route(path), expected);
}
