//! Rusty Rules Engine Wrapper
//!
//! Wrapper for rusty-rules crate with JSON DSL and composition support.

use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;

use super::hybrid_engine::{RuleContext, RuleEngine};
use crate::Result;
use crate::ValidationConfig;
use crate::constants::common::{TEST_DIR_FRAGMENT, TEST_FILE_SUFFIX};
use crate::constants::rules::{
    DEFAULT_VIOLATION_MESSAGE, YAML_FIELD_ACTION, YAML_FIELD_CONDITION, YAML_FIELD_FIX_TYPE,
    YAML_FIELD_MESSAGE, YAML_FIELD_PATTERN, YAML_FIELD_SEVERITY,
};
use crate::constants::severities::{SEVERITY_ERROR, SEVERITY_INFO};
use crate::engines::hybrid_engine::RuleViolation;
use crate::run_context::ValidationRunContext;
use crate::traits::violation::{Severity, ViolationCategory};

/// Wrapper for rusty-rules engine
pub struct RustyRulesEngineWrapper {
    // In a real implementation, this would hold the actual rusty-rules instance
    rule_definitions: HashMap<String, RustyRule>,
}

/// Rusty rule definition with composition support
#[derive(Debug, Clone)]
pub struct RustyRule {
    /// The type of rule (e.g., "`cargo_dependencies`", "`ast_pattern`").
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

const DEFAULT_RULE_TYPE: &str = "generic";
const DEFAULT_FACT_TYPE: &str = "generic";
const DEFAULT_FIELD: &str = "value";
const DEFAULT_OPERATOR: &str = "equals";
const DEFAULT_CARGO_CONDITION: &str = "not_exists";
const DEFAULT_FILE_SIZE_CONDITION: &str = "exceeds_limit";
const DEFAULT_FILE_SIZE_PATTERN: &str = ".rs";
const CUSTOM_ACTION_DEFAULT: &str = "Custom action";
const CARGO_DEP_VIOLATION_ID: &str = "CARGO_DEP";
const CARGO_DEP_MISSING_MSG: &str = "Required dependency not found";
const CARGO_DEP_FORBIDDEN_MSG: &str = "Forbidden dependency found";
const AST_PATTERN_VIOLATION_ID: &str = "AST_PATTERN";
const TARGET_DIR_FRAGMENT: &str = "/target/";
const RULE_TYPE_CARGO_DEPENDENCIES: &str = "cargo_dependencies";
const RULE_TYPE_FILE_SIZE: &str = "file_size";
const RULE_TYPE_AST_PATTERN: &str = "ast_pattern";
const CONDITION_NOT_EXISTS: &str = "not_exists";
const CONDITION_EXISTS: &str = "exists";

impl Default for RustyRulesEngineWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl RustyRulesEngineWrapper {
    /// Creates a new, empty `RustyRulesEngineWrapper`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rule_definitions: HashMap::new(),
        }
    }

    /// Parse rule definition from JSON
    ///
    /// # Errors
    ///
    /// Returns an error if the rule definition cannot be parsed.
    pub fn parse_rule_definition(&mut self, rule_id: String, definition: &Value) -> Result<()> {
        let rule = Self::parse_rule_from_json(definition)?;
        self.rule_definitions.insert(rule_id, rule);
        Ok(())
    }

    fn parse_rule_from_json(definition: &Value) -> Result<RustyRule> {
        let rule_type = definition
            .get(YAML_FIELD_FIX_TYPE)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_RULE_TYPE)
            .to_owned();
        let condition = Self::parse_optional_condition(definition)?;
        let action = Self::parse_optional_action(definition);

        Ok(RustyRule {
            rule_type,
            condition,
            action,
        })
    }

    fn parse_optional_condition(definition: &Value) -> Result<Condition> {
        definition
            .get(YAML_FIELD_CONDITION)
            .map(Self::parse_condition_value)
            .transpose()
            .map(|condition| condition.unwrap_or_else(|| Condition::All(vec![])))
    }

    fn parse_optional_action(definition: &Value) -> Action {
        definition.get(YAML_FIELD_ACTION).map_or(
            Action::Violation {
                message: DEFAULT_VIOLATION_MESSAGE.to_owned(),
                severity: Severity::Warning,
            },
            Self::parse_action,
        )
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
            .unwrap_or(DEFAULT_FACT_TYPE)
            .to_owned();

        let field = condition_json
            .get("field")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_FIELD)
            .to_owned();

        let operator = condition_json
            .get("operator")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_OPERATOR)
            .to_owned();

        let value = condition_json.get("value").cloned().unwrap_or(Value::Null);

        Ok(Condition::Simple {
            fact_type,
            field,
            operator,
            value,
        })
    }

    fn parse_action(action_json: &Value) -> Action {
        if let Some(violation) = action_json.get("violation") {
            let message = violation
                .get(YAML_FIELD_MESSAGE)
                .and_then(|v| v.as_str())
                .unwrap_or(DEFAULT_VIOLATION_MESSAGE)
                .to_owned();

            let severity = violation
                .get(YAML_FIELD_SEVERITY)
                .and_then(|v| v.as_str())
                .map_or(Severity::Warning, Self::parse_severity);

            return Action::Violation { message, severity };
        }

        Action::Custom(CUSTOM_ACTION_DEFAULT.to_owned())
    }

    fn parse_severity(raw: &str) -> Severity {
        match raw {
            SEVERITY_ERROR => Severity::Error,
            SEVERITY_INFO => Severity::Info,
            _ => Severity::Warning,
        }
    }

    fn has_forbidden_dependency(pattern: &str, context: &RuleContext) -> bool {
        // Check Cargo.toml files for forbidden dependencies
        use glob::Pattern;

        let Ok(cargo_pattern) = Pattern::new("**/Cargo.toml") else {
            return false;
        };
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
    content
        .parse::<toml::Value>()
        .ok()
        .is_some_and(|toml_value| toml_dependencies_match(&toml_value, pattern_prefix))
}

fn toml_dependencies_match(toml_value: &toml::Value, pattern_prefix: &str) -> bool {
    let Some(dependencies) = toml_value.get("dependencies") else {
        return false;
    };
    let Some(deps_table) = dependencies.as_table() else {
        return false;
    };

    deps_table
        .keys()
        .any(|dep_name| dep_name.starts_with(pattern_prefix))
}

#[async_trait]
impl RuleEngine for RustyRulesEngineWrapper {
    async fn execute(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let Some(rule_type) = rule_definition
            .get(YAML_FIELD_FIX_TYPE)
            .and_then(|v| v.as_str())
        else {
            return Ok(vec![]);
        };

        self.dispatch_rule(rule_type, rule_definition, context)
            .await
    }
}

impl RustyRulesEngineWrapper {
    async fn dispatch_rule(
        &self,
        rule_type: &str,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        match rule_type {
            RULE_TYPE_CARGO_DEPENDENCIES => {
                self.execute_cargo_dependency_rule(rule_definition, context)
                    .await
            }
            RULE_TYPE_FILE_SIZE => self.execute_file_size_rule(rule_definition, context).await,
            RULE_TYPE_AST_PATTERN => {
                self.execute_ast_pattern_rule(rule_definition, context)
                    .await
            }
            _ => Ok(vec![]),
        }
    }

    async fn execute_cargo_dependency_rule(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();
        let condition = rule_definition
            .get(YAML_FIELD_CONDITION)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_CARGO_CONDITION);
        let Some(forbidden_pattern) = rule_definition
            .get(YAML_FIELD_PATTERN)
            .and_then(|v| v.as_str())
        else {
            return Ok(violations);
        };

        let has_forbidden = Self::has_forbidden_dependency(forbidden_pattern, context);
        let should_report = match condition {
            CONDITION_NOT_EXISTS => has_forbidden,
            CONDITION_EXISTS => !has_forbidden,
            _ => false,
        };
        if !should_report {
            return Ok(violations);
        }

        let message = if condition == CONDITION_EXISTS {
            CARGO_DEP_MISSING_MSG
        } else {
            CARGO_DEP_FORBIDDEN_MSG
        };
        violations.push(
            RuleViolation::new(
                CARGO_DEP_VIOLATION_ID,
                ViolationCategory::Architecture,
                Severity::Error,
                message,
            )
            .with_context(format!("Pattern: {forbidden_pattern}")),
        );

        Ok(violations)
    }

    async fn execute_ast_pattern_rule(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();
        for pattern in forbidden_patterns(rule_definition) {
            violations.extend(ast_pattern_violations(context, pattern));
        }

        Ok(violations)
    }

    async fn execute_file_size_rule(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();
        let condition = rule_definition
            .get(YAML_FIELD_CONDITION)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_FILE_SIZE_CONDITION);
        let pattern = rule_definition
            .get(YAML_FIELD_PATTERN)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_FILE_SIZE_PATTERN);
        let message = rule_definition
            .get(YAML_FIELD_MESSAGE)
            .and_then(|v| v.as_str())
            .unwrap_or("File exceeds size limit");

        if condition != DEFAULT_FILE_SIZE_CONDITION {
            return Ok(violations);
        }

        let max_lines = crate::constants::DEFAULT_MAX_FILE_LINES;
        for (file_path, content) in &context.file_contents {
            if !file_path.ends_with(pattern) || Self::is_size_check_excluded(file_path) {
                continue;
            }

            let line_count = content.lines().count();
            if line_count <= max_lines {
                continue;
            }

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

        Ok(violations)
    }

    fn is_size_check_excluded(file_path: &str) -> bool {
        file_path.contains(TEST_DIR_FRAGMENT)
            || file_path.contains(TARGET_DIR_FRAGMENT)
            || file_path.ends_with(TEST_FILE_SUFFIX)
    }
}

fn forbidden_patterns(rule_definition: &Value) -> Vec<&str> {
    rule_definition
        .get("forbidden")
        .and_then(|v| v.as_array())
        .map(|forbidden| {
            forbidden
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect()
        })
        .unwrap_or_default()
}

fn ast_pattern_violations(context: &RuleContext, pattern: &str) -> Vec<RuleViolation> {
    context
        .file_contents
        .iter()
        .filter(|(_, content)| content.contains(pattern))
        .map(|(file_path, _)| {
            RuleViolation::new(
                AST_PATTERN_VIOLATION_ID,
                ViolationCategory::Quality,
                Severity::Error,
                format!("Found forbidden pattern: {pattern}"),
            )
            .with_file(std::path::PathBuf::from(file_path))
            .with_context(format!("Pattern: {pattern}"))
        })
        .collect()
}

impl Clone for RustyRulesEngineWrapper {
    fn clone(&self) -> Self {
        Self {
            rule_definitions: self.rule_definitions.clone(),
        }
    }
}
