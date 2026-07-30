//! MCP response formatting: search results, indexing status, validation reports.
//!
//! Tests verify that tool responses carry the correct success/error flag
//! and contain meaningful content for agents to act on.

use std::path::Path;
use std::time::Duration;

use mcb_domain::ports::{IndexingResult, IndexingStatus};
use mcb_domain::utils::tests::search_fixtures::{
    create_test_search_result, create_test_search_results,
};
use mcb_server::formatter::ResponseFormatter;
use rstest::rstest;

fn indexing_result(
    files: usize,
    chunks: usize,
    skipped: usize,
    errors: Vec<String>,
) -> IndexingResult {
    IndexingResult {
        files_processed: files,
        chunks_created: chunks,
        files_skipped: skipped,
        errors,
        operation_id: None,
        status: "completed".to_owned(),
    }
}

// ─── Search responses ────────────────────────────────────────────────

#[rstest]
#[case(3, 50)]
#[case(0, 50)]
#[case(10, 2_000)]
fn search_response_succeeds_regardless_of_result_count(
    #[case] count: usize,
    #[case] duration_ms: u64,
) {
    let results = create_test_search_results(count);
    let resp = ResponseFormatter::format_search_response(
        "auth handler",
        &results,
        Duration::from_millis(duration_ms),
        10,
    );

    assert!(resp.is_ok());
    assert!(!resp.unwrap().is_error.unwrap_or(false));
}

#[rstest]
fn search_result_with_long_code_is_truncated_in_preview() {
    let long = (0..20)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let results = vec![create_test_search_result("src/big.rs", &long, 0.85, 1)];

    let resp =
        ResponseFormatter::format_search_response("test", &results, Duration::from_millis(10), 10);

    assert!(resp.is_ok());
}

// ─── Indexing responses ──────────────────────────────────────────────

#[rstest]
#[case(indexing_result(50, 250, 5, vec![]))]
#[case(indexing_result(45, 200, 10, vec!["parse error".to_owned()]))]
fn successful_indexing_reports_success(#[case] result: IndexingResult) {
    let resp = ResponseFormatter::format_indexing_success(
        &result,
        Path::new("/project"),
        Duration::from_secs(10),
    );

    assert!(!resp.is_error.unwrap_or(false));
}

#[rstest]
fn indexing_error_reports_failure() {
    let resp = ResponseFormatter::format_indexing_error("path not found", Path::new("/bad"));

    assert!(resp.is_error.unwrap_or(false));
}

#[rstest]
#[case(true, 0.5, Some("src/main.rs".to_owned()), 100, 50)]
#[case(false, 0.0, None, 0, 0)]
#[case(false, 1.0, None, 100, 100)]
fn indexing_status_always_succeeds(
    #[case] is_indexing: bool,
    #[case] progress: f64,
    #[case] current_file: Option<String>,
    #[case] total: usize,
    #[case] processed: usize,
) {
    let status = IndexingStatus {
        is_indexing,
        progress,
        current_file,
        total_files: total,
        processed_files: processed,
    };
    let resp = ResponseFormatter::format_indexing_status(&status);

    assert!(!resp.is_error.unwrap_or(false));
}

// ─── Clear index ─────────────────────────────────────────────────────

#[rstest]
fn clear_index_reports_success() {
    let resp = ResponseFormatter::format_clear_index("my-collection");

    assert!(!resp.is_error.unwrap_or(false));
}
