//! Contract tests — `cargo test -p mcb-server --test contract`

// linkme force-link only — DO NOT use for type/function imports (CA019 enforced)
extern crate mcb_infrastructure;
extern crate mcb_providers;
extern crate mcb_validate;
#[allow(unused_imports)]
use mcb_infrastructure::infrastructure::events::BroadcastEventBus; // linkme force-link
#[allow(unused_imports)]
use mcb_providers::database::seaorm::migration::Migrator; // linkme force-link

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
