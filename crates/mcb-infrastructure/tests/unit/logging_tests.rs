//! Logging Tests

use mcb_infrastructure::constants::logging::{DEFAULT_LOG_LEVEL, LOG_MAX_FILES, LOG_ROTATION_SIZE};
use mcb_infrastructure::logging::{LoggingConfig, parse_log_level};
use rstest::*;
use tracing::Level;

#[rstest]
#[case("trace", Some(Level::TRACE))]
#[case("debug", Some(Level::DEBUG))]
#[case("info", Some(Level::INFO))]
#[case("warn", Some(Level::WARN))]
#[case("warning", Some(Level::WARN))]
#[case("error", Some(Level::ERROR))]
#[case("invalid", None)]
fn parse_log_level_values(#[case] input: &str, #[case] expected: Option<Level>) {
    match expected {
        Some(level) => assert_eq!(parse_log_level(input).unwrap(), level),
        None => assert!(parse_log_level(input).is_err()),
    }
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
