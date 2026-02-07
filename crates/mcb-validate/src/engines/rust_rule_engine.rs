//! Rust Rule Engine Wrapper
//!
//! Wrapper that delegates to `ReteEngine` for actual rust-rule-engine execution.
//! This provides backwards compatibility while using the real library.

use async_trait::async_trait;
use serde_json::Value;

use crate::Result;
use crate::engines::hybrid_engine::RuleViolation;

use super::hybrid_engine::{RuleContext, RuleEngine};
use super::rete_engine::ReteEngine;

/// Wrapper for rust-rule-engine that delegates to `ReteEngine`
///
/// This struct provides backwards compatibility with existing code
/// while the actual GRL execution is handled by `ReteEngine` which
/// uses the real rust-rule-engine library.
pub struct RustRuleEngineWrapper {
    rete_engine: ReteEngine,
}

impl Default for RustRuleEngineWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl RustRuleEngineWrapper {
    /// Create a new Rust rule engine wrapper.
    pub fn new() -> Self {
        Self {
            rete_engine: ReteEngine::new(),
        }
    }

    /// Load and compile GRL rule
    pub fn compile_grl_rule(&mut self, _rule_id: String, grl_code: &str) -> Result<()> {
        self.rete_engine.load_grl(grl_code)
    }

    /// Execute compiled GRL rules against context
    pub async fn execute_compiled(
        &mut self,
        context: &RuleContext,
        grl_code: &str,
    ) -> Result<Vec<RuleViolation>> {
        self.rete_engine.execute_grl(grl_code, context).await
    }
}

#[async_trait]
impl RuleEngine for RustRuleEngineWrapper {
    async fn execute(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // Delegate to ReteEngine's RuleEngine implementation
        self.rete_engine.execute(rule_definition, context).await
    }
}

impl Clone for RustRuleEngineWrapper {
    fn clone(&self) -> Self {
        Self {
            rete_engine: self.rete_engine.clone(),
        }
    }
}
