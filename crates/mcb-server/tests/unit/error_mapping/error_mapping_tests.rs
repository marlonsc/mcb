//! Error mapping: domain errors → MCP-safe responses.
//!
//! Tests verify two security invariants:
//! 1. Client-fixable errors (NotFound, InvalidArgument) surface to the agent
//! 2. Internal errors NEVER leak implementation details (SQL, paths, credentials)

use mcb_domain::error::Error;
use mcb_domain::utils::text::extract_text_from;
use mcb_server::error_mapping::{
    safe_internal_error, to_contextual_tool_error, to_opaque_mcp_error,
};
use rstest::rstest;

// ─── Client-fixable errors are surfaced ──────────────────────────────

#[rstest]
#[case(Error::NotFound { resource: "session-123".to_owned() }, "Not found: session-123")]
#[case(Error::InvalidArgument { message: "query empty".to_owned() }, "Invalid argument: query empty")]
fn client_errors_include_actionable_detail(#[case] err: Error, #[case] expected: &str) {
    let mcp = to_opaque_mcp_error(&err);
    assert_eq!(mcp.message, expected);
}

#[rstest]
#[case(Error::NotFound { resource: "item".to_owned() }, "Not found: item")]
#[case(Error::Database { message: "timeout".to_owned(), source: None }, "Database error: timeout")]
fn contextual_tool_error_is_marked_as_error(#[case] err: Error, #[case] expected: &str) {
    let result = to_contextual_tool_error(err);
    assert!(result.is_error.unwrap_or(false));
    assert_eq!(extract_text_from(&result.content), expected);
}

// ─── Internal details never leak to MCP clients ──────────────────────

#[rstest]
#[case(Error::Internal { message: "SELECT * FROM secrets".to_owned() })]
#[case(Error::Infrastructure { message: "pool exhausted at 0x7fff".to_owned(), source: None })]
fn internal_errors_return_generic_message(#[case] err: Error) {
    let mcp = to_opaque_mcp_error(&err);
    assert_eq!(mcp.message, "internal server error");
}

#[rstest]
#[case("db connection refused")]
#[case("/home/user/.config/mcb/credentials.json not found")]
#[case("secret key mismatch at row 42")]
fn safe_internal_error_never_leaks_sensitive_detail(#[case] detail: &str) {
    let mcp = safe_internal_error("operation", &detail);
    assert_eq!(mcp.message, "internal server error");
    assert!(!mcp.message.contains(detail));
}
