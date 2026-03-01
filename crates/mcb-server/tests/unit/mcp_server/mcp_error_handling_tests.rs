//! MCP Error Handling Tests
//!
//! Comprehensive tests that validate BOTH the `is_error` flag AND the content
//! of error/success messages for MCP compliance.
//!
//! Phase 2 of v0.1.2: These tests verify that errors return `is_error: Some(true)`
//! and contain proper troubleshooting information.

use std::path::Path;
use std::time::Duration;

use mcb_domain::ports::{IndexingResult, IndexingStatus};
use mcb_server::formatter::ResponseFormatter;
use rstest::rstest;

use crate::utils::search_fixtures::{create_test_search_result, create_test_search_results};

// =============================================================================
// ERROR RESPONSE TESTS
// =============================================================================

#[rstest]
#[case("Path does not exist", "/nonexistent/path", true, vec!["Path does not exist"])]
#[case("Directory not found", "/nonexistent/path", true, vec!["Directory not found"])]
#[case("Storage quota exceeded", "/some/path", true, vec!["Troubleshooting", "Verify the directory", "file permissions", "supported file types"])]
#[case("Parse error", "/some/path", true, vec!["Supported Languages", "Rust", "Python", "JavaScript"])]
#[case("Any error", "/some/path", true, vec!["Failed"])]
fn test_format_indexing_error(
    #[case] message: &str,
    #[case] path_str: &str,
    #[case] expected_is_error: bool,
    #[case] expected_content: Vec<&str>,
) {
    let path = Path::new(path_str);
    let response = ResponseFormatter::format_indexing_error(message, path);

    assert_eq!(
        response.is_error.unwrap_or(false),
        expected_is_error,
        "is_error mismatch"
    );

    let text = extract_text(&response.content);
    for content in expected_content {
        assert!(
            text.contains(content),
            "Expected '{content}' in response text. Got: {text}"
        );
    }
}

// =============================================================================
// SUCCESS RESPONSE TESTS
// =============================================================================

#[rstest]
#[case(
    IndexingResult {
        files_processed: 50,
        chunks_created: 250,
        files_skipped: 5,
        errors: Vec::new(),
        operation_id: None,
        status: "completed".to_owned(),
    },
    "/project/src",
    Duration::from_secs(10),
    false,
    vec!["50", "250", "5", "/project/src", "10", "search"]
)]
#[case(
    IndexingResult {
        files_processed: 45,
        chunks_created: 200,
        files_skipped: 10,
        errors: vec![
            "Failed to parse binary.bin".to_owned(),
            "Encoding error in data.csv".to_owned(),
        ],
        operation_id: None,
        status: "completed".to_owned(),
    },
    "/project",
    Duration::from_secs(5),
    false,
    vec!["binary.bin", "data.csv"]
)]
fn test_format_indexing_success(
    #[case] result: IndexingResult,
    #[case] path_str: &str,
    #[case] duration: Duration,
    #[case] expected_is_error: bool,
    #[case] expected_content: Vec<&str>,
) {
    let path = Path::new(path_str);
    let response = ResponseFormatter::format_indexing_success(&result, path, duration);

    assert_eq!(
        response.is_error.unwrap_or(false),
        expected_is_error,
        "is_error mismatch"
    );

    let text = extract_text(&response.content);
    for content in expected_content {
        assert!(
            text.contains(content),
            "Expected '{content}' in response text. Got: {text}"
        );
    }
}

// =============================================================================
// SEARCH RESPONSE TESTS
// =============================================================================

#[rstest]
#[case(
    "test query",
    create_test_search_results(3),
    Duration::from_millis(150),
    false,
    vec!["test query", "3"]
)]
#[case(
    "find authentication",
    vec![
        create_test_search_result("src/auth/login.rs", "fn login() {}", 0.95, 1),
        create_test_search_result("src/user/profile.rs", "fn get_profile() {}", 0.90, 10),
    ],
    Duration::from_millis(50),
    false,
    vec!["src/auth/login.rs", "src/user/profile.rs"]
)]
#[case(
    "test",
    vec![create_test_search_result("src/main.rs", "fn main() {}", 0.875, 1)],
    Duration::from_millis(50),
    false,
    vec!["0.875"]
)]
#[case(
    "nonexistent",
    vec![],
    Duration::from_millis(50),
    false,
    vec!["No Results Found", "indexed"]
)]
#[case(
    "test",
    create_test_search_results(3),
    Duration::from_secs(2),
    false,
    vec!["Performance"]
)]
fn test_format_search_response(
    #[case] query: &str,
    #[case] results: Vec<mcb_domain::SearchResult>,
    #[case] duration: Duration,
    #[case] expected_is_error: bool,
    #[case] expected_content: Vec<&str>,
) {
    let response = ResponseFormatter::format_search_response(query, &results, duration, 10)
        .expect("Should format successfully");

    assert_eq!(
        response.is_error.unwrap_or(false),
        expected_is_error,
        "is_error mismatch"
    );

    let text = extract_text(&response.content);
    for content in expected_content {
        assert!(
            text.contains(content),
            "Expected '{content}' in response text. Got: {text}"
        );
    }
}

// =============================================================================
// INDEXING STATUS TESTS
// =============================================================================

#[rstest]
#[case(
    IndexingStatus {
        is_indexing: false,
        progress: 0.0,
        current_file: None,
        total_files: 0,
        processed_files: 0,
    },
    false,
    vec!["Idle"]
)]
#[case(
    IndexingStatus {
        is_indexing: true,
        progress: 0.65,
        current_file: Some("src/main.rs".to_owned()),
        total_files: 100,
        processed_files: 65,
    },
    false,
    vec!["65.0%", "src/main.rs", "65", "100"]
)]
fn test_format_indexing_status(
    #[case] status: IndexingStatus,
    #[case] expected_is_error: bool,
    #[case] expected_content: Vec<&str>,
) {
    let response = ResponseFormatter::format_indexing_status(&status);

    assert_eq!(
        response.is_error.unwrap_or(false),
        expected_is_error,
        "is_error mismatch"
    );

    let text = extract_text(&response.content);
    for content in expected_content {
        assert!(
            text.contains(content),
            "Expected '{content}' in response text. Got: {text}"
        );
    }
}

// =============================================================================
// CLEAR INDEX TESTS
// =============================================================================

#[rstest]
#[case("test-collection", false, vec!["test-collection", "Cleared"])]
fn test_format_clear_index(
    #[case] collection: &str,
    #[case] expected_is_error: bool,
    #[case] expected_content: Vec<&str>,
) {
    let response = ResponseFormatter::format_clear_index(collection);

    assert_eq!(
        response.is_error.unwrap_or(false),
        expected_is_error,
        "is_error mismatch"
    );

    let text = extract_text(&response.content);
    for content in expected_content {
        assert!(
            text.contains(content),
            "Expected '{content}' in response text. Got: {text}"
        );
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

use crate::utils::text::extract_text;

mod handler_error_tests {

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    use mcb_server::args::{IndexAction, IndexArgs};
    use mcb_server::handlers::IndexHandler;
    use rmcp::handler::server::wrapper::Parameters;

    async fn create_handler() -> TestResult<IndexHandler> {
        let state = crate::utils::shared_context::shared_mcb_state()?;
        Ok(IndexHandler::new(state.mcp_server.indexing_service()))
    }

    #[tokio::test]
    async fn test_handler_service_error_handling() -> TestResult {
        let handler = create_handler().await?;

        let args = IndexArgs {
            action: IndexAction::Start,
            path: Some("/definitely/nonexistent/mcb-path".to_owned()),
            collection: Some("test".to_owned()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        };

        let result = handler.handle(Parameters(args)).await;

        let err = result.expect_err("expected invalid path to fail");
        let err_str = format!("{err:?}");
        assert!(
            err_str.contains("path")
                || err_str.contains("nonexistent")
                || err_str.contains("not found"),
            "Expected path-related error. Got: {err_str}"
        );
        Ok(())
    }
}
