//! Logging Tests

use mcb_domain::error::Error;
use mcb_infrastructure::constants::{DEFAULT_LOG_LEVEL, LOG_MAX_FILES, LOG_ROTATION_SIZE};
use mcb_infrastructure::logging::{init_logging, log_operation, parse_log_level, LoggingConfig};
use std::sync::Once;
use tracing::Level;

static INIT: Once = Once::new();

fn init_test_logging() {
    INIT.call_once(|| {
        let _ = init_logging(LoggingConfig::default());
    });
}

#[test]
fn test_parse_log_level() {
    assert_eq!(parse_log_level("trace").unwrap(), Level::TRACE);
    assert_eq!(parse_log_level("debug").unwrap(), Level::DEBUG);
    assert_eq!(parse_log_level("info").unwrap(), Level::INFO);
    assert_eq!(parse_log_level("warn").unwrap(), Level::WARN);
    assert_eq!(parse_log_level("warning").unwrap(), Level::WARN);
    assert_eq!(parse_log_level("error").unwrap(), Level::ERROR);

    assert!(parse_log_level("invalid").is_err());
}

#[test]
fn test_log_operation_success() {
    init_test_logging();

    let result = log_operation("test_operation", || Ok::<_, Error>(42));
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_log_operation_failure() {
    init_test_logging();

    let result: mcb_domain::Result<()> = log_operation("test_operation", || {
        Err(Error::Infrastructure {
            message: "test error".to_string(),
            source: None,
        })
    });
    assert!(result.is_err());
}

#[test]
fn test_logging_config_default() {
    let config = LoggingConfig::default();
    assert_eq!(config.level, DEFAULT_LOG_LEVEL);
    assert!(!config.json_format);
    assert!(config.file_output.is_none());
    assert_eq!(config.max_file_size, LOG_ROTATION_SIZE);
    assert_eq!(config.max_files, LOG_MAX_FILES);
}
