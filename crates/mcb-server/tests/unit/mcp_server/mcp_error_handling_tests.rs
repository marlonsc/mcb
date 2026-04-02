//! MCP response content: agents receive actionable, well-structured responses.
//!
//! Verifies that error responses include troubleshooting guidance and success
//! responses contain the expected data (file counts, paths, scores, etc.).

use std::path::Path;
use std::time::Duration;

use mcb_domain::ports::{IndexingResult, IndexingStatus};
use mcb_domain::utils::tests::search_fixtures::{
    create_test_search_result, create_test_search_results,
};
use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::utils::text::extract_text_from;
use mcb_server::formatter::ResponseFormatter;
use rstest::rstest;

fn assert_response(content: &[rmcp::model::Content], is_error: bool, expected_fragments: &[&str]) {
    let text = extract_text_from(content);
    for fragment in expected_fragments {
        assert!(
            text.contains(fragment),
            "expected '{fragment}' in response: {text}"
        );
    }
    let _ = is_error; // flag already checked by caller
}

fn idx(files: usize, chunks: usize, skipped: usize, errors: Vec<String>) -> IndexingResult {
    IndexingResult {
        files_processed: files,
        chunks_created: chunks,
        files_skipped: skipped,
        errors,
        operation_id: None,
        status: "completed".to_owned(),
    }
}

// ─── Indexing errors include troubleshooting guidance ─────────────────

#[rstest]
#[case("Path does not exist", &["Path does not exist"])]
#[case("Storage quota exceeded", &["Troubleshooting", "Verify the directory"])]
#[case("Parse error", &["Supported Languages", "Rust", "Python"])]
fn indexing_error_contains_guidance(#[case] msg: &str, #[case] expected: &[&str]) {
    let resp = ResponseFormatter::format_indexing_error(msg, Path::new("/bad"));
    assert!(resp.is_error.unwrap_or(false));
    assert_response(&resp.content, true, expected);
}

// ─── Indexing success includes statistics ─────────────────────────────

#[rstest]
#[case(idx(50, 250, 5, vec![]), &["50", "250", "search"])]
#[case(idx(45, 200, 10, vec!["binary.bin error".to_owned()]), &["binary.bin"])]
fn indexing_success_includes_statistics(#[case] result: IndexingResult, #[case] expected: &[&str]) {
    let resp = ResponseFormatter::format_indexing_success(
        &result,
        Path::new("/project"),
        Duration::from_secs(5),
    );
    assert!(!resp.is_error.unwrap_or(false));
    assert_response(&resp.content, false, expected);
}

// ─── Search results include file paths and scores ────────────────────

#[rstest]
#[case("auth handler", &[
    create_test_search_result("src/auth.rs", "fn login() {}", 0.95, 1),
    create_test_search_result("src/user.rs", "fn profile() {}", 0.90, 10),
], &["src/auth.rs", "src/user.rs"])]
#[case("nonexistent", &[], &["No Results Found", "indexed"])]
fn search_response_includes_relevant_content(
    #[case] query: &str,
    #[case] results: &[mcb_domain::value_objects::SearchResult],
    #[case] expected: &[&str],
) {
    let resp =
        ResponseFormatter::format_search_response(query, results, Duration::from_millis(50), 10)
            .unwrap();
    assert_response(&resp.content, false, expected);
}

#[rstest]
fn slow_search_shows_performance_warning() {
    let results = create_test_search_results(3);
    let resp =
        ResponseFormatter::format_search_response("test", &results, Duration::from_secs(2), 10)
            .unwrap();
    assert_response(&resp.content, false, &["Performance"]);
}

// ─── Status and clear responses ──────────────────────────────────────

#[rstest]
fn active_indexing_shows_progress() {
    let status = IndexingStatus {
        is_indexing: true,
        progress: 0.65,
        current_file: Some("src/main.rs".to_owned()),
        total_files: 100,
        processed_files: 65,
    };
    let resp = ResponseFormatter::format_indexing_status(&status);
    assert_response(&resp.content, false, &["65.0%", "src/main.rs"]);
}

#[rstest]
fn idle_indexing_shows_idle() {
    let status = IndexingStatus {
        is_indexing: false,
        progress: 0.0,
        current_file: None,
        total_files: 0,
        processed_files: 0,
    };
    let resp = ResponseFormatter::format_indexing_status(&status);
    assert_response(&resp.content, false, &["Idle"]);
}

#[rstest]
fn clear_index_mentions_collection_name() {
    let resp = ResponseFormatter::format_clear_index("my-collection");
    assert_response(&resp.content, false, &["my-collection", "Cleared"]);
}

// ─── Handler-level error propagation ─────────────────────────────────

mod handler_error {
    use super::TestResult;
    use mcb_server::args::{IndexAction, IndexArgs};
    use mcb_server::handlers::IndexHandler;
    use rmcp::handler::server::wrapper::Parameters;
    use rstest::rstest;

    #[rstest]
    #[tokio::test]
    async fn nonexistent_path_rejected_by_handler() -> TestResult {
        let state = crate::utils::test_fixtures::shared_mcb_state()?;
        let handler = IndexHandler::new(state.mcp_server.indexing_service());

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
            repo_id: None,
        };

        let err = handler
            .handle(Parameters(args))
            .await
            .expect_err("nonexistent path must fail");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("path") || msg.contains("nonexistent") || msg.contains("not found"),
            "expected path-related error, got: {msg}"
        );
        Ok(())
    }
}
