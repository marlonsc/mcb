//! Integration tests — `cargo test -p mcb-validate --test integration`

#[path = "../utils/mod.rs"]
pub mod utils;

mod ca001_integration_tests;
mod ca009_tests;
mod integration_architecture;
mod integration_ast;
mod integration_duplication;
mod integration_engines;
mod integration_full;
mod integration_linters;
mod integration_metrics;
mod integration_rca_metrics;
mod integration_tests;
mod integration_yaml_metrics;
mod test_individual;
