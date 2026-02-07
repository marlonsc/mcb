use mcb_server::args::{IndexAction, IndexArgs};
use mcb_server::handlers::IndexHandler;
use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

use crate::test_utils::mock_services::MockIndexingService;

#[tokio::test]
async fn test_get_indexing_status_success() {
    let mock_service = MockIndexingService::new();
    let handler = IndexHandler::new(Arc::new(mock_service));

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
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}
