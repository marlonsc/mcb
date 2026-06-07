//! Unit tests for domain error types

use mcb_domain::Error;
use rstest::rstest;

#[rstest]
#[case::not_found(Error::not_found("user"), "NotFound", "user")]
#[case::invalid_argument(Error::invalid_argument("bad input"), "InvalidArgument", "bad input")]
#[case::embedding(Error::embedding("no model"), "Embedding", "no model")]
#[case::vector_db(Error::vector_db("conn failed"), "VectorDb", "conn failed")]
#[case::io(Error::io("file missing"), "Io", "file missing")]
#[case::config(Error::config("missing key"), "Config", "missing key")]
#[case::internal(Error::internal("server error"), "Internal", "server error")]
#[case::cache(Error::cache("cache miss"), "Cache", "cache miss")]
#[case::network(Error::network("timeout"), "Network", "timeout")]
#[case::database(Error::database("sql error"), "Database", "sql error")]
#[case::authentication(Error::authentication("bad token"), "Authentication", "bad token")]
#[case::infrastructure(
    Error::infrastructure("service down"),
    "Infrastructure",
    "service down"
)]
#[case::configuration(Error::configuration("bad config"), "Configuration", "bad config")]
fn test_error_variants(
    #[case] error: Error,
    #[case] expected_variant: &str,
    #[case] expected_message: &str,
) {
    // Check variant via Debug
    let debug_str = format!("{error:?}");
    assert!(
        debug_str.contains(expected_variant),
        "Expected variant {expected_variant} in {debug_str:?}"
    );

    // Check message via Display or Debug (depending on how thiserror implements it)
    let display_str = format!("{error}");
    assert!(
        display_str.contains(expected_message) || debug_str.contains(expected_message),
        "Expected message '{expected_message}' in error"
    );
}

#[rstest]
#[case("Something went wrong")]
fn error_generic(#[case] message: &str) {
    let error = Error::generic(message);
    let display_str = format!("{error}");
    assert!(display_str.contains(message));
}

#[rstest]
#[test]
fn test_error_equality_discrimination() {
    let not_found = Error::not_found("resource");
    let invalid_arg = Error::invalid_argument("bad argument");

    // Verify they are different variants via pattern matching
    assert!(matches!(not_found, Error::NotFound { .. }));
    assert!(matches!(invalid_arg, Error::InvalidArgument { .. }));
    assert!(!matches!(not_found, Error::InvalidArgument { .. }));
}
