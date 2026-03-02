use mcb_server::args::{IndexAction, IndexArgs};
use mcb_server::handlers::IndexHandler;
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;

use crate::utils::test_fixtures::create_temp_codebase;
use crate::utils::test_fixtures::create_test_mcb_state;

#[rstest]
#[case(true, None, Some("test"), true)]
#[case(false, Some("/nonexistent/path/to/codebase"), Some("test"), false)]
#[case(false, None, Some("test"), true)]
#[case(true, None, None, true)]
#[case(false, Some("/definitely/nonexistent/mcb-path"), Some("test"), false)]
#[rstest]
#[tokio::test]
async fn test_index_codebase(
    #[case] create_codebase: bool,
    #[case] path_override: Option<&str>,
    #[case] collection: Option<&str>,
    #[case] should_succeed: bool,
) {
    let Some((state, _services_temp_dir)) = create_test_mcb_state().await else {
        return;
    };
    let handler = IndexHandler::new(state.mcp_server.indexing_service());

    let _temp_dir_guard;
    let path_val = if create_codebase {
        let (td, p) = create_temp_codebase();
        _temp_dir_guard = Some(td);
        Some(p.to_string_lossy().to_string())
    } else {
        _temp_dir_guard = None;
        path_override.map(std::borrow::ToOwned::to_owned)
    };

    let args = IndexArgs {
        action: IndexAction::Start,
        path: path_val,
        collection: collection.map(std::borrow::ToOwned::to_owned),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
        repo_id: None,
    };

    let result = handler.handle(Parameters(args)).await;

    if should_succeed {
        let response = result.expect("index handler should succeed for valid start request");
        assert!(!response.content.is_empty(), "response should have content");
        assert!(!response.is_error.unwrap_or(false));
    } else {
        let err = result.expect_err("index handler should fail for invalid start request");
        let err_str = err.to_string();
        assert!(
            err_str.contains("path")
                || err_str.contains("collection")
                || err_str.contains("not found")
                || err_str.contains("invalid"),
            "error should mention invalid indexing inputs, got: {err_str}"
        );
    }
}
