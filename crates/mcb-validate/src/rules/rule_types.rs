//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Rule data types produced by the YAML rule loader.
//!
//! Pure value shapes consumed across the engines; kept separate from the
//! loading logic in [`super::yaml_loader`] so each module stays under the
//! file-size cap.

use serde::{Deserialize, Serialize};

use crate::filters::rule_filters::RuleFilters;

/// Loaded and validated YAML rule
#[derive(Debug, Clone)]
pub struct ValidatedRule {
    /// Unique identifier for the rule.
    pub id: String,
    /// Human-readable name of the rule.
    pub name: String,
    /// Category of the rule (e.g., quality, security).
    pub category: String,
    /// Severity level (error, warning, info).
    pub severity: String,
    /// Whether the rule is active.
    pub enabled: bool,
    /// Detailed description of what the rule checks.
    pub description: String,
    /// Explanation of why this rule exists.
    pub rationale: String,
    /// The engine used to execute this rule.
    pub engine: String,
    /// Engine-specific configuration.
    pub config: serde_json::Value,
    /// Raw rule definition.
    pub rule_definition: serde_json::Value,
    /// List of available automated fixes.
    pub fixes: Vec<RuleFix>,
    /// Linter codes to execute (e.g., `["F401"]` for Ruff, `["clippy::unwrap_used"]` for Clippy)
    pub lint_select: Vec<String>,
    /// Custom message for violations
    pub message: Option<String>,
    /// AST selectors for multi-language pattern matching (Phase 2)
    pub selectors: Vec<AstSelector>,
    /// Tree-sitter query string for complex AST matching (Phase 2)
    pub ast_query: Option<String>,
    /// Metrics configuration for schema v3 rules (Phase 4)
    pub metrics: Option<MetricsConfig>,
    /// Optional filters to restrict rule applicability by language, dependency, or file pattern.
    pub filters: Option<RuleFilters>,
}

/// Metrics configuration for rule/v3 rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Cognitive complexity threshold
    pub cognitive_complexity: Option<MetricThresholdConfig>,
    /// Cyclomatic complexity threshold
    pub cyclomatic_complexity: Option<MetricThresholdConfig>,
    /// Function length threshold
    pub function_length: Option<MetricThresholdConfig>,
    /// Nesting depth threshold
    pub nesting_depth: Option<MetricThresholdConfig>,
}

/// Configuration for a single metric threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThresholdConfig {
    /// Maximum allowed value
    pub max: u32,
    /// Severity level when threshold is exceeded
    pub severity: Option<String>,
    /// Languages this threshold applies to
    pub languages: Option<Vec<String>>,
}

/// AST selector for language-specific pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSelector {
    /// Programming language (e.g., "rust", "python")
    pub language: String,
    /// AST node type to match (e.g., "`call_expression`", "`function_definition`")
    pub node_type: String,
    /// Tree-sitter query pattern (optional, for complex matching)
    pub pattern: Option<String>,
}

/// Suggested fix for a rule violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFix {
    /// Type of fix (e.g., replacement, suppression).
    pub fix_type: String,
    /// Pattern to replace (if applicable).
    pub pattern: Option<String>,
    /// Message describing the fix.
    pub message: String,
}
