//! Integration tests — `cargo test -p mcb-server --test integration`

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

/// Handler integration tests.
pub mod handlers;

mod error_recovery_integration;
mod error_shape_tests;
mod form_deserialization_test;
mod full_stack_integration;
mod golden_acceptance_integration;
mod hooks_integration;

mod admin_api_tests;
mod auto_context_tests;
