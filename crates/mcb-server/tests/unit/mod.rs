//! Unit test suite for mcb-server
//!
//! Run with: `cargo test -p mcb-server --test unit`

// Shared test utilities (single declaration for all unit tests)
#[path = "../test_utils/mod.rs"]
mod test_utils;

// Search-specific fixtures (only used by unit tests, not integration)
#[path = "../test_utils/search_fixtures.rs"]
mod search_fixtures;

mod args_tests;

mod builder_tests;

mod formatter_tests;

mod fairing_tests;

mod mcp_error_handling_tests;

mod browse_handlers_tests;

mod highlight_service_tests;

mod mcp_protocol_tests;

mod mcp_contract_tests;

mod fixtures_smoke;

mod processor_tests;

mod config_tests;

mod http_client_tests;

mod stdio_tests;

mod types_tests;

mod http_tests;

mod constants_tests;

mod json_tests;

mod init_tests;

mod project_handler_tests;

mod issue_entity_handler_tests;

mod org_entity_handler_tests;

mod plan_entity_handler_tests;

mod vcs_entity_handler_tests;

mod collections_utils_tests;

mod context_resolution_tests;

mod unified_execution_gate_tests;
