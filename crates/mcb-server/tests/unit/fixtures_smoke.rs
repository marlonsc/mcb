//! Ensures test_fixtures and mock_services symbols are used when building the unit test binary,
//! so they are not reported as dead code. Integration tests use them; this keeps unit build clean.

use crate::test_utils::test_fixtures::{
    GOLDEN_COLLECTION, SAMPLE_CODEBASE_FILES, create_idle_status, create_in_progress_status,
    create_temp_codebase, create_test_indexing_result, create_test_indexing_result_with_errors,
    golden_content_to_string, golden_count_result_entries, golden_parse_indexing_stats,
    golden_parse_results_found, sample_codebase_path,
};

#[test]
fn test_fixtures_referenced() {
    let _ = GOLDEN_COLLECTION;
    let _ = SAMPLE_CODEBASE_FILES;
    let _ = sample_codebase_path();
    let _ = golden_parse_results_found("**Results found:** 0");
    let _ = golden_count_result_entries("📁 foo");
    let _ = golden_parse_indexing_stats("Files processed: 1\nChunks created: 2");
    let _ = golden_content_to_string;
    let (_temp, path) = create_temp_codebase();
    let _ = path;
    let _ = create_test_indexing_result(1, 2, 0);
    let _ = create_test_indexing_result_with_errors(1, 2, vec![]);
    let _ = create_idle_status();
    let _ = create_in_progress_status(0.5, "x");
}
