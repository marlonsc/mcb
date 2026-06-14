//! Unit tests for domain constants

use mcb_utils::constants::values::INDEXING_CHUNKS_MAX_PER_FILE;
use rstest::rstest;

#[rstest]
#[case(INDEXING_CHUNKS_MAX_PER_FILE, 50)]
fn test_indexing_constant_values(#[case] constant: usize, #[case] expected: usize) {
    assert_eq!(constant, expected);
}

#[rstest]
fn test_indexing_constants_relationships() {
    const { assert!(INDEXING_CHUNKS_MAX_PER_FILE >= 10) };
    const { assert!(INDEXING_CHUNKS_MAX_PER_FILE <= 200) };
}
