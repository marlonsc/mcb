//! FastEmbed Embedding Provider Tests
//!
//! Note: FastEmbed tests require model download, so most are integration tests.
//! These unit tests just verify the basic structure.

use mcb_infrastructure::constants::EMBEDDING_DIMENSION_FASTEMBED_DEFAULT;

#[test]
fn test_dimensions_constant() {
    // Verify the constant is correct for AllMiniLML6V2 model
    assert_eq!(EMBEDDING_DIMENSION_FASTEMBED_DEFAULT, 384);
}
