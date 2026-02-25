//! Error Extension Tests

use std::io;

use mcb_domain::error::{Error, Result};
use mcb_infrastructure::error_ext::{ErrorContext, infra};
use rstest::rstest;

#[rstest]
#[allow(clippy::wildcard_enum_match_arm)]
fn test_error_context_extension() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");

    let result: Result<()> = Err(io_error).io_context("failed to read file");
    let err = result.expect_err("io_context should produce an error");
    match err {
        Error::Io { source, message } => {
            assert!(message.contains("failed to read file"));
            assert!(source.is_some());
        }
        other => panic!("Expected Io error, got: {other:?}"),
    }
}

#[rstest]
#[allow(clippy::wildcard_enum_match_arm)]
fn test_infra_error_creation() {
    let error = infra::infrastructure_error_msg("test error message");

    match error {
        Error::Infrastructure { message, source } => {
            assert_eq!(message, "test error message");
            assert!(source.is_none());
        }
        _ => panic!("Expected Infrastructure error"),
    }
}
