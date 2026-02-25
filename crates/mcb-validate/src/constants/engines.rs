//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Rule engine type identifiers.
//!
//! String constants for the pluggable rule engine system used in
//! YAML-based rule routing.

/// Rete network engine type.
pub const ENGINE_TYPE_RETE: &str = "rete";

/// Rust Rule Engine type.
pub const ENGINE_TYPE_RUST_RULE: &str = "rust-rule-engine";

/// GRL (Grule Rule Language) engine type.
pub const ENGINE_TYPE_GRL: &str = "grl";

/// Expression evaluator engine type.
pub const ENGINE_TYPE_EXPRESSION: &str = "expression";

/// `EvalExpr` engine type.
pub const ENGINE_TYPE_EVALEXPR: &str = "evalexpr";

/// Rusty Rules engine type.
pub const ENGINE_TYPE_RUSTY_RULES: &str = "rusty-rules";

/// JSON DSL engine type.
pub const ENGINE_TYPE_JSON_DSL: &str = "json-dsl";

// ============================================================================
// Rusty Rules Engine (engines/rusty_rules_engine.rs)
// ============================================================================

/// Default rule type when not specified.
pub const RUSTY_DEFAULT_RULE_TYPE: &str = "generic";

/// Default fact type for conditions.
pub const RUSTY_DEFAULT_FACT_TYPE: &str = "generic";

/// Default field name for condition checks.
pub const RUSTY_DEFAULT_FIELD: &str = "value";

/// Default operator for condition checks.
pub const RUSTY_DEFAULT_OPERATOR: &str = "equals";

/// Cargo dependency condition: `not_exists`.
pub const RUSTY_DEFAULT_CARGO_CONDITION: &str = "not_exists";

/// File size rule condition: `exceeds_limit`.
pub const RUSTY_DEFAULT_FILE_SIZE_CONDITION: &str = "exceeds_limit";

/// Default file extension pattern for `file_size` rules.
pub const RUSTY_DEFAULT_FILE_SIZE_PATTERN: &str = ".rs";

/// Default label for custom actions.
pub const RUSTY_CUSTOM_ACTION_DEFAULT: &str = "Custom action";

/// Violation ID for cargo dependency rules.
pub const RUSTY_CARGO_DEP_VIOLATION_ID: &str = "CARGO_DEP";

/// Message when required dependency is missing.
pub const RUSTY_CARGO_DEP_MISSING_MSG: &str = "Required dependency not found";

/// Message when forbidden dependency is present.
pub const RUSTY_CARGO_DEP_FORBIDDEN_MSG: &str = "Forbidden dependency found";

/// Violation ID for AST pattern rules.
pub const RUSTY_AST_PATTERN_VIOLATION_ID: &str = "AST_PATTERN";

/// Path fragment for target directory (skip in scans).
pub const RUSTY_TARGET_DIR_FRAGMENT: &str = "/target/";

/// Rule type: `cargo_dependencies`.
pub const RUSTY_RULE_TYPE_CARGO_DEPENDENCIES: &str = "cargo_dependencies";

/// Rule type: `file_size`.
pub const RUSTY_RULE_TYPE_FILE_SIZE: &str = "file_size";

/// Rule type: `ast_pattern`.
pub const RUSTY_RULE_TYPE_AST_PATTERN: &str = "ast_pattern";

/// Condition: `not_exists`.
pub const RUSTY_CONDITION_NOT_EXISTS: &str = "not_exists";

/// Condition: exists.
pub const RUSTY_CONDITION_EXISTS: &str = "exists";

// ============================================================================
// Linter Command Names
// ============================================================================

/// Ruff linter command name.
pub const LINTER_CMD_RUFF: &str = "ruff";

/// Cargo command name (for Clippy).
pub const LINTER_CMD_CARGO: &str = "cargo";
