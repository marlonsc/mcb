//! Tests for infrastructure constants
//!
//! Validates that infrastructure constants have reasonable values
//! and maintain expected relationships.
#![allow(clippy::assertions_on_constants)]

use mcb_infrastructure::constants::auth::*;
use mcb_infrastructure::constants::cache::*;
use mcb_infrastructure::constants::crypto::*;
use mcb_infrastructure::constants::embedding::*;
use mcb_infrastructure::constants::fs::*;
use mcb_infrastructure::constants::health::*;
use mcb_infrastructure::constants::http::*;
use mcb_infrastructure::constants::process::*;
use mcb_infrastructure::constants::resilience::*;
use rstest::*;

// ============================================================================
// HTTP Pool Constants Tests
// ============================================================================

#[rstest]
#[case(HTTP_REQUEST_TIMEOUT_SECS, 5, 120)]
#[case(HTTP_CLIENT_IDLE_TIMEOUT_SECS, 30, 300)]
#[case(HTTP_KEEPALIVE_SECS, 30, 120)]
fn test_http_time_constants(#[case] value: u64, #[case] min: u64, #[case] max: u64) {
    assert!(value >= min, "Value {} too small (< {})", value, min);
    assert!(value <= max, "Value {} too large (> {})", value, max);
}

#[rstest]
#[case(HTTP_MAX_IDLE_PER_HOST, 5, 50)]
fn test_http_count_constants(#[case] value: usize, #[case] min: usize, #[case] max: usize) {
    assert!(value >= min);
    assert!(value <= max);
}

#[test]
fn test_http_timeout_relationships() {
    assert!(
        HTTP_CLIENT_IDLE_TIMEOUT_SECS >= HTTP_REQUEST_TIMEOUT_SECS,
        "Idle timeout should be >= request timeout"
    );
}

// ============================================================================
// Embedding Dimension Constants Tests
// ============================================================================

#[rstest]
#[case(EMBEDDING_DIMENSION_NULL)]
#[case(EMBEDDING_DIMENSION_FASTEMBED_DEFAULT)]
#[case(EMBEDDING_DIMENSION_OPENAI_SMALL)]
#[case(EMBEDDING_DIMENSION_OPENAI_LARGE)]
#[case(EMBEDDING_DIMENSION_OPENAI_ADA)]
#[case(EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT)]
#[case(EMBEDDING_DIMENSION_VOYAGEAI_CODE)]
#[case(EMBEDDING_DIMENSION_OLLAMA_NOMIC)]
#[case(EMBEDDING_DIMENSION_OLLAMA_MINILM)]
#[case(EMBEDDING_DIMENSION_OLLAMA_MXBAI)]
#[case(EMBEDDING_DIMENSION_OLLAMA_ARCTIC)]
#[case(EMBEDDING_DIMENSION_OLLAMA_DEFAULT)]
#[case(EMBEDDING_DIMENSION_GEMINI)]
fn test_embedding_dimensions_positive(#[case] dim: usize) {
    assert!(dim > 0);
}

#[test]
fn test_embedding_dimension_common_values() {
    let common_dims = [256, 384, 512, 768, 1024, 1536, 2048, 3072];
    assert!(common_dims.contains(&EMBEDDING_DIMENSION_NULL));

    assert!(EMBEDDING_DIMENSION_OPENAI_SMALL >= 1024);
    assert!(EMBEDDING_DIMENSION_OPENAI_LARGE > EMBEDDING_DIMENSION_OPENAI_SMALL);
    assert_eq!(
        EMBEDDING_DIMENSION_OPENAI_ADA,
        EMBEDDING_DIMENSION_OPENAI_SMALL
    );
}

// ============================================================================
// Cache Constants Tests
// ============================================================================

#[rstest]
#[case(CACHE_DEFAULT_TTL_SECS, 60, 86400)]
fn test_cache_ttl_range(#[case] value: u64, #[case] min: u64, #[case] max: u64) {
    assert!(value >= min);
    assert!(value <= max);
}

#[rstest]
#[case(CACHE_DEFAULT_SIZE_LIMIT, 1024 * 1024, 1024 * 1024 * 1024)]
fn test_cache_size_range(#[case] value: usize, #[case] min: usize, #[case] max: usize) {
    assert!(value >= min);
    assert!(value <= max);
}

#[test]
fn test_cache_namespace_separator() {
    assert_eq!(CACHE_NAMESPACE_SEPARATOR.len(), 1);
}

// ============================================================================
// Authentication Constants Tests
// ============================================================================

#[rstest]
#[case(JWT_DEFAULT_EXPIRATION_SECS, 3600, 604800)]
fn test_jwt_constants_range(#[case] value: u64, #[case] min: u64, #[case] max: u64) {
    assert!(value >= min);
    assert!(value <= max);
}

#[test]
fn test_jwt_relationship() {
    assert!(JWT_REFRESH_EXPIRATION_SECS > JWT_DEFAULT_EXPIRATION_SECS);
}

#[rstest]
#[case(BCRYPT_DEFAULT_COST, 10, 15)]
fn test_bcrypt_cost_range(#[case] value: u32, #[case] min: u32, #[case] max: u32) {
    assert!(value >= min);
    assert!(value <= max);
}

#[rstest]
#[case(API_KEY_HEADER)]
#[case(AUTHORIZATION_HEADER)]
fn test_auth_headers_lowercase(#[case] header: &str) {
    assert_eq!(header.to_lowercase(), header);
}

#[test]
fn test_bearer_prefix() {
    assert!(BEARER_PREFIX.ends_with(' '));
}

// ============================================================================
// Crypto Constants Tests
// ============================================================================

#[test]
fn test_aes_gcm_constants() {
    assert_eq!(AES_GCM_KEY_SIZE, 32);
    assert_eq!(AES_GCM_NONCE_SIZE, 12);
}

#[test]
fn test_pbkdf2_iterations() {
    assert!(PBKDF2_ITERATIONS >= 10_000);
}

// ============================================================================
// Server Constants Tests
// ============================================================================

#[rstest]
#[case(DEFAULT_HTTP_PORT)]
#[case(DEFAULT_HTTPS_PORT)]
fn test_server_ports(#[case] port: u16) {
    assert!(port > 1024);
    assert!(port < 65535);
}

#[test]
fn test_server_ports_distinct() {
    assert_ne!(DEFAULT_HTTP_PORT, DEFAULT_HTTPS_PORT);
}

#[rstest]
#[case(REQUEST_TIMEOUT_SECS, 10)]
#[case(CONNECTION_TIMEOUT_SECS, 5)]
fn test_server_timeouts(#[case] value: u64, #[case] min: u64) {
    assert!(value >= min);
}

#[test]
fn test_server_timeout_relationship() {
    assert!(REQUEST_TIMEOUT_SECS >= CONNECTION_TIMEOUT_SECS);
}

// ============================================================================
// Health Check Constants Tests
// ============================================================================

#[rstest]
#[case(HEALTH_CHECK_TIMEOUT_SECS, 0, 30)]
#[case(HEALTH_CHECK_INTERVAL_SECS, 10, 300)]
fn test_health_time_constants_range(#[case] value: u64, #[case] min: u64, #[case] max: u64) {
    assert!(value >= min);
    assert!(value <= max);
}

#[rstest]
#[case(HEALTH_CHECK_FAILURE_THRESHOLD, 2, 10)]
fn test_health_count_constants_range(#[case] value: u32, #[case] min: u32, #[case] max: u32) {
    assert!(value >= min);
    assert!(value <= max);
}

// ============================================================================
// Circuit Breaker Constants Tests
// ============================================================================

#[rstest]
#[case(CIRCUIT_BREAKER_FAILURE_THRESHOLD, 3u32)]
fn test_circuit_breaker_min_values(#[case] value: u32, #[case] min: u32) {
    assert!(value >= min);
}

#[test]
fn test_circuit_breaker_time_values() {
    assert!(
        CIRCUIT_BREAKER_TIMEOUT_SECS >= 30,
        "Circuit breaker timeout too short"
    );
}

#[test]
fn test_circuit_breaker_relationships() {
    assert!(CIRCUIT_BREAKER_SUCCESS_THRESHOLD <= CIRCUIT_BREAKER_FAILURE_THRESHOLD);
}

// ============================================================================
// Rate Limiter Constants Tests
// ============================================================================

#[test]
fn test_rate_limiter_constants() {
    assert!(RATE_LIMITER_DEFAULT_RPS >= 10);
    assert!(RATE_LIMITER_DEFAULT_BURST >= RATE_LIMITER_DEFAULT_RPS);
}

// ============================================================================
// File System Constants Tests
// ============================================================================

#[test]
fn test_file_permissions() {
    assert_eq!(DEFAULT_FILE_PERMISSIONS, 0o644);
    assert_eq!(DEFAULT_DIR_PERMISSIONS, 0o755);
}

// ============================================================================
// Shutdown Constants Tests
// ============================================================================

#[rstest]
#[case(GRACEFUL_SHUTDOWN_TIMEOUT_SECS, 120)]
fn test_graceful_shutdown_limits(#[case] value: u64, #[case] max: u64) {
    assert!(value <= max);
}

#[rstest]
#[case(FORCE_SHUTDOWN_TIMEOUT_SECS, 5)]
fn test_force_shutdown_limits(#[case] value: u64, #[case] min: u64) {
    assert!(value >= min);
}

#[test]
fn test_shutdown_relationship() {
    assert!(GRACEFUL_SHUTDOWN_TIMEOUT_SECS > FORCE_SHUTDOWN_TIMEOUT_SECS);
}
