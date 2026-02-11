//! Unit tests for browse handlers.

use mcb_server::admin::browse_handlers::BrowseErrorResponse;

#[test]
fn test_browse_error_response_not_found() {
    let err = BrowseErrorResponse::not_found("Collection");
    assert_eq!(err.error, "Collection not found");
    assert_eq!(err.code, "NOT_FOUND");
}

#[test]
fn test_browse_error_response_internal() {
    let err = BrowseErrorResponse::internal("Something went wrong");
    assert_eq!(err.error, "Something went wrong");
    assert_eq!(err.code, "INTERNAL_ERROR");
}
