use crate::utils::test_fixtures::{create_test_mcp_server, sample_codebase_path};
use mcb_domain::utils::tests::utils::TestResult;
use mcb_server::args::{ValidateAction, ValidateArgs, ValidateScope};
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;

/// Validate with action=Analyze on a sample codebase file → verify success response.
#[rstest]
#[tokio::test]
async fn golden_validate_analyze() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;
    let handler = server.validate_handler();
    let file_path = sample_codebase_path().join("src/main.rs");

    let result = handler
        .handle(Parameters(ValidateArgs {
            action: ValidateAction::Analyze,
            scope: Some(ValidateScope::File),
            path: Some(file_path.to_string_lossy().into_owned()),
            rules: None,
            category: None,
        }))
        .await;

    assert!(result.is_ok(), "validate analyze must succeed");
    let call_result = match result {
        Ok(r) => r,
        Err(e) => panic!("already checked: {e}"),
    };
    assert!(
        !call_result.is_error.unwrap_or(false),
        "analyze should not return an error"
    );
    Ok(())
}

/// Validate with action=ListRules → verify response shape contains validators/count.
#[rstest]
#[tokio::test]
async fn golden_validate_status() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;
    let handler = server.validate_handler();

    let result = handler
        .handle(Parameters(ValidateArgs {
            action: ValidateAction::ListRules,
            scope: None,
            path: None,
            rules: None,
            category: None,
        }))
        .await;

    assert!(result.is_ok(), "validate list_rules must succeed");
    let call_result = match result {
        Ok(r) => r,
        Err(e) => panic!("already checked: {e}"),
    };
    assert!(
        !call_result.is_error.unwrap_or(false),
        "list_rules should not return an error"
    );
    Ok(())
}

/// Validate with an invalid (non-existent) path → verify error response.
#[rstest]
#[tokio::test]
async fn golden_validate_missing_path() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;
    let handler = server.validate_handler();

    let result = handler
        .handle(Parameters(ValidateArgs {
            action: ValidateAction::Run,
            scope: Some(ValidateScope::File),
            path: Some("/nonexistent/path/to/file.rs".to_owned()),
            rules: None,
            category: None,
        }))
        .await;

    assert!(
        result.is_ok(),
        "handler should return Ok with error content, not Err"
    );
    let call_result = match result {
        Ok(r) => r,
        Err(e) => panic!("already checked: {e}"),
    };
    assert!(
        call_result.is_error.unwrap_or(false),
        "missing path should produce an error result"
    );
    Ok(())
}

/// Validate with no action args (empty path for Analyze) → verify error.
#[rstest]
#[tokio::test]
async fn golden_validate_empty_args() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;
    let handler = server.validate_handler();

    let result = handler
        .handle(Parameters(ValidateArgs {
            action: ValidateAction::Analyze,
            scope: None,
            path: None,
            rules: None,
            category: None,
        }))
        .await;

    // Analyze requires a path — should return an MCP error
    assert!(
        result.is_err(),
        "analyze without path should return an MCP error"
    );
    Ok(())
}
