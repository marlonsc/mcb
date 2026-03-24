//! MCP tool command tests via rmcp client API (stdio transport).
//!
//! Each module tests one MCP tool through the real binary.
//! ALL tests MUST use `#[serial]` — only one mcb process at a time.
//!
//! Run with: `cargo test -p mcb --test integration mcp_commands`

pub mod common;

mod agent_tests;
mod entity_tests;
mod index_tests;
mod memory_tests;
mod project_tests;
mod search_tests;
mod session_tests;
mod validate_tests;
mod vcs_tests;
