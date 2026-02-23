//! Unit tests — `cargo test -p mcb-server --test unit`

#[path = "../utils/mod.rs"]
#[allow(dead_code, unused_imports)]
mod utils;

pub mod builder;
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
