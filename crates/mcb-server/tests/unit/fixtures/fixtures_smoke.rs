use mcb_domain::test_fixtures::sample_codebase_path;
use mcb_domain::test_utils::{
    GOLDEN_COLLECTION, SAMPLE_CODEBASE_FILES, create_temp_codebase, create_test_indexing_result,
};
use mcb_domain::utils::tests::mcp_assertions::{
    extract_text as extract_result_text, golden_count_result_entries, golden_parse_results_found,
};
use mcb_domain::utils::text::extract_text_from;
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

    let _ = extract_result_text as fn(&rmcp::model::CallToolResult) -> String;
    let _ = extract_text_from::<rmcp::model::Content> as fn(&[rmcp::model::Content]) -> String;

    let (_temp, path) = create_temp_codebase();
    assert!(path.exists());

    let result = create_test_indexing_result(1, 2, 0);
    assert_eq!(result.files_processed, 1);
    assert_eq!(result.chunks_created, 2);
}
