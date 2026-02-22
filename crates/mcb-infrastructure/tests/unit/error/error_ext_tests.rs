//! Error Extension Tests

use std::io;

use mcb_domain::error::{Error, Result};
use mcb_infrastructure::error_ext::{ErrorContext, infra};
use rstest::rstest;

#[rstest]
fn test_error_context_extension() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");

    let result: Result<()> = Err(io_error).io_context("failed to read file");
    assert!(result.is_err());

    if let Err(Error::Io { source, message }) = result {
        assert!(message.contains("failed to read file"));
        assert!(source.is_some());
    } else {
        panic!("Expected Io error");
    }
}

#[rstest]
fn test_infra_error_creation() {
    let error = infra::infrastructure_error_msg("test error message");

    match &error {
        Error::Infrastructure { message, source } => {
            assert_eq!(message, "test error message");
            assert!(source.is_none());
        }
        Error::Vcs(_)
        | Error::Database(_)
        | Error::Validation(_)
        | Error::Embedding(_)
        | Error::VectorStore(_)
        | Error::Cache(_)
        | Error::Config(_)
        | Error::NotFound(_)
        | Error::Language(_)
        | Error::Other(_) => panic!("Expected Infrastructure error, got {error:?}"),
    }
}
