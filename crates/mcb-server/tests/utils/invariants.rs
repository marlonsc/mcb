//! MCP tool result assertions — re-exports from centralized `mcb_domain::test_mcp_assertions`.

pub use mcb_domain::test_mcp_assertions::{
    assert_error_shape, assert_invalid_params, assert_tool_error, error_text, extract_text,
    is_error,
};
