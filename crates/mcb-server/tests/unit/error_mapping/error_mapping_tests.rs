use mcb_domain::error::Error;
use mcb_server::error_mapping::{
    safe_internal_error, to_contextual_tool_error, to_opaque_mcp_error,
};
use rstest::rstest;

use crate::utils::text::extract_text;

#[rstest]
#[case(Error::NotFound { resource: "test".to_owned() }, "Not found: test")]
#[case(Error::Internal { message: "secret".to_owned() }, "internal server error")]
fn test_to_opaque_mcp_error(#[case] err: Error, #[case] expected_message: &str) {
    let mcp_err = to_opaque_mcp_error(&err);
    assert_eq!(mcp_err.message, expected_message);
}

#[rstest]
#[case(Error::NotFound { resource: "item".to_owned() }, "Not found: item")]
#[case(
    Error::Database {
        message: "db fail".to_owned(),
        source: None,
    },
    "Database error: db fail"
)]
fn test_to_contextual_tool_error(#[case] err: Error, #[case] expected: &str) {
    let result = to_contextual_tool_error(err);
    assert!(result.is_error.unwrap_or(false));
    let text = extract_text(&result.content);
    assert_eq!(text, expected);
}

#[rstest]
#[case("db connection refused", "internal server error")]
#[case("secret key mismatch at row 42", "internal server error")]
#[case(
    "/home/user/.config/mcb/credentials.json not found",
    "internal server error"
)]
fn safe_internal_error_never_leaks_underlying_message(
    #[case] sensitive_detail: &str,
    #[case] expected: &str,
) {
    let mcp_err = safe_internal_error("test operation", &sensitive_detail);
    assert_eq!(mcp_err.message, expected);
    assert!(
        !mcp_err.message.contains(sensitive_detail),
        "internal error must not contain sensitive detail '{sensitive_detail}'"
    );
}

#[rstest]
#[case(
    Error::Internal { message: "secret SQL query".to_owned() },
    "internal server error"
)]
#[case(
    Error::Infrastructure { message: "connection pool exhausted".to_owned(), source: None },
    "internal server error"
)]
fn opaque_error_never_leaks_internal_details(#[case] err: Error, #[case] expected: &str) {
    let mcp_err = to_opaque_mcp_error(&err);
    assert_eq!(mcp_err.message, expected);
    assert!(!mcp_err.message.contains("SQL"));
    assert!(!mcp_err.message.contains("pool"));
}
