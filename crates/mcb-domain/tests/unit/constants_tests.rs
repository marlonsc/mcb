//! Unit tests for domain constants

use mcb_domain::{
    INDEXING_BATCH_SIZE, INDEXING_CHUNK_MIN_LENGTH, INDEXING_CHUNK_MIN_LINES,
    INDEXING_CHUNKS_MAX_PER_FILE,
};

#[test]
fn test_indexing_constants() {
    assert_eq!(INDEXING_BATCH_SIZE, 10);
    assert_eq!(INDEXING_CHUNK_MIN_LENGTH, 25);
    assert_eq!(INDEXING_CHUNK_MIN_LINES, 2);
    assert_eq!(INDEXING_CHUNKS_MAX_PER_FILE, 50);
}

#[test]
fn test_indexing_constants_relationships() {
    let min_len = INDEXING_CHUNK_MIN_LENGTH;
    let min_lines = INDEXING_CHUNK_MIN_LINES;
    let batch = INDEXING_BATCH_SIZE;
    let max_chunks = INDEXING_CHUNKS_MAX_PER_FILE;

    assert!(min_len > 0);
    assert!(min_lines > 0);
    assert!(batch > 0);
    assert!(max_chunks > 0);
    assert!((1..=100).contains(&batch));
    assert!(min_len >= 10);
    assert!(min_lines >= 1);
    assert!((10..=200).contains(&max_chunks));
}

#[test]
fn test_constants_are_compile_time() {
    let batch_size = INDEXING_BATCH_SIZE;
    let min_length = INDEXING_CHUNK_MIN_LENGTH;
    let _min_lines = INDEXING_CHUNK_MIN_LINES;
    let _max_chunks = INDEXING_CHUNKS_MAX_PER_FILE;

    assert_eq!(batch_size * 2, 20);
    assert_eq!(min_length + 5, 30);
}
