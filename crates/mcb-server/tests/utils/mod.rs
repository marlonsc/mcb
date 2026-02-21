//! Test utilities for mcb-server
//!
//! ALL shared test helpers live here. No helpers outside this directory.

pub mod admin_harness;
pub mod axum_harness;
pub mod collection;
pub mod domain_services;
pub mod http_mcp;
pub mod invariants;
pub mod real_providers;
pub mod search_fixtures;
pub mod service_detection;
pub mod shared_context;
pub mod sync;
pub mod test_fixtures;
pub mod text;
pub mod timeouts;
