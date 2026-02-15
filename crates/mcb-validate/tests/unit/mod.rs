//! Unit test suite for mcb-validate
//!
//! Run with: `cargo test -p mcb-validate --test unit`

// Shared test utilities (single source — lives outside unit/ dir)
#[path = "../test_utils.rs"]
mod test_utils;

// Centralized test constants (shared across unit and integration tests)
#[path = "../test_constants.rs"]
mod test_constants;

#[path = "architecture_rules_tests.rs"]
mod architecture_rules;

#[path = "ast_tests.rs"]
mod ast;

#[path = "ast_query_tests.rs"]
mod ast_query_tests;

#[path = "async_patterns_tests.rs"]
mod async_patterns;

#[path = "cargo_dependency_tests.rs"]
mod cargo_dependency;

#[path = "dependency_tests.rs"]
mod dependency_tests;

#[path = "documentation_tests.rs"]
mod documentation;

#[path = "error_boundary_tests.rs"]
mod error_boundary;

#[path = "expression_engine_tests.rs"]
mod expression_engine;

#[path = "implementation_tests.rs"]
mod implementation;

#[path = "kiss_tests.rs"]
mod kiss;

#[path = "lib_tests.rs"]
mod lib_tests;

#[path = "linters_tests.rs"]
mod linters;

#[path = "organization_tests.rs"]
mod organization;

#[path = "patterns_tests.rs"]
mod patterns;

#[path = "performance_tests.rs"]
mod performance;

#[path = "quality_tests.rs"]
mod quality;

#[path = "refactoring_tests.rs"]
mod refactoring;

#[path = "rete_engine_tests.rs"]
mod rete_engine;

#[path = "solid_tests.rs"]
mod solid;

#[path = "template_engine_tests.rs"]
mod template_engine;

#[path = "hygiene_tests.rs"]
mod hygiene_tests;

#[path = "unwrap_detector_tests.rs"]
mod unwrap_detector;

#[path = "yaml_loader_tests.rs"]
mod yaml_loader;

#[path = "yaml_validator_tests.rs"]
mod yaml_validator_tests;

#[path = "rust_rule_engine_tests.rs"]
mod rust_rule_engine_tests;

#[path = "validator_engine_tests.rs"]
mod validator_engine_tests;

#[path = "router_tests.rs"]
mod router_tests;

#[path = "file_matcher_tests.rs"]
mod file_matcher_tests;

#[path = "language_detector_tests.rs"]
mod language_detector_tests;

#[path = "rule_filters_tests.rs"]
mod rule_filters_tests;

#[path = "discovery_tests.rs"]
mod discovery;

#[path = "walkdir_guardrail_tests.rs"]
mod walkdir_guardrail_tests;

#[path = "display_parity_tests.rs"]
mod display_parity_tests;

#[path = "field_selection_tests.rs"]
mod field_selection_tests;

#[path = "declarative_validator_tests.rs"]
mod declarative_validator_tests;
