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

/// Application layer test crate name.
pub const APPLICATION_CRATE: &str = "my-application";

/// Infrastructure layer test crate name.
pub const INFRASTRUCTURE_CRATE: &str = "my-infrastructure";

/// Providers layer test crate name.
pub const PROVIDERS_CRATE: &str = "my-providers";

/// Validate crate name (self-referential).
pub const VALIDATE_CRATE: &str = "my-validate";

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

/// Standard error module file name (triggers exemptions in some validators).
pub const ERROR_RS: &str = "error.rs";

/// Constants module file name (triggers exemptions in some validators).
pub const CONSTANTS_RS: &str = "constants.rs";

/// Null provider file name (triggers exemptions in some validators).
pub const NULL_RS: &str = "null.rs";

/// Domain service file path — triggers domain layer detection in validators.
pub const DOMAIN_SERVICE_PATH: &str = "domain/services/agent.rs";

/// Domain error file path — triggers error.rs exemption in validators.
pub const DOMAIN_ERROR_PATH: &str = "domain/error.rs";

/// Handler file path — triggers handler/boundary detection in validators.
pub const HANDLER_PATH: &str = "handlers/agent_handler.rs";

/// Adapter file path — triggers adapter detection in validators.
pub const ADAPTER_PATH: &str = "adapters/agent_adapter.rs";

/// Service file path — triggers service boundary detection in validators.
pub const SERVICE_PATH: &str = "services/agent_service.rs";

// ---------------------------------------------------------------------------
// Fixture file names (in tests/fixtures/rust/)
// ---------------------------------------------------------------------------

/// Domain service with infrastructure error type violations.
pub const FIXTURE_DOMAIN_WRONG_ERROR: &str = "domain_wrong_error.rs";

/// Domain error.rs that should be exempt from layer checks.
pub const FIXTURE_DOMAIN_ERROR_EXEMPT: &str = "domain_error_exempt.rs";

/// Handler with missing error context and leaked error violations.
pub const FIXTURE_HANDLER_MISSING_CONTEXT: &str = "handler_missing_context.rs";

// ---------------------------------------------------------------------------
// Crate-based fixture src paths (relative to crate src/)
// ---------------------------------------------------------------------------

/// Domain service path inside the fixture crate.
pub const FIXTURE_DOMAIN_SERVICE_PATH: &str = "domain/service.rs";

/// Domain model path inside the fixture crate.
pub const FIXTURE_DOMAIN_MODEL_PATH: &str = "domain/model.rs";

/// Server handler path inside the fixture crate.
pub const FIXTURE_SERVER_HANDLER_PATH: &str = "handlers/user_handler.rs";

/// Infrastructure null.rs path.
pub const FIXTURE_INFRA_NULL_PATH: &str = "null.rs";

/// Infrastructure constants.rs path.
pub const FIXTURE_INFRA_CONSTANTS_PATH: &str = "constants.rs";

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
    (
        "language_support",
        "my-language-support",
        "my_language_support",
    ),
    ("ast_utils", "my-ast-utils", "my_ast_utils"),
];

// ---------------------------------------------------------------------------
// Validator thresholds (used to configure validators in tests)
// ---------------------------------------------------------------------------

/// Low file-size threshold for testing (triggers FileTooLarge on fixture files).
pub const FILE_SIZE_LOW_THRESHOLD: usize = 100;

/// Threshold for "too many struct fields" in KISS tests.
pub const MAX_STRUCT_FIELDS_THRESHOLD: usize = 7;

/// Number of lines to generate for "function too long" tests.
pub const LONG_FUNCTION_LINE_COUNT: usize = 60;

/// Threshold for "too many trait methods" in SOLID ISP tests.
pub const MAX_TRAIT_METHODS_THRESHOLD: usize = 6;

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
