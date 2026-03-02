//! Tests for infrastructure constants
//!
//! Validates that infrastructure constants have reasonable values
//! and maintain expected relationships.

use mcb_domain::constants::embedding::*;
use mcb_infrastructure::constants::auth::*;
use mcb_infrastructure::constants::crypto::*;
use mcb_infrastructure::constants::http::*;
use rstest::rstest;

// ============================================================================
// HTTP Pool Constants Tests
// ============================================================================

#[rstest]
#[case(HTTP_REQUEST_TIMEOUT_SECS, 5, 120)]
#[case(HTTP_CLIENT_IDLE_TIMEOUT_SECS, 30, 300)]
#[case(HTTP_KEEPALIVE_SECS, 30, 120)]
fn test_http_time_constants(#[case] value: u64, #[case] min: u64, #[case] max: u64) {
    assert!(value >= min, "Value {value} too small (< {min})");
    assert!(value <= max, "Value {value} too large (> {max})");
}

#[rstest]
#[case(HTTP_MAX_IDLE_PER_HOST, 5, 50)]
fn test_http_count_constants(#[case] value: usize, #[case] min: usize, #[case] max: usize) {
    assert!(value >= min);
    assert!(value <= max);
}

#[rstest]
fn test_http_timeout_relationships() {
    const { assert!(HTTP_CLIENT_IDLE_TIMEOUT_SECS >= HTTP_REQUEST_TIMEOUT_SECS) };
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

#[rstest]
fn test_embedding_dimension_common_values() {
    let common_dims = [256, 384, 512, 768, 1024, 1536, 2048, 3072];
    assert!(common_dims.contains(&EMBEDDING_DIMENSION_NULL));

    const { assert!(EMBEDDING_DIMENSION_OPENAI_SMALL >= 1024) };
    const { assert!(EMBEDDING_DIMENSION_OPENAI_LARGE > EMBEDDING_DIMENSION_OPENAI_SMALL) };
    assert_eq!(
        EMBEDDING_DIMENSION_OPENAI_ADA,
        EMBEDDING_DIMENSION_OPENAI_SMALL
    );
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

#[rstest]
fn test_jwt_relationship() {
    const { assert!(JWT_REFRESH_EXPIRATION_SECS > JWT_DEFAULT_EXPIRATION_SECS) };
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

#[rstest]
fn test_bearer_prefix() {
    assert!(BEARER_PREFIX.ends_with(' '));
}

// ============================================================================
// Crypto Constants Tests
// ============================================================================

#[rstest]
fn test_aes_gcm_constants() {
    const { assert!(AES_GCM_KEY_SIZE == 32) };
    const { assert!(AES_GCM_NONCE_SIZE == 12) };
}

#[rstest]
fn test_pbkdf2_iterations() {
    const { assert!(PBKDF2_ITERATIONS >= 10_000) };
}
