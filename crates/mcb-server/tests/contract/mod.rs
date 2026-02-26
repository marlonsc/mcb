#![allow(missing_docs)]

extern crate mcb_infrastructure;
extern crate mcb_providers;

#[path = "../utils/mod.rs"]
#[allow(dead_code, unused_imports)]
mod utils;

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
