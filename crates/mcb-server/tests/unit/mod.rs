//! Unit tests — `cargo test -p mcb-server --test unit`

// Force-link provider crates so linkme-registered entries populate the
// distributed slices. The explicit `use` prevents linker gc-sections from
// stripping the modules that contain the registrations.
extern crate mcb_infrastructure;
extern crate mcb_providers;
extern crate mcb_validate;
#[allow(unused_imports)]
use mcb_infrastructure::events::BroadcastEventBus;
#[allow(unused_imports)]
use mcb_providers::database::seaorm::migration::Migrator;

/// Shared test utilities.
#[path = "../utils/mod.rs"]
pub mod utils;

/// Auth unit tests.
pub mod auth_tests;
/// `McbState` unit tests.
pub mod state_tests;

pub mod constants;
pub mod context_resolution;
pub mod error_mapping;
pub mod fixtures;
pub mod formatter;
/// Handler unit tests.
pub mod handlers;
pub mod hooks;
/// Macro unit tests.
pub mod macros;
pub mod mcp_server;
/// Tests for rmcp transport-streamable-http-server feature availability
pub mod rmcp_http_feature;
pub mod services;
/// Tool unit tests.
pub mod tools;
/// Transport unit tests.
pub mod transport;
pub mod util_tests;
