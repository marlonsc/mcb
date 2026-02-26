use mcb_server::args::{IndexAction, IndexArgs};
use mcb_server::handlers::IndexHandler;
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;

use crate::utils::domain_services::create_real_domain_services;

#[rstest]
#[case(Some("test_collection".to_owned()), true)]
#[case(None, false)]
#[tokio::test]
async fn test_clear_index(#[case] collection: Option<String>, #[case] should_succeed: bool) {
    let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
        return;
    };
    let handler = IndexHandler::new(state.mcp_server.indexing_service());

    let args = IndexArgs {
        action: IndexAction::Clear,
        path: None,
        collection,
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    if should_succeed {
        let response = result.expect("index handler should succeed for clear with collection");
        assert!(!response.content.is_empty(), "response should have content");
        assert!(!response.is_error.unwrap_or(false));
    } else {
        let err = result.expect_err("index handler should fail when clear collection is missing");
        let err_str = err.to_string();
        assert!(
            err_str.contains("collection")
                || err_str.contains("missing")
                || err_str.contains("required"),
            "error should mention missing collection, got: {err_str}"
        );
    }
}
