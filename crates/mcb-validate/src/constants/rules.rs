//! YAML rule field names and default values.
//!
//! Constants for the YAML-based rule definition format used by
//! the rule loader, template engine, and execution engines.

// ============================================================================
// YAML Rule Field Names
// ============================================================================

/// YAML field: rule identifier.
pub const YAML_FIELD_ID: &str = "id";

/// YAML field: rule display name.
pub const YAML_FIELD_NAME: &str = "name";

/// YAML field: rule category.
pub const YAML_FIELD_CATEGORY: &str = "category";

/// YAML field: rule severity level.
pub const YAML_FIELD_SEVERITY: &str = "severity";

/// YAML field: rule enabled flag.
pub const YAML_FIELD_ENABLED: &str = "enabled";

/// YAML field: rule description text.
pub const YAML_FIELD_DESCRIPTION: &str = "description";

/// YAML field: rule rationale text.
pub const YAML_FIELD_RATIONALE: &str = "rationale";

/// YAML field: rule engine type.
pub const YAML_FIELD_ENGINE: &str = "engine";

/// YAML field: rule configuration block.
pub const YAML_FIELD_CONFIG: &str = "config";

/// YAML field: rule definition block.
pub const YAML_FIELD_RULE: &str = "rule";

/// YAML field: auto-fix suggestions.
pub const YAML_FIELD_FIXES: &str = "fixes";

/// YAML field: fix type.
pub const YAML_FIELD_FIX_TYPE: &str = "type";

/// YAML field: pattern match string.
pub const YAML_FIELD_PATTERN: &str = "pattern";

/// YAML field: violation message.
pub const YAML_FIELD_MESSAGE: &str = "message";

/// YAML field: lint select rules.
pub const YAML_FIELD_LINT_SELECT: &str = "lint_select";

/// YAML field: selectors block.
pub const YAML_FIELD_SELECTORS: &str = "selectors";

/// YAML field: language filter.
pub const YAML_FIELD_LANGUAGE: &str = "language";

/// YAML field: AST node type.
pub const YAML_FIELD_NODE_TYPE: &str = "node_type";

/// YAML field: AST query string.
pub const YAML_FIELD_AST_QUERY: &str = "ast_query";

/// YAML field: metrics thresholds.
pub const YAML_FIELD_METRICS: &str = "metrics";

/// YAML field: file filters.
pub const YAML_FIELD_FILTERS: &str = "filters";

/// YAML field: template base marker.
pub const YAML_FIELD_BASE: &str = "_base";

/// YAML field: template reference.
pub const YAML_FIELD_TEMPLATE: &str = "_template";

/// YAML field: rule extension marker.
pub const YAML_FIELD_EXTENDS: &str = "_extends";

/// YAML field: regex pattern.
pub const YAML_FIELD_REGEX: &str = "regex";

/// YAML field: patterns array.
pub const YAML_FIELD_PATTERNS: &str = "patterns";

/// YAML field: crate name.
pub const YAML_FIELD_CRATE_NAME: &str = "crate_name";

/// YAML field: allowed dependencies list.
pub const YAML_FIELD_ALLOWED_DEPS: &str = "allowed_dependencies";

/// YAML field: rule expression (for expression engine).
pub const YAML_FIELD_EXPRESSION: &str = "expression";

/// YAML field: rule condition (for condition-action engines).
pub const YAML_FIELD_CONDITION: &str = "condition";

/// YAML field: rule action (for condition-action engines).
pub const YAML_FIELD_ACTION: &str = "action";

/// YAML field: GRL rule definition.
pub const YAML_FIELD_GRL: &str = "grl";

/// YAML field: rule definition block reference.
pub const YAML_FIELD_RULE_DEFINITION: &str = "rule_definition";

// ============================================================================
// Metrics threshold field names
// ============================================================================

/// Metrics field: maximum threshold.
pub const METRICS_FIELD_MAX: &str = "max";

/// Metrics field: severity override.
pub const METRICS_FIELD_SEVERITY: &str = "severity";

// ============================================================================
// YAML Rule Default Values
// ============================================================================

/// Default rule name when not specified.
pub const DEFAULT_RULE_NAME: &str = "Unnamed Rule";

/// Default rule category.
pub const DEFAULT_RULE_CATEGORY: &str = "quality";

/// Default rule severity.
pub const DEFAULT_RULE_SEVERITY: &str = "warning";

/// Default rule description.
pub const DEFAULT_RULE_DESCRIPTION: &str = "No description provided";

/// Default rule rationale.
pub const DEFAULT_RULE_RATIONALE: &str = "No rationale provided";

/// Default rule engine type.
pub const DEFAULT_RULE_ENGINE: &str = "rusty-rules";

/// Default violation message for expression engine rules.
pub const DEFAULT_EXPR_RULE_ID: &str = "EXPR_RULE";

/// Default expression engine violation message.
pub const DEFAULT_EXPR_MESSAGE: &str = "Expression rule violation";

/// Default Rete engine violation message.
pub const DEFAULT_RETE_MESSAGE: &str = "Rule violation detected";

/// Default GRL rule ID.
pub const DEFAULT_GRL_RULE_ID: &str = "GRL_RULE";

/// Default violation message for rusty-rules engine.
pub const DEFAULT_VIOLATION_MESSAGE: &str = "Rule violation";
