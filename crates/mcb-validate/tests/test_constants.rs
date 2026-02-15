//! Centralized constants for mcb-validate tests.
//!
//! All test crate names, fixture paths, file names, and commonly used
//! string literals live here. Tests MUST import from this module instead
//! of hardcoding values.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Test crate names
// ---------------------------------------------------------------------------

/// Generic test crate name for unit tests that don't need layer semantics.
pub const TEST_CRATE: &str = "my-test";

/// Domain layer test crate name.
pub const DOMAIN_CRATE: &str = "my-domain";

/// Server / handler layer test crate name.
pub const SERVER_CRATE: &str = "my-server";

// ---------------------------------------------------------------------------
// Project config
// ---------------------------------------------------------------------------

/// Generic project prefix used across test fixtures.
pub const PROJECT_PREFIX: &str = "my";

// ---------------------------------------------------------------------------
// File / path conventions
// ---------------------------------------------------------------------------

/// Standard lib.rs file name.
pub const LIB_RS: &str = "lib.rs";

// ---------------------------------------------------------------------------
// Crate-based fixture src paths (relative to crate src/)
// ---------------------------------------------------------------------------

/// Domain service path inside the fixture crate.
pub const FIXTURE_DOMAIN_SERVICE_PATH: &str = "domain/service.rs";

/// Server handler path inside the fixture crate.
pub const FIXTURE_SERVER_HANDLER_PATH: &str = "handlers/user_handler.rs";

/// Infrastructure layer test crate name (short form).
pub const INFRA_CRATE: &str = "my-infra";

// ---------------------------------------------------------------------------
// Workspace structure
// ---------------------------------------------------------------------------

/// Relative path prefix for crates inside a workspace.
pub const CRATES_DIR: &str = "crates";

/// Default workspace Cargo.toml content.
pub const WORKSPACE_CARGO_TOML: &str = r#"[workspace]
members = ["crates/*"]
"#;

/// Default package version for test crates.
pub const DEFAULT_VERSION: &str = "0.1.0";

// ---------------------------------------------------------------------------
// Patterns and prefixes
// ---------------------------------------------------------------------------

/// Forbidden dependency prefix pattern (glob style).
pub const FORBIDDEN_PREFIX_PATTERN: &str = "my-*";

/// Forbidden prefix used in rule configs (list format).
pub const FORBIDDEN_PREFIX: &str = "my-";

// ---------------------------------------------------------------------------
// Config files
// ---------------------------------------------------------------------------

/// Validation config file name.
pub const CONFIG_FILE_NAME: &str = "mcb-validate.toml";

/// Dummy workspace path for tests that don't need a real workspace.
pub const TEST_WORKSPACE_PATH: &str = "/test/workspace";

/// Test crate name used in cargo dependency tests as the subject crate.
pub const TEST_SUBJECT_CRATE: &str = "test-crate";

// ---------------------------------------------------------------------------
// Crate layer mappings for architecture tests
// ---------------------------------------------------------------------------

/// Architecture layer → (layer_key, crate_name, module_name) tuples.
/// Used by architecture_rules_tests and integration tests.
pub const CRATE_LAYER_MAPPINGS: &[(&str, &str, &str)] = &[
    ("domain", "my-domain", "my_domain"),
    ("application", "my-application", "my_application"),
    ("providers", "my-providers", "my_providers"),
    ("infrastructure", "my-infrastructure", "my_infrastructure"),
    ("server", "my-server", "my_server"),
    ("validate", "my-validate", "my_validate"),
];

// ---------------------------------------------------------------------------
// Validator thresholds (used to configure validators in tests)
// ---------------------------------------------------------------------------

/// Low file-size threshold for testing (triggers FileTooLarge on fixture files).
pub const FILE_SIZE_LOW_THRESHOLD: usize = 100;

// ---------------------------------------------------------------------------
// Rule IDs and engine names
// ---------------------------------------------------------------------------

/// Architecture rule for domain layer boundary.
pub const RULE_CA001: &str = "CA001";

/// Engine used by architecture rules.
pub const ENGINE_RUST_RULE: &str = "rust-rule-engine";

/// Expected keyword in CA001 rule name.
pub const RULE_CA001_NAME_KEYWORD: &str = "Domain";

// ---------------------------------------------------------------------------
// Linter constants
// ---------------------------------------------------------------------------

/// Ruff linter file extension.
pub const RUFF_EXTENSION: &str = "py";

/// Clippy linter file extension.
pub const CLIPPY_EXTENSION: &str = "rs";

/// Severity level: error.
pub const SEVERITY_ERROR: &str = "error";

/// Severity level: warning.
pub const SEVERITY_WARNING: &str = "warning";

/// Severity level: info.
pub const SEVERITY_INFO: &str = "info";

// ---------------------------------------------------------------------------
// Ruff rule codes for severity mapping tests
// ---------------------------------------------------------------------------

/// Ruff error-level rule code.
pub const RUFF_CODE_ERROR: &str = "F401";

/// Ruff warning-level rule code.
pub const RUFF_CODE_WARNING: &str = "W291";

/// Ruff info-level rule code.
pub const RUFF_CODE_INFO: &str = "I001";

/// Clippy note-level label (maps to "info").
pub const CLIPPY_LEVEL_NOTE: &str = "note";

// ---------------------------------------------------------------------------
// Engine routing constants (used by router_tests)
// ---------------------------------------------------------------------------

/// Engine name for RETE/GRL-based rules.
pub const ENGINE_NAME_RETE: &str = "rete";

/// Engine name for expression-based rules.
pub const ENGINE_NAME_EXPRESSION: &str = "expression";

/// Engine name for rust-rule-engine (full name used in YAML rules).
pub const ENGINE_NAME_RUST_RULE: &str = "rust-rule-engine";

// ---------------------------------------------------------------------------
// GRL rule templates (used by rete_engine_tests)
// ---------------------------------------------------------------------------

/// Simple GRL rule template — single condition, single action.
/// Use `format!()` to substitute `{name}`, `{condition}`, `{action}`.
pub const GRL_SIMPLE_RULE: &str = r#"
rule "{name}" salience 10 {{
    when
        {condition}
    then
        {action};
}}
"#;

/// Fact key: whether crate has internal (forbidden) dependencies.
pub const FACT_HAS_INTERNAL_DEPS: &str = "Facts.has_internal_dependencies";
/// Fact key: whether a violation was triggered by the rule.
pub const FACT_VIOLATION_TRIGGERED: &str = "Facts.violation_triggered";
/// Fact key: human-readable violation message.
pub const FACT_VIOLATION_MESSAGE: &str = "Facts.violation_message";
/// Fact key: rule ID that triggered the violation.
pub const FACT_VIOLATION_RULE_NAME: &str = "Facts.violation_rule_name";
/// Fact key: name of the crate being checked.
pub const FACT_CRATE_NAME: &str = "Facts.crate_name";
/// Fact key: generic result value set by test rules.
pub const FACT_RESULT_VALUE: &str = "Facts.result_value";

// ---------------------------------------------------------------------------
// AST root node kinds (used by ast_tests)
// ---------------------------------------------------------------------------

/// Root node kind for Rust source files.
pub const AST_ROOT_RUST: &str = "source_file";

/// Root node kind for Python modules.
pub const AST_ROOT_PYTHON: &str = "module";

/// Root node kind for JavaScript/TypeScript programs.
pub const AST_ROOT_PROGRAM: &str = "program";

// ---------------------------------------------------------------------------
// Unwrap detector constants (used by unwrap_detector_tests)
// ---------------------------------------------------------------------------

/// Method name detected for `.unwrap()` calls.
pub const UNWRAP_METHOD: &str = "unwrap";

/// Method name detected for `.expect()` calls.
pub const EXPECT_METHOD: &str = "expect";

// ---------------------------------------------------------------------------
// Inline code snippets for engine tests
// ---------------------------------------------------------------------------

/// Simple Rust main.rs content for RuleContext file_contents.
pub const SNIPPET_MAIN_RS: &str = r#"fn main() { println!("hello"); }"#;

/// Simple Rust lib.rs content for RuleContext file_contents.
pub const SNIPPET_LIB_RS: &str = "pub fn helper() -> Result<(), Error> { Ok(()) }";

// ---------------------------------------------------------------------------
// Cargo.toml template for dependency tests
// ---------------------------------------------------------------------------

/// Template for generating a Cargo.toml with forbidden dependencies.
/// Placeholders: `{crate_name}`, `{version}`, `{deps}`.
pub const CARGO_TOML_TEMPLATE: &str = r#"[package]
name = "{crate_name}"
version = "{version}"

[dependencies]
{deps}
"#;
