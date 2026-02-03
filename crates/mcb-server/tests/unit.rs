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

#[path = "unit/mcp_protocol_tests.rs"]
mod mcp_protocol_tests;

#[path = "unit/collection_mapping_tests.rs"]
mod collection_mapping_tests;

#[path = "unit/fixtures_smoke.rs"]
mod fixtures_smoke;
