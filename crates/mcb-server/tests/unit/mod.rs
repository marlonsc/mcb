//! Unit tests — `cargo test -p mcb-server --test unit`

// linkme force-link only — DO NOT use for type/function imports (CA019 enforced)
extern crate mcb_infrastructure;
extern crate mcb_providers;
extern crate mcb_validate;
use mcb_infrastructure::infrastructure::events::BroadcastEventBus as _; // linkme force-link
use mcb_providers::database::seaorm::migration::Migrator as _; // linkme force-link

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
