//! Unit test suite for mcb-server
//!
//! Run with: `cargo test -p mcb-server --test unit`

// Shared test utilities
#[path = "../utils/mod.rs"]
mod utils;

// Search-specific fixtures
#[path = "../utils/search_fixtures.rs"]
mod search_fixtures;

mod shared_context;

pub mod builder;

pub mod constants;
pub mod context_resolution;
pub mod error_mapping;
pub mod fixtures;
pub mod formatter;
pub mod handlers;
pub mod hooks;
pub mod init;
pub mod macros;
pub mod mcp_server;
pub mod services;
pub mod templates;
pub mod tools;
pub mod transport;
#[path = "utils/mod.rs"]
pub mod util_tests;
