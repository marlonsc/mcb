//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Expression Engine Wrapper
//!
//! Wrapper for evalexpr crate providing simple boolean expression evaluation.
//! Use this engine for rules that don't require complex GRL syntax (when/then).

use std::collections::HashMap;

use async_trait::async_trait;
use evalexpr::{ContextWithMutableVariables, HashMapContext, Value as EvalValue};
use serde_json::Value;

use crate::Result;
use crate::constants::common::{
    ASYNC_FN_PREFIX, EXPECT_CALL, TEST_DIR_FRAGMENT, TEST_FILE_SUFFIX, UNWRAP_CALL,
};
use crate::constants::rules::{
    DEFAULT_EXPR_MESSAGE, DEFAULT_EXPR_RULE_ID, YAML_FIELD_CATEGORY, YAML_FIELD_EXPRESSION,
    YAML_FIELD_ID, YAML_FIELD_MESSAGE, YAML_FIELD_SEVERITY,
};
use crate::constants::severities::{SEVERITY_ERROR, SEVERITY_WARNING};
use crate::engines::hybrid_engine::{RuleContext, RuleEngine, RuleViolation};
use mcb_domain::ports::validation::{Severity, ViolationCategory};

/// Wrapper for evalexpr engine
///
/// Evaluates simple boolean expressions like:
/// - `file_count > 500`
/// - `dependency_exists("serde")`
/// - `not contains_pattern(".unwrap()")`
pub struct ExpressionEngine {
    /// Cached contexts for repeated evaluations
    cached_contexts: HashMap<String, HashMapContext>,
}

/// `ExpressionRuleInput` struct.
pub struct ExpressionRuleInput<'a> {
    /// Rule ID
    pub rule_id: &'a str,
    /// Expression to evaluate
    pub expression: &'a str,
    /// Rule context containing file contents
    pub context: &'a RuleContext,
    /// Violation message
    pub message: &'a str,
    /// Violation severity
    pub severity: Severity,
    /// Violation category
    pub category: ViolationCategory,
}

impl Default for ExpressionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionEngine {
    /// Create a new expression engine instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cached_contexts: HashMap::new(),
        }
    }

    /// Build context from rule context for expression evaluation
    fn build_eval_context(rule_context: &RuleContext) -> HashMapContext {
        let mut ctx = HashMapContext::new();

        let _ = ctx.set_value(
            "file_count".to_owned(),
            EvalValue::Int(rule_context.file_contents.len() as i64),
        );

        let _ = ctx.set_value(
            "workspace_path_len".to_owned(),
            EvalValue::Int(
                rule_context
                    .workspace_root
                    .to_str()
                    .map_or(0_i64, |s| s.len() as i64),
            ),
        );

        // Check for common patterns in files
        let has_unwrap = rule_context
            .file_contents
            .values()
            .any(|content| content.contains(UNWRAP_CALL));
        let _ = ctx.set_value("has_unwrap".to_owned(), EvalValue::Boolean(has_unwrap));

        let has_expect = rule_context
            .file_contents
            .values()
            .any(|content| content.contains(EXPECT_CALL));
        let _ = ctx.set_value("has_expect".to_owned(), EvalValue::Boolean(has_expect));

        // Check for async patterns
        let has_async = rule_context
            .file_contents
            .values()
            .any(|content| content.contains(ASYNC_FN_PREFIX));
        let _ = ctx.set_value("has_async".to_owned(), EvalValue::Boolean(has_async));

        // Check for test patterns (supports both absolute and relative paths)
        let has_tests = rule_context.file_contents.keys().any(|path| {
            path.contains(TEST_DIR_FRAGMENT)
                || path.starts_with("tests/")
                || path.contains(TEST_FILE_SUFFIX)
                || path.contains("test_")
        });
        let _ = ctx.set_value("has_tests".to_owned(), EvalValue::Boolean(has_tests));

        ctx
    }

    /// Evaluate a simple expression
    ///
    /// # Errors
    ///
    /// Returns an error if the expression evaluation fails.
    pub fn evaluate_expression(&self, expression: &str, context: &RuleContext) -> Result<bool> {
        let eval_ctx = Self::build_eval_context(context);

        match evalexpr::eval_boolean_with_context(expression, &eval_ctx) {
            Ok(result) => Ok(result),
            Err(e) => Err(crate::ValidationError::Config(format!(
                "Expression evaluation error: {e}"
            ))),
        }
    }

    /// Evaluate with custom variables
    ///
    /// # Errors
    ///
    /// Returns an error if the expression evaluation fails.
    pub fn evaluate_with_variables(
        &self,
        expression: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        let mut ctx = HashMapContext::new();

        for (key, value) in variables {
            let eval_value = Self::json_to_eval_value(value);
            let _ = ctx.set_value(key.clone(), eval_value);
        }

        match evalexpr::eval_boolean_with_context(expression, &ctx) {
            Ok(result) => Ok(result),
            Err(e) => Err(crate::ValidationError::Config(format!(
                "Expression evaluation error: {e}"
            ))),
        }
    }

    /// Convert JSON value to evalexpr value (associated function to avoid `only_used_in_recursion`).
    fn json_to_eval_value(value: &serde_json::Value) -> EvalValue {
        match value {
            serde_json::Value::Null => EvalValue::Empty,
            serde_json::Value::Bool(b) => EvalValue::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    EvalValue::Int(i)
                } else if let Some(f) = n.as_f64() {
                    EvalValue::Float(f)
                } else {
                    EvalValue::Empty
                }
            }
            serde_json::Value::String(s) => EvalValue::String(s.clone()),
            serde_json::Value::Array(arr) => {
                let tuple: Vec<EvalValue> = arr.iter().map(Self::json_to_eval_value).collect();
                EvalValue::Tuple(tuple)
            }
            serde_json::Value::Object(_) => {
                // Objects not directly supported, convert to string
                EvalValue::String(value.to_string())
            }
        }
    }

    /// Execute expression-based rule and generate violations
    ///
    /// # Errors
    ///
    /// Returns an error if expression evaluation fails.
    pub async fn execute_expression_rule(
        &self,
        input: ExpressionRuleInput<'_>,
    ) -> Result<Vec<RuleViolation>> {
        let ExpressionRuleInput {
            rule_id,
            expression,
            context,
            message,
            severity,
            category,
        } = input;

        let mut violations = Vec::new();

        match self.evaluate_expression(expression, context) {
            Ok(true) => {
                // Expression matched - generate violation
                violations.push(
                    RuleViolation::new(rule_id, category, severity, message)
                        .with_context(format!("Expression: {expression}")),
                );
            }
            Ok(false) => {
                // Expression did not match - no violation
            }
            Err(e) => {
                // Expression evaluation failed - report as warning
                violations.push(
                    RuleViolation::new(
                        rule_id,
                        ViolationCategory::Configuration,
                        Severity::Warning,
                        format!("Expression evaluation failed: {e}"),
                    )
                    .with_context(format!("Expression: {expression}")),
                );
            }
        }

        Ok(violations)
    }
}

#[async_trait]
impl RuleEngine for ExpressionEngine {
    async fn execute(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // Extract expression from rule definition
        let expression = rule_definition
            .get(YAML_FIELD_EXPRESSION)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::ValidationError::Config(
                    "Missing 'expression' field in rule definition".into(),
                )
            })?;

        let rule_id = rule_definition
            .get(YAML_FIELD_ID)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_EXPR_RULE_ID);

        let message = rule_definition
            .get(YAML_FIELD_MESSAGE)
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_EXPR_MESSAGE);

        let severity = rule_definition
            .get(YAML_FIELD_SEVERITY)
            .and_then(|v| v.as_str())
            .map_or(Severity::Warning, |s| match s {
                SEVERITY_ERROR => Severity::Error,
                SEVERITY_WARNING => Severity::Warning,
                _ => Severity::Info,
            });

        let category = rule_definition
            .get(YAML_FIELD_CATEGORY)
            .and_then(|v| v.as_str())
            .map_or(ViolationCategory::Quality, |c| match c {
                crate::constants::severities::CATEGORY_ARCHITECTURE => {
                    ViolationCategory::Architecture
                }
                crate::constants::severities::CATEGORY_PERFORMANCE => {
                    ViolationCategory::Performance
                }
                _ => ViolationCategory::Quality,
            });

        self.execute_expression_rule(ExpressionRuleInput {
            rule_id,
            expression,
            context,
            message,
            severity,
            category,
        })
        .await
    }
}

impl Clone for ExpressionEngine {
    fn clone(&self) -> Self {
        Self {
            cached_contexts: self.cached_contexts.clone(),
        }
    }
}
