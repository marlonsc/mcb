use mcb_server::args::{IndexAction, IndexArgs};
use mcb_server::handlers::IndexHandler;
use rmcp::handler::server::wrapper::Parameters;

use crate::utils::test_fixtures::create_test_mcb_state;
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_get_indexing_status_success() {
    let Some((state, _services_temp_dir)) = create_test_mcb_state().await else {
        return;
    };
    let handler = IndexHandler::new(state.mcp_server.indexing_service());

    let args = IndexArgs {
        action: IndexAction::Status,
        path: None,
        collection: None,
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
        repo_id: None,
    };

    let result = handler.handle(Parameters(args)).await;

    let response = result.expect("index handler should succeed for status request");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(!response.is_error.unwrap_or(false));
}
