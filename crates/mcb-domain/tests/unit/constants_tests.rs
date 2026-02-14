//! Unit tests for domain constants

use mcb_domain::{
    INDEXING_BATCH_SIZE, INDEXING_CHUNK_MIN_LENGTH, INDEXING_CHUNK_MIN_LINES,
    INDEXING_CHUNKS_MAX_PER_FILE,
};
use rstest::*;

#[rstest]
#[case(INDEXING_BATCH_SIZE, 10)]
#[case(INDEXING_CHUNK_MIN_LENGTH, 25)]
#[case(INDEXING_CHUNK_MIN_LINES, 2)]
#[case(INDEXING_CHUNKS_MAX_PER_FILE, 50)]
fn test_indexing_constant_values(#[case] constant: usize, #[case] expected: usize) {
    assert_eq!(constant, expected);
}

#[test]
fn test_indexing_constants_relationships() {
    const { assert!(INDEXING_BATCH_SIZE > 0) };
    const { assert!(INDEXING_BATCH_SIZE <= 100) };
    const { assert!(INDEXING_CHUNK_MIN_LENGTH >= 10) };
    const { assert!(INDEXING_CHUNK_MIN_LINES >= 1) };
    const { assert!(INDEXING_CHUNKS_MAX_PER_FILE >= 10) };
    const { assert!(INDEXING_CHUNKS_MAX_PER_FILE <= 200) };
}

#[rstest]
#[case(INDEXING_BATCH_SIZE, 20)]
fn constants_compile_time_computable(#[case] batch: usize, #[case] expected_double: usize) {
    const _BATCH: usize = INDEXING_BATCH_SIZE;
    assert_eq!(batch * 2, expected_double);
}
