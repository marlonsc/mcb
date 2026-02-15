//! Unit test suite for mcb-validate
//!
//! Run with: `cargo test -p mcb-validate --test unit`

// Shared test utilities (single source — lives outside unit/ dir)
#[path = "test_utils.rs"]
mod test_utils;

// Centralized test constants (shared across unit and integration tests)
#[path = "test_constants.rs"]
mod test_constants;

#[path = "unit/architecture_rules_tests.rs"]
mod architecture_rules;

#[path = "unit/ast_tests.rs"]
mod ast;

#[path = "unit/ast_query_tests.rs"]
mod ast_query_tests;

#[path = "unit/async_patterns_tests.rs"]
mod async_patterns;

#[path = "unit/cargo_dependency_tests.rs"]
mod cargo_dependency;

#[path = "unit/dependency_tests.rs"]
mod dependency_tests;

#[path = "unit/documentation_tests.rs"]
mod documentation;

#[path = "unit/error_boundary_tests.rs"]
mod error_boundary;

#[path = "unit/expression_engine_tests.rs"]
mod expression_engine;

#[path = "unit/implementation_tests.rs"]
mod implementation;

#[path = "unit/kiss_tests.rs"]
mod kiss;

#[path = "unit/lib_tests.rs"]
mod lib_tests;

#[path = "unit/linters_tests.rs"]
mod linters;

#[path = "unit/organization_tests.rs"]
mod organization;

#[path = "unit/patterns_tests.rs"]
mod patterns;

#[path = "unit/performance_tests.rs"]
mod performance;

#[path = "unit/quality_tests.rs"]
mod quality;

#[path = "unit/refactoring_tests.rs"]
mod refactoring;

#[path = "unit/rete_engine_tests.rs"]
mod rete_engine;

#[path = "unit/solid_tests.rs"]
mod solid;

#[path = "unit/template_engine_tests.rs"]
mod template_engine;

#[path = "unit/hygiene_tests.rs"]
mod hygiene_tests;

#[path = "unit/unwrap_detector_tests.rs"]
mod unwrap_detector;

#[path = "unit/yaml_loader_tests.rs"]
mod yaml_loader;

#[path = "unit/yaml_validator_tests.rs"]
mod yaml_validator_tests;

#[path = "unit/rust_rule_engine_tests.rs"]
mod rust_rule_engine_tests;

#[path = "unit/validator_engine_tests.rs"]
mod validator_engine_tests;

#[path = "unit/router_tests.rs"]
mod router_tests;

#[path = "unit/file_matcher_tests.rs"]
mod file_matcher_tests;

#[path = "unit/language_detector_tests.rs"]
mod language_detector_tests;

#[path = "unit/rule_filters_tests.rs"]
mod rule_filters_tests;

#[path = "unit/discovery_tests.rs"]
mod discovery;

#[path = "unit/walkdir_guardrail_tests.rs"]
mod walkdir_guardrail_tests;

#[path = "unit/display_parity_tests.rs"]
mod display_parity_tests;
