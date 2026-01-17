//! Tests for authentication infrastructure

use mcb_infrastructure::infrastructure::auth::{AuthServiceInterface, NullAuthService};

#[test]
fn test_null_auth_service_creation() {
    let service = NullAuthService::new();
    // Test that service can be created without panicking
    let _service: Box<dyn AuthServiceInterface> = Box::new(service);
}

#[test]
fn test_null_auth_service_validate_token() {
    let service = NullAuthService::new();

    // Null implementation always returns true
    let result = service.validate_token("any-token");
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_null_auth_service_generate_token() {
    let service = NullAuthService::new();

    let result = service.generate_token("test-subject");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "null-token");
}

#[test]
fn test_null_auth_service_default() {
    let service = NullAuthService::default();

    // Test that default implementation works
    let result = service.validate_token("token");
    assert!(result.is_ok());
}