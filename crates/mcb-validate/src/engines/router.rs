//! Rule Engine Router
//!
//! Routes rules to the appropriate engine based on complexity detection.
//!
//! Routing logic:
//! - Rules with "when"/"then" keywords -> RETE engine (GRL syntax)
//! - Rules with "expression" field -> Expression engine (evalexpr)
//! - Rules with "condition"/"action" -> Rusty Rules engine (JSON DSL)
//! - Default fallback -> Rusty Rules engine

use serde_json::Value;

use crate::Result;
use crate::engines::expression_engine::ExpressionEngine;
use crate::engines::hybrid_engine::{RuleContext, RuleViolation};
use crate::engines::rete_engine::ReteEngine;
use crate::engines::rusty_rules_engine::RustyRulesEngineWrapper;

/// Engine type determined by router
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutedEngine {
    /// RETE engine for GRL rules with when/then
    Rete,
    /// Expression engine for simple boolean expressions
    Expression,
    /// Rusty Rules engine for JSON DSL rules
    RustyRules,
}

impl std::fmt::Display for RoutedEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rete => write!(f, "RETE"),
            Self::Expression => write!(f, "Expression"),
            Self::RustyRules => write!(f, "RustyRules"),
        }
    }
}

/// Rule Engine Router
///
/// Analyzes rule definitions and routes them to the appropriate engine.
#[allow(clippy::struct_field_names)]
pub struct RuleEngineRouter {
    /// Engine for processing complex rules using RETE algorithm
    rete_engine: ReteEngine,
    /// Engine for processing simple boolean expressions
    expression_engine: ExpressionEngine,
    /// Engine for processing rules using JSON DSL
    rusty_rules_engine: RustyRulesEngineWrapper,
}

impl Default for RuleEngineRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleEngineRouter {
    /// Create a new rule engine router with all available engines.
    pub fn new() -> Self {
        Self {
            rete_engine: ReteEngine::new(),
            expression_engine: ExpressionEngine::new(),
            rusty_rules_engine: RustyRulesEngineWrapper::new(),
        }
    }

    /// Detect which engine should handle the rule
    pub fn detect_engine(&self, rule_definition: &Value) -> RoutedEngine {
        // Check for explicit engine specification
        if let Some(engine) = rule_definition.get("engine").and_then(|v| v.as_str()) {
            return match engine {
                "rete" | "rust-rule-engine" | "grl" => RoutedEngine::Rete,
                "expression" | "evalexpr" => RoutedEngine::Expression,
                "rusty-rules" | "json-dsl" => RoutedEngine::RustyRules,
                _ => self.detect_by_content(rule_definition),
            };
        }

        self.detect_by_content(rule_definition)
    }

    /// Detect engine based on rule content
    fn detect_by_content(&self, rule_definition: &Value) -> RoutedEngine {
        // Check for GRL syntax (when/then keywords)
        if self.has_grl_syntax(rule_definition) {
            return RoutedEngine::Rete;
        }

        // Check for expression field
        if rule_definition.get("expression").is_some() {
            return RoutedEngine::Expression;
        }

        // Check for JSON DSL structure
        if rule_definition.get("condition").is_some() || rule_definition.get("action").is_some() {
            return RoutedEngine::RustyRules;
        }

        // Default to RustyRules
        RoutedEngine::RustyRules
    }

    /// Check if rule contains GRL syntax (when/then)
    fn has_grl_syntax(&self, rule_definition: &Value) -> bool {
        // Check "rule" or "grl" field for when/then keywords
        if let Some(rule_str) = rule_definition
            .get("rule")
            .or_else(|| rule_definition.get("grl"))
            .and_then(|v| v.as_str())
        {
            let lower = rule_str.to_lowercase();
            return lower.contains("when") && lower.contains("then");
        }

        // Check if there's a rule definition with GRL markers
        if let Some(rule_str) = rule_definition
            .get("rule_definition")
            .and_then(|v| v.as_str())
        {
            let lower = rule_str.to_lowercase();
            return lower.contains("when") && lower.contains("then");
        }

        false
    }

    /// Route and execute rule (auto-detects engine from rule content)
    pub async fn execute(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let engine = self.detect_engine(rule_definition);
        self.execute_with_engine(engine, rule_definition, context)
            .await
    }

    /// Execute rule with a specific engine (bypasses auto-detection)
    pub async fn execute_with_engine(
        &self,
        engine: RoutedEngine,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        match engine {
            RoutedEngine::Rete => self.execute_with_rete(rule_definition, context).await,
            RoutedEngine::Expression => {
                self.execute_with_expression(rule_definition, context).await
            }
            RoutedEngine::RustyRules => {
                self.execute_with_rusty_rules(rule_definition, context)
                    .await
            }
        }
    }

    /// Execute with RETE engine
    async fn execute_with_rete(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        use crate::engines::hybrid_engine::RuleEngine;
        self.rete_engine.execute(rule_definition, context).await
    }

    /// Execute with Expression engine
    async fn execute_with_expression(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        use crate::engines::hybrid_engine::RuleEngine;
        self.expression_engine
            .execute(rule_definition, context)
            .await
    }

    /// Execute with Rusty Rules engine
    async fn execute_with_rusty_rules(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        use crate::engines::hybrid_engine::RuleEngine;
        self.rusty_rules_engine
            .execute(rule_definition, context)
            .await
    }

    /// Get the engine type for a rule (for logging/debugging)
    pub fn get_engine_type(&self, rule_definition: &Value) -> String {
        self.detect_engine(rule_definition).to_string()
    }

    /// Check if a rule is valid for routing
    pub fn validate_rule(&self, rule_definition: &Value) -> Result<()> {
        let engine = self.detect_engine(rule_definition);

        match engine {
            RoutedEngine::Rete => {
                // Validate GRL syntax
                if rule_definition.get("rule").is_none() && rule_definition.get("grl").is_none() {
                    return Err(crate::ValidationError::Config(
                        "RETE rule must have 'rule' or 'grl' field".into(),
                    ));
                }
            }
            RoutedEngine::Expression => {
                // Validate expression
                if rule_definition.get("expression").is_none() {
                    return Err(crate::ValidationError::Config(
                        "Expression rule must have 'expression' field".into(),
                    ));
                }
            }
            RoutedEngine::RustyRules => {
                // RustyRules is flexible, minimal validation
            }
        }

        Ok(())
    }
}

impl Clone for RuleEngineRouter {
    fn clone(&self) -> Self {
        Self {
            rete_engine: self.rete_engine.clone(),
            expression_engine: self.expression_engine.clone(),
            rusty_rules_engine: self.rusty_rules_engine.clone(),
        }
    }
}
