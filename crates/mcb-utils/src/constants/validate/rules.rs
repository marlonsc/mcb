//! YAML rule field names, GRL keywords, engine types, and rule defaults.

use super::super::define_str_consts;
use super::super::values::TAG_QUALITY;

// ============================================================================
// Default Validation Settings
// ============================================================================

/// Default cyclomatic complexity threshold.
pub const DEFAULT_COMPLEXITY_THRESHOLD: u32 = 15;

/// Default TDG score threshold (0-100, higher is worse).
pub const DEFAULT_TDG_THRESHOLD: u32 = 50;

/// Default max lines per file before triggering a size violation.
pub const DEFAULT_MAX_FILE_LINES: usize = 500;

// ============================================================================
// YAML Rule Field Names (macro-generated)
// ============================================================================

define_str_consts! {
    /// YAML field: rule identifier.
    YAML_FIELD_ID = "id";
    /// YAML field: rule display name.
    YAML_FIELD_NAME = "name";
    /// YAML field: rule category.
    YAML_FIELD_CATEGORY = "category";
    /// YAML field: rule severity level.
    YAML_FIELD_SEVERITY = "severity";
    /// YAML field: rule enabled flag.
    YAML_FIELD_ENABLED = "enabled";
    /// YAML field: rule description text.
    YAML_FIELD_DESCRIPTION = "description";
    /// YAML field: rule rationale text.
    YAML_FIELD_RATIONALE = "rationale";
    /// YAML field: rule engine type.
    YAML_FIELD_ENGINE = "engine";
    /// YAML field: rule configuration block.
    YAML_FIELD_CONFIG = "config";
    /// YAML field: rule definition block.
    YAML_FIELD_RULE = "rule";
    /// YAML field: auto-fix suggestions.
    YAML_FIELD_FIXES = "fixes";
    /// YAML field: fix type.
    YAML_FIELD_FIX_TYPE = "type";
    /// YAML field: pattern match string.
    YAML_FIELD_PATTERN = "pattern";
    /// YAML field: violation message.
    YAML_FIELD_MESSAGE = "message";
    /// YAML field: lint select rules.
    YAML_FIELD_LINT_SELECT = "lint_select";
    /// YAML field: selectors block.
    YAML_FIELD_SELECTORS = "selectors";
    /// YAML field: language filter.
    YAML_FIELD_LANGUAGE = "language";
    /// YAML field: AST node type.
    YAML_FIELD_NODE_TYPE = "node_type";
    /// YAML field: AST query string.
    YAML_FIELD_AST_QUERY = "ast_query";
    /// YAML field: metrics thresholds.
    YAML_FIELD_METRICS = "metrics";
    /// YAML field: file filters.
    YAML_FIELD_FILTERS = "filters";
    /// YAML field: template base marker.
    YAML_FIELD_BASE = "_base";
    /// YAML field: template reference.
    YAML_FIELD_TEMPLATE = "_template";
    /// YAML field: rule extension marker.
    YAML_FIELD_EXTENDS = "_extends";
    /// YAML field: regex pattern.
    YAML_FIELD_REGEX = "regex";
    /// YAML field: patterns array.
    YAML_FIELD_PATTERNS = "patterns";
    /// YAML field: crate name.
    YAML_FIELD_CRATE_NAME = "crate_name";
    /// YAML field: allowed dependencies list.
    YAML_FIELD_ALLOWED_DEPS = "allowed_dependencies";
    /// YAML field: rule expression.
    YAML_FIELD_EXPRESSION = "expression";
    /// YAML field: rule condition.
    YAML_FIELD_CONDITION = "condition";
    /// YAML field: rule action.
    YAML_FIELD_ACTION = "action";
    /// YAML field: GRL rule definition.
    GRL = "grl";
    /// YAML field: rule definition block reference.
    YAML_FIELD_RULE_DEFINITION = "rule_definition";
}

define_str_consts! {
    // --- GRL Keywords ---
    /// GRL keyword: "when" condition block.
    GRL_KEYWORD_WHEN = "when";
    /// GRL keyword: "then" action block.
    GRL_KEYWORD_THEN = "then";
    // --- Metrics Specific Fields ---
    /// YAML field: cognitive complexity metric.
    YAML_FIELD_COGNITIVE_COMPLEXITY = "cognitive_complexity";
    /// YAML field: cyclomatic complexity metric.
    YAML_FIELD_CYCLOMATIC_COMPLEXITY = "cyclomatic_complexity";
    /// YAML field: function length metric.
    YAML_FIELD_FUNCTION_LENGTH = "function_length";
    /// YAML field: nesting depth metric.
    YAML_FIELD_NESTING_DEPTH = "nesting_depth";
    // --- Rusty Rules specific fields ---
    /// Rusty Rules field: "all" logical operation.
    RUSTY_FIELD_ALL = "all";
    /// Rusty Rules field: "any" logical operation.
    RUSTY_FIELD_ANY = "any";
    /// Rusty Rules field: "not" logical operation.
    RUSTY_FIELD_NOT = "not";
    /// Rusty Rules field: fact type identifier.
    RUSTY_FIELD_FACT_TYPE = "fact_type";
    /// Rusty Rules field: field name indicator.
    RUSTY_FIELD_FIELD = "field";
    /// Rusty Rules field: matching operator.
    RUSTY_FIELD_OPERATOR = "operator";
    /// Rusty Rules field: expected value.
    RUSTY_FIELD_VALUE = "value";
    /// Rusty Rules field: violation details.
    RUSTY_FIELD_VIOLATION = "violation";
    // --- Metrics Threshold Fields ---
    /// Metrics field: maximum threshold.
    METRICS_FIELD_MAX = "max";
    /// Metrics field: severity override.
    METRICS_FIELD_SEVERITY_OVERRIDE = "severity_override";
    // --- Rule Defaults ---
    /// Default rule name when not specified.
    DEFAULT_RULE_NAME = "Unnamed Rule";
    /// Default rule description.
    DEFAULT_RULE_DESCRIPTION = "No description provided";
    /// Default rule rationale.
    DEFAULT_RULE_RATIONALE = "No rationale provided";
    /// Default rule engine type.
    RUSTY_RULES = "rusty-rules";
    /// Default violation message for expression engine rules.
    DEFAULT_EXPR_RULE_ID = "EXPR_RULE";
    /// Default expression engine violation message.
    DEFAULT_EXPR_MESSAGE = "Expression rule violation";
    /// Default Rete engine violation message.
    DEFAULT_RETE_MESSAGE = "Rule violation detected";
    /// Default GRL rule ID.
    DEFAULT_GRL_RULE_ID = "GRL_RULE";
    /// Default violation message for rusty-rules engine.
    DEFAULT_VIOLATION_MESSAGE = "Rule violation";
    // --- Rule Engine Type Identifiers ---
    /// Rete network engine type.
    ENGINE_TYPE_RETE = "rete";
    /// Rust Rule Engine type.
    ENGINE_TYPE_RUST_RULE = "rust-rule-engine";
    /// `EvalExpr` engine type.
    ENGINE_TYPE_EVALEXPR = "evalexpr";
    /// JSON DSL engine type.
    ENGINE_TYPE_JSON_DSL = "json-dsl";
    // --- Rusty Rules Engine Defaults ---
    /// Default rule type when not specified.
    GENERIC = "generic";
    /// Default field name for condition checks.
    RUSTY_DEFAULT_FIELD = "value";
    /// Default operator for condition checks.
    RUSTY_DEFAULT_OPERATOR = "equals";
}

/// Default rule category.
pub const DEFAULT_RULE_CATEGORY: &str = TAG_QUALITY;

/// Cargo dependency condition: `not_exists`.
pub const NOT_EXISTS: &str = "not_exists";

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

/// Condition: exists.
pub const RUSTY_CONDITION_EXISTS: &str = "exists";

// --- Linter Command Names ---

/// Ruff linter command name.
pub const LINTER_CMD_RUFF: &str = "ruff";

/// Cargo command name (for Clippy).
pub const LINTER_CMD_CARGO: &str = "cargo";
