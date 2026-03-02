use mcb_domain::utils::tests::fixtures::{
    GOLDEN_COLLECTION, SAMPLE_CODEBASE_FILES, create_temp_codebase, create_test_indexing_result,
    golden_content_to_string, golden_count_result_entries, golden_parse_results_found,
    sample_codebase_path,
};
use mcb_domain::utils::text::extract_text;
use rstest::rstest;

#[rstest]
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
    let _ = extract_text as fn(&[rmcp::model::Content]) -> String;

    let (_temp, path) = create_temp_codebase();
    assert!(path.exists());

    let result = create_test_indexing_result(1, 2, 0);
    assert_eq!(result.files_processed, 1);
    assert_eq!(result.chunks_created, 2);
}
