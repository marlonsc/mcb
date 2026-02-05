use mcb_server::args::{ValidateAction, ValidateArgs, ValidateScope};
use mcb_server::handlers::ValidateHandler;
use rmcp::handler::server::wrapper::Parameters;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

use crate::test_utils::mock_services::MockValidationService;

fn create_temp_file() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, "fn main() { println!(\"test\"); }").expect("Failed to write file");
    (temp_dir, file_path)
}

fn create_temp_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dir_path = temp_dir.path().to_path_buf();
    (temp_dir, dir_path)
}

#[tokio::test]
async fn test_validate_run_with_valid_file() {
    let (_temp_dir, file_path) = create_temp_file();
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Run,
        path: Some(file_path.to_string_lossy().to_string()),
        scope: Some(ValidateScope::File),
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_validate_run_with_nonexistent_path() {
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Run,
        path: Some("/nonexistent/path/to/file.rs".to_string()),
        scope: Some(ValidateScope::File),
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Nonexistent path should return error"
    );
}

#[tokio::test]
async fn test_validate_run_missing_path() {
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Run,
        path: None,
        scope: Some(ValidateScope::File),
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing path should return error"
    );
}

#[tokio::test]
async fn test_validate_run_with_project_scope() {
    let (_temp_dir, dir_path) = create_temp_dir();
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Run,
        path: Some(dir_path.to_string_lossy().to_string()),
        scope: Some(ValidateScope::Project),
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_validate_run_auto_detect_scope_file() {
    let (_temp_dir, file_path) = create_temp_file();
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Run,
        path: Some(file_path.to_string_lossy().to_string()),
        scope: None,
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_validate_run_auto_detect_scope_project() {
    let (_temp_dir, dir_path) = create_temp_dir();
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Run,
        path: Some(dir_path.to_string_lossy().to_string()),
        scope: None,
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_validate_run_with_specific_rules() {
    let (_temp_dir, file_path) = create_temp_file();
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Run,
        path: Some(file_path.to_string_lossy().to_string()),
        scope: Some(ValidateScope::File),
        rules: Some(vec!["rule1".to_string(), "rule2".to_string()]),
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_validate_list_rules_all() {
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::ListRules,
        path: None,
        scope: None,
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_validate_list_rules_by_category() {
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::ListRules,
        path: None,
        scope: None,
        rules: None,
        category: Some("style".to_string()),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_validate_analyze_valid_file() {
    let (_temp_dir, file_path) = create_temp_file();
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Analyze,
        path: Some(file_path.to_string_lossy().to_string()),
        scope: None,
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_validate_analyze_nonexistent_file() {
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Analyze,
        path: Some("/nonexistent/file.rs".to_string()),
        scope: None,
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Nonexistent file should return error"
    );
}

#[tokio::test]
async fn test_validate_analyze_directory_should_fail() {
    let (_temp_dir, dir_path) = create_temp_dir();
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Analyze,
        path: Some(dir_path.to_string_lossy().to_string()),
        scope: None,
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Directory should return error for analyze"
    );
}

#[tokio::test]
async fn test_validate_analyze_missing_path() {
    let mock_service = MockValidationService::new();
    let handler = ValidateHandler::new(Arc::new(mock_service));

    let args = ValidateArgs {
        action: ValidateAction::Analyze,
        path: None,
        scope: None,
        rules: None,
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing path should return error"
    );
}
