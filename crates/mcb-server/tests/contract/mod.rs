//! Contract tests — `cargo test -p mcb-server --test contract`

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

mod agent_contract_tests;
mod common;
mod entity_contract_tests;
mod index_contract_tests;
mod memory_contract_tests;
mod project_contract_tests;
mod search_contract_tests;
mod session_contract_tests;
mod validate_contract_tests;
mod vcs_contract_tests;
