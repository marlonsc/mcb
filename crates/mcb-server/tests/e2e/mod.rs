//! E2E golden tests — `cargo test -p mcb-server --test e2e`

#[path = "../utils/mod.rs"]
#[allow(dead_code, unused_imports)]
mod utils;

mod browse_e2e;
mod gap_fixes;
mod golden_admin_web;
mod golden_e2e_complete;
mod golden_highlight_service;
mod golden_memory_project;
mod golden_tools;
mod validation_fixes;
