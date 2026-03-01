//! Tests for `ResponseFormatter`

use std::path::Path;
use std::time::Duration;

use mcb_domain::ports::{IndexingResult, IndexingStatus};
use mcb_server::formatter::ResponseFormatter;
use rstest::rstest;

use crate::utils::search_fixtures::{create_test_search_result, create_test_search_results};

fn build_indexing_result(
    files_processed: usize,
    chunks_created: usize,
    files_skipped: usize,
    errors: Vec<String>,
) -> IndexingResult {
    IndexingResult {
        files_processed,
        chunks_created,
        files_skipped,
        errors,
        operation_id: None,
        status: "completed".to_owned(),
    }
}

#[rstest]
#[case(3, 150, 10)]
#[case(0, 50, 10)]
#[case(5, 2_000, 10)]
#[case(10, 100, 10)]
fn test_format_search_response(#[case] count: usize, #[case] duration_ms: u64, #[case] max: usize) {
    let results = if count == 0 {
        Vec::new()
    } else {
        create_test_search_results(count)
    };
    let duration = Duration::from_millis(duration_ms);
    let response = ResponseFormatter::format_search_response("test query", &results, duration, max);

    assert!(response.is_ok());
    if count > 0 {
        let result = response.expect("Expected successful response");
        assert!(!result.is_error.unwrap_or(false));
    }
}

#[rstest]
#[case(build_indexing_result(50, 250, 5, Vec::new()), "/project/src", 10_000)]
#[case(
    build_indexing_result(
        45,
        200,
        10,
        vec![
            "Failed to parse binary.bin".to_owned(),
            "Encoding error in data.csv".to_owned(),
        ],
    ),
    "/project/src",
    8_000
)]
#[case(build_indexing_result(100, 500, 0, Vec::new()), "/project", 100)]
#[rstest]
#[test]
fn test_format_indexing_success(
    #[case] result: IndexingResult,
    #[case] path: &str,
    #[case] duration_ms: u64,
) {
    let response = ResponseFormatter::format_indexing_success(
        &result,
        Path::new(path),
        Duration::from_millis(duration_ms),
    );
    assert!(!response.is_error.unwrap_or(false));
}

#[rstest]
#[test]
fn test_format_indexing_error() {
    let path = Path::new("/nonexistent/path");

    let response = ResponseFormatter::format_indexing_error("Path does not exist", path);

    // Error responses should have is_error: true (MCP compliance)
    assert!(
        response.is_error.unwrap_or(false),
        "Error response should have is_error: true"
    );
}

#[rstest]
#[case(IndexingStatus {
    is_indexing: false,
    progress: 0.0,
    current_file: None,
    total_files: 0,
    processed_files: 0,
})]
#[case(IndexingStatus {
    is_indexing: true,
    progress: 0.5,
    current_file: Some("src/main.rs".to_owned()),
    total_files: 100,
    processed_files: 50,
})]
#[case(IndexingStatus {
    is_indexing: false,
    progress: 1.0,
    current_file: None,
    total_files: 100,
    processed_files: 100,
})]
#[rstest]
#[test]
fn test_format_indexing_status(#[case] status: IndexingStatus) {
    let response = ResponseFormatter::format_indexing_status(&status);
    assert!(!response.is_error.unwrap_or(false));
}

#[rstest]
#[test]
fn test_format_clear_index() {
    let response = ResponseFormatter::format_clear_index("test-collection");

    assert!(!response.is_error.unwrap_or(false));
}

#[rstest]
#[test]
fn test_format_search_result_code_preview() {
    // Test with Rust code
    let result = create_test_search_result(
        "src/lib.rs",
        "fn main() {\n    println!(\"Hello\");\n}",
        0.95,
        1,
    );
    let results = vec![result];
    let duration = Duration::from_millis(50);

    let response =
        ResponseFormatter::format_search_response("main function", &results, duration, 10);

    assert!(response.is_ok());
}

#[rstest]
#[test]
fn test_format_search_result_long_content() {
    // Create content with more than 10 lines
    let long_content = (0..20)
        .map(|i| format!("line {i} of content"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = create_test_search_result("src/long_file.rs", &long_content, 0.85, 1);
    let results = vec![result];
    let duration = Duration::from_millis(50);

    let response = ResponseFormatter::format_search_response("test", &results, duration, 10);

    assert!(response.is_ok());
    // Preview should be truncated to 10 lines
}
