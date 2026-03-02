//! Unit tests — `cargo test -p mcb-server --test unit`

pub mod auth_tests;
pub mod state_tests;

pub mod constants;
pub mod context_resolution;
pub mod error_mapping;
pub mod fixtures;
pub mod formatter;
pub mod handlers;
pub mod hooks;
pub mod macros;
pub mod mcp_server;
/// Tests for rmcp transport-streamable-http-server feature availability
pub mod rmcp_http_feature;
pub mod services;
pub mod tools;
pub mod transport;
pub mod util_tests;
