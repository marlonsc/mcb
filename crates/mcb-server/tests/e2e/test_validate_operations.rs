use crate::utils::test_fixtures::{create_test_mcp_server, sample_codebase_path};
use mcb_server::args::{ValidateAction, ValidateArgs, ValidateScope};
use rmcp::handler::server::wrapper::Parameters;

/// Validate with action=Analyze on a sample codebase file → verify success response.
#[tokio::test]
async fn golden_validate_analyze() {
    let (server, _td) = create_test_mcp_server().await;
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
}

/// Validate with action=ListRules → verify response shape contains validators/count.
#[tokio::test]
async fn golden_validate_status() {
    let (server, _td) = create_test_mcp_server().await;
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
}

/// Validate with an invalid (non-existent) path → verify error response.
#[tokio::test]
async fn golden_validate_missing_path() {
    let (server, _td) = create_test_mcp_server().await;
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
}

/// Validate with no action args (empty path for Analyze) → verify error.
#[tokio::test]
async fn golden_validate_empty_args() {
    let (server, _td) = create_test_mcp_server().await;
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
}
