//! Unit tests for admin operations default configuration
//!
//! Tests for centralized default values for admin service operations.

use mcp_context_browser::application::admin::helpers::admin_defaults::{
    get_env_u32, get_env_u64, get_env_usize, BYTES_PER_GIGABYTE, BYTES_PER_KILOBYTE,
    BYTES_PER_MEGABYTE,
};

#[test]
fn test_get_env_usize_uses_default_when_not_set() {
    let result = get_env_usize("NONEXISTENT_ADMIN_VAR_12345", 100);
    assert_eq!(result, 100);
}

#[test]
fn test_get_env_usize_parses_valid_value() {
    std::env::set_var("TEST_ADMIN_USIZE", "250");
    let result = get_env_usize("TEST_ADMIN_USIZE", 100);
    assert_eq!(result, 250);
    std::env::remove_var("TEST_ADMIN_USIZE");
}

#[test]
fn test_get_env_usize_uses_default_on_invalid_value() {
    std::env::set_var("TEST_ADMIN_USIZE_INVALID", "not_a_number");
    let result = get_env_usize("TEST_ADMIN_USIZE_INVALID", 100);
    assert_eq!(result, 100);
    std::env::remove_var("TEST_ADMIN_USIZE_INVALID");
}

#[test]
fn test_get_env_usize_handles_empty_string() {
    std::env::set_var("TEST_ADMIN_USIZE_EMPTY", "");
    let result = get_env_usize("TEST_ADMIN_USIZE_EMPTY", 100);
    assert_eq!(result, 100);
    std::env::remove_var("TEST_ADMIN_USIZE_EMPTY");
}

#[test]
fn test_get_env_u32_uses_default_when_not_set() {
    let result = get_env_u32("NONEXISTENT_ADMIN_VAR_U32_12345", 100);
    assert_eq!(result, 100);
}

#[test]
fn test_get_env_u32_parses_valid_value() {
    std::env::set_var("TEST_ADMIN_U32", "500");
    let result = get_env_u32("TEST_ADMIN_U32", 100);
    assert_eq!(result, 500);
    std::env::remove_var("TEST_ADMIN_U32");
}

#[test]
fn test_get_env_u32_uses_default_on_invalid_value() {
    std::env::set_var("TEST_ADMIN_U32_INVALID", "abc123");
    let result = get_env_u32("TEST_ADMIN_U32_INVALID", 100);
    assert_eq!(result, 100);
    std::env::remove_var("TEST_ADMIN_U32_INVALID");
}

#[test]
fn test_get_env_u64_uses_default_when_not_set() {
    let result = get_env_u64("NONEXISTENT_ADMIN_VAR_U64_12345", 3600);
    assert_eq!(result, 3600);
}

#[test]
fn test_get_env_u64_parses_valid_value() {
    std::env::set_var("TEST_ADMIN_U64", "7200");
    let result = get_env_u64("TEST_ADMIN_U64", 3600);
    assert_eq!(result, 7200);
    std::env::remove_var("TEST_ADMIN_U64");
}

#[test]
fn test_get_env_u64_uses_default_on_invalid_value() {
    std::env::set_var("TEST_ADMIN_U64_INVALID", "xyz");
    let result = get_env_u64("TEST_ADMIN_U64_INVALID", 3600);
    assert_eq!(result, 3600);
    std::env::remove_var("TEST_ADMIN_U64_INVALID");
}

#[test]
fn test_byte_conversion_constants() {
    assert_eq!(BYTES_PER_KILOBYTE, 1024);
    assert_eq!(BYTES_PER_MEGABYTE, 1024 * 1024);
    assert_eq!(BYTES_PER_GIGABYTE, 1024 * 1024 * 1024);
}
