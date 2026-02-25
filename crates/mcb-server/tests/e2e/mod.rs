//! E2E golden tests — `cargo test -p mcb-server --test e2e`
//!
//! Allow test-idiomatic use of `expect/panic/to_string` in e2e helpers.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::str_to_string,
    clippy::map_unwrap_or,
    clippy::uninlined_format_args
)]

#[path = "../utils/mod.rs"]
#[allow(dead_code, unused_imports)]
mod utils;

mod gap_fixes;
mod golden_e2e_complete;
mod golden_highlight_service;
mod golden_memory_project;
mod golden_tools;
mod test_api_key_lifecycle;
mod test_issue_entity_crud;
mod test_org_data_isolation;
mod test_org_entity_crud;
mod test_plan_entity_crud;
mod test_project_operations;
mod test_session_lifecycle;
mod test_validate_operations;
mod test_vcs_entity_crud;
mod validation_fixes;
