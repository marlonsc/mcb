use std::fs;
use std::path::PathBuf;

use mcb_server::args::{ValidateAction, ValidateArgs, ValidateScope};
use mcb_server::handlers::ValidateHandler;
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;
use tempfile::TempDir;

use crate::utils::domain_services::create_real_domain_services;

fn create_temp_file() -> Result<(TempDir, PathBuf), std::io::Error> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, "fn main() { println!(\"test\"); }")?;
    Ok((temp_dir, file_path))
}

fn create_temp_dir() -> Result<(TempDir, PathBuf), std::io::Error> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().to_path_buf();
    Ok((temp_dir, dir_path))
}

/// Macro to generate validation handler tests with common setup and assertions.
///
/// Usage patterns:
/// - `validate_test!(test_name, action, path_expr, expect_ok)`
/// - `validate_test!(test_name, action, path_expr, expect_error)`
/// - `validate_test!(test_name, action, path_expr, scope: Some(scope), expect_ok)`
/// - `validate_test!(test_name, action, path_expr, scope: Some(scope), rules: Some(vec![...]), expect_ok)`
macro_rules! validate_test {
    ($test_name:ident, $action:expr, expect_mcp_error) => {
        #[rstest]
        #[tokio::test]
        async fn $test_name() {
            let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
                return;
            };
            let handler = ValidateHandler::new(state.mcp_server.validation_service());

            let args = ValidateArgs {
                action: $action,
                path: None,
                scope: None,
                rules: None,
                category: None,
            };

            let result = handler.handle(Parameters(args)).await;
            let err = result.expect_err("validate handler should fail when path is missing");
            let err_str = err.to_string();
            assert!(
                err_str.contains("path") || err_str.contains("missing required field"),
                "error should mention missing path, got: {err_str}"
            );
        }
    };

    ($test_name:ident, $action:expr, $path_expr:expr, $(scope: $scope:expr,)? $(rules: $rules:expr,)? $(category: $category:expr,)? expect_ok) => {
        #[rstest]
        #[tokio::test]
        async fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            let (_temp_dir, path) = $path_expr?;
            let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
                return Ok(());
            };
            let handler = ValidateHandler::new(state.mcp_server.validation_service());

            let args = ValidateArgs {
                action: $action,
                path: Some(path.to_string_lossy().to_string()),
                scope: None $(.or($scope))?,
                rules: None $(.or($rules))?,
                category: None $(.or($category))?,
            };

            let result = handler.handle(Parameters(args)).await;
            let response = result.expect("validate handler should succeed for valid validation input");
            assert!(!response.content.is_empty(), "response should have content");
            assert!(!response.is_error.unwrap_or(false));
            Ok(())
        }
    };

    ($test_name:ident, $action:expr, path: $path:expr, $(scope: $scope:expr,)? expect_error) => {
        #[rstest]
        #[tokio::test]
        async fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
                return Ok(());
            };
            let handler = ValidateHandler::new(state.mcp_server.validation_service());

            let args = ValidateArgs {
                action: $action,
                path: Some($path.to_string()),
                scope: None $(.or($scope))?,
                rules: None,
                category: None,
            };

            let result = handler.handle(Parameters(args)).await;
            let response =
                result.expect("validate handler should return structured error response for invalid path");
            assert!(!response.content.is_empty(), "response should have content");
            assert!(response.is_error.unwrap_or(false), "Should return error");
            Ok(())
        }
    };

    ($test_name:ident, $action:expr, $path_expr:expr, expect_error) => {
        #[rstest]
        #[tokio::test]
        async fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            let (_temp_dir, path) = $path_expr?;
            let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
                return Ok(());
            };
            let handler = ValidateHandler::new(state.mcp_server.validation_service());

            let args = ValidateArgs {
                action: $action,
                path: Some(path.to_string_lossy().to_string()),
                scope: None,
                rules: None,
                category: None,
            };

            let result = handler.handle(Parameters(args)).await;
            let response = result.expect(
                "validate handler should return structured error response for invalid analyze input",
            );
            assert!(!response.content.is_empty(), "response should have content");
            assert!(response.is_error.unwrap_or(false), "Should return error");
            Ok(())
        }
    };
}

validate_test!(
    test_validate_run_with_valid_file,
    ValidateAction::Run,
    create_temp_file(),
    scope: Some(ValidateScope::File),
    expect_ok
);

validate_test!(
    test_validate_run_with_nonexistent_path,
    ValidateAction::Run,
    path: "/nonexistent/path/to/file.rs",
    scope: Some(ValidateScope::File),
    expect_error
);

validate_test!(
    test_validate_run_missing_path,
    ValidateAction::Run,
    expect_mcp_error
);

validate_test!(
    test_validate_run_with_project_scope,
    ValidateAction::Run,
    create_temp_dir(),
    scope: Some(ValidateScope::Project),
    expect_ok
);

validate_test!(
    test_validate_run_auto_detect_scope_file,
    ValidateAction::Run,
    create_temp_file(),
    expect_ok
);

validate_test!(
    test_validate_run_auto_detect_scope_project,
    ValidateAction::Run,
    create_temp_dir(),
    expect_ok
);

#[rstest]
#[tokio::test]
async fn test_validate_run_with_specific_rules() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, path) = create_temp_file()?;
    let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
        return Ok(());
    };
    let handler = ValidateHandler::new(state.mcp_server.validation_service());

    let args = ValidateArgs {
        action: ValidateAction::Run,
        path: Some(path.to_string_lossy().to_string()),
        scope: Some(ValidateScope::File),
        rules: Some(vec!["rule1".to_owned(), "rule2".to_owned()]),
        category: None,
    };

    let result = handler.handle(Parameters(args)).await;
    let response = result.expect("validate handler should return rule validation response");
    assert!(!response.content.is_empty(), "response should have content");
    // Non-existent rules produce 0 violations = success (no matching rules to enforce)
    assert!(
        !response.is_error.unwrap_or(false),
        "non-existent rules should not cause an error — they simply produce 0 violations"
    );
    Ok(())
}

#[rstest]
#[case(None)]
#[case(Some("style".to_owned()))]
#[tokio::test]
async fn test_validate_list_rules(
    #[case] category: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
        return Ok(());
    };
    let handler = ValidateHandler::new(state.mcp_server.validation_service());

    let args = ValidateArgs {
        action: ValidateAction::ListRules,
        path: None,
        scope: None,
        rules: None,
        category,
    };

    let result = handler.handle(Parameters(args)).await;

    let response = result.expect("validate handler should succeed for list rules action");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(!response.is_error.unwrap_or(false));
    Ok(())
}

validate_test!(
    test_validate_analyze_valid_file,
    ValidateAction::Analyze,
    create_temp_file(),
    expect_ok
);

validate_test!(
    test_validate_analyze_nonexistent_file,
    ValidateAction::Analyze,
    path: "/nonexistent/file.rs",
    expect_error
);

validate_test!(
    test_validate_analyze_directory_should_fail,
    ValidateAction::Analyze,
    create_temp_dir(),
    expect_error
);

validate_test!(
    test_validate_analyze_missing_path,
    ValidateAction::Analyze,
    expect_mcp_error
);
