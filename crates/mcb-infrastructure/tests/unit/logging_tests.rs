//! Logging Tests
#![allow(unsafe_code)]

use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::logging::parse_log_level;
use rstest::rstest;
use serial_test::serial;
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

#[rstest]
#[serial]
fn test_logging_config_default() {
    // SAFETY: Tests run serially via #[serial], so no concurrent env access.
    unsafe { std::env::remove_var("MCP__AUTH__ENABLED") };
    // SAFETY: Tests run serially via #[serial], so no concurrent env access.
    unsafe { std::env::remove_var("MCP__AUTH__JWT__SECRET") };
    // SAFETY: Tests run serially via #[serial], so no concurrent env access.
    unsafe { std::env::remove_var("MCP__PROVIDERS__EMBEDDING__PROVIDER") };
    let config = ConfigLoader::new().load().expect("load config").logging;
    assert_eq!(config.level, "info");
    assert!(!config.json_format);
    assert!(config.file_output.is_none());
    assert_eq!(config.max_file_size, 10_485_760); // 10 MB per default.toml
    assert_eq!(config.max_files, 5);
}
