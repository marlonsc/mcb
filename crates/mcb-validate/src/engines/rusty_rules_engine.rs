//! Rusty Rules Engine Wrapper
//!
//! Wrapper for rusty-rules crate with JSON DSL and composition support.

use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;

use super::hybrid_engine::{RuleContext, RuleEngine};
use crate::Result;
use crate::ValidationConfig;
use crate::engines::hybrid_engine::RuleViolation;
use crate::run_context::ValidationRunContext;
use crate::violation_trait::{Severity, ViolationCategory};

/// Wrapper for rusty-rules engine
pub struct RustyRulesEngineWrapper {
    // In a real implementation, this would hold the actual rusty-rules instance
    rule_definitions: HashMap<String, RustyRule>,
}

/// Rusty rule definition with composition support
#[derive(Debug, Clone)]
pub struct RustyRule {
    /// The type of rule (e.g., "cargo_dependencies", "ast_pattern").
    pub rule_type: String,
    /// The condition logic to evaluate.
    pub condition: Condition,
    /// The action to take if the condition is met.
    pub action: Action,
}

/// Conditions with composition (all/any/not)
#[derive(Debug, Clone)]
pub enum Condition {
    /// All conditions must be true
    All(Vec<Condition>),
    /// Any condition must be true
    Any(Vec<Condition>),
    /// Negate condition
    Not(Box<Condition>),
    /// Simple condition
    Simple {
        /// The type of fact being checked.
        fact_type: String,
        /// The field of the fact to check.
        field: String,
        /// The operator to use for comparison.
        operator: String,
        /// The value to compare against.
        value: Value,
    },
}

/// Actions to execute when condition matches
#[derive(Debug, Clone)]
pub enum Action {
    /// Report a standard violation.
    Violation {
        /// The violation message.
        message: String,
        /// The severity of the violation.
        severity: Severity,
    },
    /// Execute a custom action string.
    Custom(String),
}

impl Default for RustyRulesEngineWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl RustyRulesEngineWrapper {
    /// Creates a new, empty RustyRulesEngineWrapper.
    pub fn new() -> Self {
        Self {
            rule_definitions: HashMap::new(),
        }
    }

    /// Parse rule definition from JSON
    pub fn parse_rule_definition(&mut self, rule_id: String, definition: &Value) -> Result<()> {
        let rule = self.parse_rule_from_json(definition)?;
        self.rule_definitions.insert(rule_id, rule);
        Ok(())
    }

    fn parse_rule_from_json(&self, definition: &Value) -> Result<RustyRule> {
        // Parse rule type
        let rule_type = definition
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("generic")
            .to_string();

        // Parse condition
        let condition = if let Some(condition_json) = definition.get("condition") {
            Self::parse_condition_value(condition_json)?
        } else {
            Condition::All(vec![]) // Default empty condition
        };

        // Parse action
        let action = if let Some(action_json) = definition.get("action") {
            self.parse_action(action_json)
        } else {
            Action::Violation {
                message: "Rule violation".to_string(),
                severity: Severity::Warning,
            }
        };

        Ok(RustyRule {
            rule_type,
            condition,
            action,
        })
    }

    fn parse_condition_value(condition_json: &Value) -> Result<Condition> {
        if let Some(all_conditions) = condition_json.get("all")
            && let Some(conditions_array) = all_conditions.as_array()
        {
            let conditions = conditions_array
                .iter()
                .map(Self::parse_condition_value)
                .collect::<Result<Vec<_>>>()?;
            return Ok(Condition::All(conditions));
        }

        if let Some(any_conditions) = condition_json.get("any")
            && let Some(conditions_array) = any_conditions.as_array()
        {
            let conditions = conditions_array
                .iter()
                .map(Self::parse_condition_value)
                .collect::<Result<Vec<_>>>()?;
            return Ok(Condition::Any(conditions));
        }

        if let Some(not_condition) = condition_json.get("not") {
            let condition = Self::parse_condition_value(not_condition)?;
            return Ok(Condition::Not(Box::new(condition)));
        }

        // Simple condition
        let fact_type = condition_json
            .get("fact_type")
            .and_then(|v| v.as_str())
            .unwrap_or("generic")
            .to_string();

        let field = condition_json
            .get("field")
            .and_then(|v| v.as_str())
            .unwrap_or("value")
            .to_string();

        let operator = condition_json
            .get("operator")
            .and_then(|v| v.as_str())
            .unwrap_or("equals")
            .to_string();

        let value = condition_json.get("value").cloned().unwrap_or(Value::Null);

        Ok(Condition::Simple {
            fact_type,
            field,
            operator,
            value,
        })
    }

    fn parse_action(&self, action_json: &Value) -> Action {
        if let Some(violation) = action_json.get("violation") {
            let message = violation
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Rule violation")
                .to_string();

            let severity =
                violation
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .map_or(Severity::Warning, |s| match s {
                        "error" => Severity::Error,
                        "info" => Severity::Info,
                        _ => Severity::Warning,
                    });

            return Action::Violation { message, severity };
        }

        Action::Custom("Custom action".to_string())
    }

    fn has_forbidden_dependency(&self, pattern: &str, context: &RuleContext) -> bool {
        // Check Cargo.toml files for forbidden dependencies
        use glob::Pattern;

        let cargo_pattern = Pattern::new("**/Cargo.toml").unwrap();
        let trimmed_pattern = pattern.trim_matches('"');
        let pattern_prefix = trimmed_pattern.trim_end_matches('*');

        if let Ok(run_context) =
            ValidationRunContext::active_or_build(&ValidationConfig::new(&context.workspace_root))
        {
            for entry in run_context.file_inventory() {
                let path = &entry.absolute_path;
                if !cargo_pattern.matches_path(path) {
                    continue;
                }

                if let Ok(content) = run_context.read_cached(path)
                    && dependency_matches(content.as_ref(), pattern_prefix)
                {
                    return true;
                }
            }

            return false;
        }

        for (path, content) in &context.file_contents {
            if cargo_pattern.matches_path(std::path::Path::new(path.as_str()))
                && dependency_matches(content, pattern_prefix)
            {
                return true;
            }
        }

        false
    }
}

fn dependency_matches(content: &str, pattern_prefix: &str) -> bool {
    // Try to parse as TOML and check dependencies section
    if let Ok(toml_value) = content.parse::<toml::Value>() {
        if let Some(dependencies) = toml_value.get("dependencies")
            && let Some(deps_table) = dependencies.as_table()
        {
            for dep_name in deps_table.keys() {
                if dep_name.starts_with(pattern_prefix) {
                    return true;
                }
            }
        }
    } else {
        // Fallback to simple pattern matching
        for line in content.lines() {
            let line = line.trim();
            if line.contains('=') {
                let dep_name = line.split('=').next().unwrap_or_default().trim();
                if dep_name.starts_with(pattern_prefix) {
                    return true;
                }
            }
        }
    }

    false
}

#[async_trait]
impl RuleEngine for RustyRulesEngineWrapper {
    async fn execute(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // In a real implementation, this would use the rusty-rules engine
        // For now, we'll simulate the behavior

        let _rule_id = "unknown"; // Would be passed in real implementation

        if let Some(rule_type) = rule_definition.get("type").and_then(|v| v.as_str()) {
            match rule_type {
                "cargo_dependencies" => {
                    self.execute_cargo_dependency_rule(rule_definition, context)
                        .await
                }
                "file_size" => self.execute_file_size_rule(rule_definition, context).await,
                "ast_pattern" => {
                    self.execute_ast_pattern_rule(rule_definition, context)
                        .await
                }
                _ => Ok(vec![]),
            }
        } else {
            Ok(vec![])
        }
    }
}

impl RustyRulesEngineWrapper {
    async fn execute_cargo_dependency_rule(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        // Get the condition (default to "not_exists" for backwards compatibility)
        let condition = rule_definition
            .get("condition")
            .and_then(|v| v.as_str())
            .unwrap_or("not_exists");

        if let Some(forbidden_pattern) = rule_definition.get("pattern").and_then(|v| v.as_str()) {
            let has_forbidden = self.has_forbidden_dependency(forbidden_pattern, context);

            match condition {
                "not_exists" => {
                    // Create violation if forbidden dependency EXISTS (should NOT exist)
                    if has_forbidden {
                        violations.push(
                            RuleViolation::new(
                                "CARGO_DEP",
                                ViolationCategory::Architecture,
                                Severity::Error,
                                "Forbidden dependency found",
                            )
                            .with_context(format!("Pattern: {forbidden_pattern}")),
                        );
                    }
                }
                "exists" => {
                    // Create violation if forbidden dependency does NOT exist (should exist)
                    if !has_forbidden {
                        violations.push(
                            RuleViolation::new(
                                "CARGO_DEP",
                                ViolationCategory::Architecture,
                                Severity::Error,
                                "Required dependency not found",
                            )
                            .with_context(format!("Pattern: {forbidden_pattern}")),
                        );
                    }
                }
                _ => {
                    // Unknown condition, do nothing
                }
            }
        }

        Ok(violations)
    }

    async fn execute_ast_pattern_rule(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        if let Some(forbidden) = rule_definition.get("forbidden").and_then(|v| v.as_array()) {
            for pattern_value in forbidden {
                if let Some(pattern) = pattern_value.as_str() {
                    // Simplified check - in real implementation would use AST analysis
                    for (file_path, content) in &context.file_contents {
                        if content.contains(pattern) {
                            violations.push(
                                RuleViolation::new(
                                    "AST_PATTERN",
                                    ViolationCategory::Quality,
                                    Severity::Error,
                                    format!("Found forbidden pattern: {pattern}"),
                                )
                                .with_file(std::path::PathBuf::from(file_path))
                                .with_context(format!("Pattern: {pattern}")),
                            );
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    async fn execute_file_size_rule(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        // Get the condition (default to "exceeds_limit")
        let condition = rule_definition
            .get("condition")
            .and_then(|v| v.as_str())
            .unwrap_or("exceeds_limit");

        // Get the pattern (file extension)
        let pattern = rule_definition
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or(".rs");

        // Get the message
        let message = rule_definition
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("File exceeds size limit");

        if condition == "exceeds_limit" {
            // Check files that match the pattern
            let max_lines = 500; // Hardcoded for now, could be configurable

            for (file_path, content) in &context.file_contents {
                if file_path.ends_with(pattern) {
                    let line_count = content.lines().count();

                    // Check exclusions
                    let path_str = file_path.clone();
                    let should_exclude = path_str.contains("/tests/")
                        || path_str.contains("/target/")
                        || path_str.ends_with("_test.rs");

                    if line_count > max_lines && !should_exclude {
                        violations.push(
                            RuleViolation::new(
                                "QUAL006",
                                ViolationCategory::Quality,
                                Severity::Warning,
                                format!("{message}: {line_count} lines (max: {max_lines})"),
                            )
                            .with_file(std::path::PathBuf::from(file_path))
                            .with_context(format!("File: {file_path}, Lines: {line_count}")),
                        );
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl Clone for RustyRulesEngineWrapper {
    fn clone(&self) -> Self {
        Self {
            rule_definitions: self.rule_definitions.clone(),
        }
    }
}
