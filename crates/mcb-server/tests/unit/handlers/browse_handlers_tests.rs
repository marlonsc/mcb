//! Unit tests for browse handlers.

use mcb_server::admin::browse_handlers::BrowseErrorResponse;
use rstest::rstest;

#[rstest]
#[case(true, "Collection", "Collection not found", "NOT_FOUND")]
#[case(
    false,
    "Something went wrong",
    "Something went wrong",
    "INTERNAL_ERROR"
)]
#[test]
fn test_browse_error_response_factories(
    #[case] use_not_found: bool,
    #[case] input: &str,
    #[case] expected_error: &str,
    #[case] expected_code: &str,
) {
    let err = if use_not_found {
        BrowseErrorResponse::not_found(input)
    } else {
        BrowseErrorResponse::internal(input)
    };
    assert_eq!(err.error, expected_error);
    assert_eq!(err.code, expected_code);
}
