use std::sync::Arc;

use mcb_server::args::{IndexAction, IndexArgs};
use mcb_server::handlers::IndexHandler;
use rmcp::handler::server::wrapper::Parameters;

use crate::test_utils::mock_services::MockIndexingService;
use crate::test_utils::test_fixtures::{create_temp_codebase, create_test_indexing_result};

#[tokio::test]
async fn test_index_codebase_valid_path() {
    let (_temp_dir, codebase_path) = create_temp_codebase();
    let indexing_result = create_test_indexing_result(10, 50, 2);

    let mock_service = MockIndexingService::new().with_result(indexing_result);
    let handler = IndexHandler::new(Arc::new(mock_service));

    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some(codebase_path.to_string_lossy().to_string()),
        collection: Some("test".to_string()),
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

#[tokio::test]
async fn test_index_codebase_nonexistent_path() {
    let mock_service = MockIndexingService::new();
    let handler = IndexHandler::new(Arc::new(mock_service));

    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some("/nonexistent/path/to/codebase".to_string()),
        collection: None,
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_index_codebase_empty_path() {
    let mock_service = MockIndexingService::new();
    let handler = IndexHandler::new(Arc::new(mock_service));

    let args = IndexArgs {
        action: IndexAction::Start,
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

    assert!(result.is_err());
}

#[tokio::test]
async fn test_index_codebase_default_collection() {
    let (_temp_dir, codebase_path) = create_temp_codebase();
    let indexing_result = create_test_indexing_result(5, 20, 0);

    let mock_service = MockIndexingService::new().with_result(indexing_result);
    let handler = IndexHandler::new(Arc::new(mock_service));

    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some(codebase_path.to_string_lossy().to_string()),
        collection: None,
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_index_codebase_service_error() {
    let (_temp_dir, codebase_path) = create_temp_codebase();

    let mock_service = MockIndexingService::new().with_failure("Test error");
    let handler = IndexHandler::new(Arc::new(mock_service));

    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some(codebase_path.to_string_lossy().to_string()),
        collection: Some("test".to_string()),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_index_codebase_with_errors() {
    let (_temp_dir, codebase_path) = create_temp_codebase();
    let indexing_result = create_test_indexing_result(2, 8, 1);

    let mock_service = MockIndexingService::new().with_result(indexing_result);
    let handler = IndexHandler::new(Arc::new(mock_service));

    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some(codebase_path.to_string_lossy().to_string()),
        collection: Some("test".to_string()),
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
