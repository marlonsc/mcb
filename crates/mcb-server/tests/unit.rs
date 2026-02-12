//! Unit test suite for mcb-server
//!
//! Run with: `cargo test -p mcb-server --test unit`

// Shared test utilities (single declaration for all unit tests)
#[path = "test_utils/mod.rs"]
mod test_utils;

#[path = "unit/args_tests.rs"]
mod args_tests;

#[path = "unit/builder_tests.rs"]
mod builder_tests;

#[path = "unit/formatter_tests.rs"]
mod formatter_tests;

#[path = "unit/mcp_error_handling_tests.rs"]
mod mcp_error_handling_tests;

#[path = "unit/browse_handlers_tests.rs"]
mod browse_handlers_tests;

#[path = "unit/highlight_service_tests.rs"]
mod highlight_service_tests;

#[path = "unit/mcp_protocol_tests.rs"]
mod mcp_protocol_tests;

#[path = "unit/fixtures_smoke.rs"]
mod fixtures_smoke;

#[path = "unit/processor_tests.rs"]
mod processor_tests;

#[path = "unit/config_tests.rs"]
mod config_tests;

#[path = "unit/http_client_tests.rs"]
mod http_client_tests;

#[path = "unit/stdio_tests.rs"]
mod stdio_tests;

#[path = "unit/types_tests.rs"]
mod types_tests;

#[path = "unit/http_tests.rs"]
mod http_tests;

#[path = "unit/constants_tests.rs"]
mod constants_tests;

#[path = "unit/json_tests.rs"]
mod json_tests;

#[path = "unit/init_tests.rs"]
mod init_tests;
