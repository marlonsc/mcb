//! Integration test suite for mcb-validate
//!
//! Run with: `cargo test -p mcb-validate --test integration`

#[path = "ca001_integration_tests.rs"]
mod ca001;

#[path = "integration_architecture.rs"]
mod architecture;

#[path = "integration_ast.rs"]
mod ast;

#[path = "integration_duplication.rs"]
mod duplication;

#[path = "integration_engines.rs"]
mod engines;

#[path = "integration_full.rs"]
mod full;

#[path = "integration_linters.rs"]
mod linters;

#[path = "integration_metrics.rs"]
mod metrics;

#[path = "integration_rca_metrics.rs"]
mod rca_metrics;

#[path = "integration_tests.rs"]
mod integration_tests;

#[path = "integration_yaml_metrics.rs"]
mod yaml_metrics;

#[path = "ca009_tests.rs"]
mod ca009;
