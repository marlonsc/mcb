//! Ensures test_fixtures and mock_services symbols are used when building the unit test binary,
//! so they are not reported as dead code. Integration tests use them; this keeps unit build clean.

use crate::test_utils::test_fixtures::{
    GOLDEN_COLLECTION, SAMPLE_CODEBASE_FILES, create_temp_codebase, create_test_indexing_result,
    golden_content_to_string, golden_count_result_entries, golden_parse_results_found,
    sample_codebase_path,
};

#[test]
fn test_fixtures_referenced() {
    assert!(!GOLDEN_COLLECTION.is_empty());
    assert!(!SAMPLE_CODEBASE_FILES.is_empty());
    assert!(
        sample_codebase_path()
            .to_string_lossy()
            .contains("sample_codebase")
    );
    assert_eq!(golden_parse_results_found("**Results found:** 5"), Some(5));
    assert_eq!(golden_count_result_entries("📁 foo\n📁 bar"), 2);

    let _ = golden_content_to_string as fn(&rmcp::model::CallToolResult) -> String;

    let (_temp, path) = create_temp_codebase();
    assert!(path.exists());

    let result = create_test_indexing_result(1, 2, 0);
    assert_eq!(result.files_processed, 1);
    assert_eq!(result.chunks_created, 2);
}
