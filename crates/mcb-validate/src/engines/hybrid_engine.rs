//! Hybrid Rule Engine
//!
//! Orchestrates multiple rule engines for maximum flexibility:
//! - rust-rule-engine: Complex rules with RETE algorithm
//! - rusty-rules: Composable rules with JSON DSL
//! - validator/garde: Field validations

use std::collections::HashMap;

use async_trait::async_trait;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use super::router::{RoutedEngine, RuleEngineRouter};
use super::validator_engine::ValidatorEngine;
use crate::Result;
use crate::ValidationConfig;
use crate::traits::violation::{Severity, Violation, ViolationCategory};

/// Types of rule engines supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleEngineType {
    /// RETE-UL algorithm with GRL syntax
    RustRuleEngine,
    /// JSON DSL with composition (all/any/not)
    RustyRules,
    /// Simple boolean expression evaluation
    Expression,
    /// Auto-detect engine based on rule content
    Auto,
}

/// Concrete violation structure for rule engines
#[derive(Debug, Clone, Display)]
#[display("[{id}] {message}")]
pub struct RuleViolation {
    /// Unique identifier for the violation
    pub id: String,
    /// Category of the violation (SOLID, quality, etc.)
    pub category: ViolationCategory,
    /// Severity level
    pub severity: Severity,
    /// Detailed error message
    pub message: String,
    /// Path to the file containing the violation
    pub file: Option<std::path::PathBuf>,
    /// Line number of the violation
    pub line: Option<usize>,
    /// Column number of the violation
    pub column: Option<usize>,
    /// Additional context or code snippet
    pub context: Option<String>,
}

impl Violation for RuleViolation {
    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ViolationCategory {
        self.category
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn file(&self) -> Option<&std::path::PathBuf> {
        self.file.as_ref()
    }

    fn line(&self) -> Option<usize> {
        self.line
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

impl RuleViolation {
    /// Create a new rule violation
    pub fn new(
        id: impl Into<String>,
        category: ViolationCategory,
        severity: Severity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            severity,
            message: message.into(),
            file: None,
            line: None,
            column: None,
            context: None,
        }
    }

    /// Attach personal file to the violation
    #[must_use]
    pub fn with_file(mut self, file: std::path::PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    /// Set the location of the violation
    #[must_use]
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Attach context information
    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Result of rule execution
#[derive(Debug, Clone)]
pub struct RuleResult {
    /// List of violations found
    pub violations: Vec<RuleViolation>,
    /// Total time taken for execution
    pub execution_time_ms: u64,
}

use crate::extractor::Fact;
use crate::graph::DependencyGraph;
use std::sync::Arc;

/// Context passed to rule engines during execution
#[derive(Debug, Clone)]
pub struct RuleContext {
    /// Path to the workspace root
    pub workspace_root: std::path::PathBuf,
    /// Global validation configuration
    pub config: ValidationConfig,
    /// Pre-parsed AST data for analysis
    pub ast_data: HashMap<String, serde_json::Value>,
    /// Cargo package metadata
    pub cargo_data: HashMap<String, serde_json::Value>,
    /// Raw file contents
    pub file_contents: HashMap<String, String>,
    /// Extracted Facts (New System)
    pub facts: Arc<Vec<Fact>>,
    /// Dependency Graph (New System)
    pub graph: Arc<DependencyGraph>,
}

/// Hybrid engine that coordinates multiple rule engines.
///
/// ## Architecture
///
/// The `HybridRuleEngine` is the top-level orchestrator. It delegates all rule
/// execution to the [`RuleEngineRouter`], which owns the three rule engines
/// (RETE, Expression, `RustyRules`) and selects the appropriate one per rule.
///
/// Additionally, the `ValidatorEngine` handles rule _definition_ validation
/// (field-level checks on rule JSON structure) — this is a separate concern
/// from rule _execution_.
///
/// ```text
/// HybridRuleEngine (orchestrator)
///   ├── RuleEngineRouter (dispatch + execution)
///   │   ├── ReteEngine        (GRL / when-then)
///   │   ├── ExpressionEngine   (evalexpr booleans)
///   │   └── RustyRulesEngineWrapper (JSON DSL)
///   ├── ValidatorEngine (rule definition validation)
///   └── cache: HashMap<String, Vec<u8>> (compiled rule cache)
/// ```
pub struct HybridRuleEngine {
    router: RuleEngineRouter,
    validator_engine: ValidatorEngine,
    cache: HashMap<String, Vec<u8>>, // Compiled rule cache
}

impl HybridRuleEngine {
    /// Create a new hybrid rule engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            router: RuleEngineRouter::new(),
            validator_engine: ValidatorEngine::new(),
            cache: HashMap::new(),
        }
    }

    /// Execute a rule using the appropriate engine
    pub async fn execute_rule(
        &self,
        _rule_id: &str,
        engine_type: RuleEngineType,
        rule_definition: &serde_json::Value,
        context: &RuleContext,
    ) -> Result<RuleResult> {
        let start_time = std::time::Instant::now();

        let violations = match engine_type {
            RuleEngineType::RustRuleEngine => {
                self.router
                    .execute_with_engine(RoutedEngine::Rete, rule_definition, context)
                    .await?
            }
            RuleEngineType::RustyRules => {
                self.router
                    .execute_with_engine(RoutedEngine::RustyRules, rule_definition, context)
                    .await?
            }
            RuleEngineType::Expression => {
                self.router
                    .execute_with_engine(RoutedEngine::Expression, rule_definition, context)
                    .await?
            }
            RuleEngineType::Auto => self.router.execute(rule_definition, context).await?,
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(RuleResult {
            violations,
            execution_time_ms: execution_time,
        })
    }

    /// Execute a rule with automatic engine detection
    ///
    /// Uses the router to analyze the rule definition and select
    /// the most appropriate engine.
    pub async fn execute_auto(
        &self,
        rule_definition: &serde_json::Value,
        context: &RuleContext,
    ) -> Result<RuleResult> {
        let start_time = std::time::Instant::now();

        let violations = self.router.execute(rule_definition, context).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(RuleResult {
            violations,
            execution_time_ms: execution_time,
        })
    }

    /// Get the engine type that would be used for a rule
    #[must_use]
    pub fn detect_engine(&self, rule_definition: &serde_json::Value) -> String {
        self.router.get_engine_type(rule_definition)
    }

    /// Execute multiple rules in parallel
    pub async fn execute_rules_batch(
        &self,
        rules: Vec<(String, RuleEngineType, serde_json::Value)>,
        context: &RuleContext,
    ) -> Result<Vec<(String, RuleResult)>> {
        let mut handles = Vec::new();

        for (rule_id, engine_type, rule_def) in rules {
            let engine = self.clone();
            let ctx = context.clone();
            let rule_id_clone = rule_id.clone();

            let handle = tokio::spawn(async move {
                let result = engine
                    .execute_rule(&rule_id_clone, engine_type, &rule_def, &ctx)
                    .await;
                (rule_id_clone, result)
            });

            handles.push((rule_id, handle));
        }

        let mut results = Vec::new();
        for (rule_id, handle) in handles {
            match handle.await {
                Ok((returned_rule_id, Ok(result))) => results.push((returned_rule_id, result)),
                Ok((returned_rule_id, Err(e))) => {
                    warn!(rule_id = %returned_rule_id, error = %e, "Rule execution error");
                }
                Err(e) => {
                    error!(rule_id = %rule_id, error = %e, "Task join error");
                }
            }
        }

        Ok(results)
    }

    /// Validate rule definition using validator/garde
    pub fn validate_rule_definition(&self, rule_definition: &serde_json::Value) -> Result<()> {
        self.validator_engine
            .validate_rule_definition(rule_definition)
    }

    /// Get cached compiled rule
    #[must_use]
    pub fn get_cached_rule(&self, rule_id: &str) -> Option<&Vec<u8>> {
        self.cache.get(rule_id)
    }

    /// Cache compiled rule
    pub fn cache_compiled_rule(&mut self, rule_id: String, compiled: Vec<u8>) {
        self.cache.insert(rule_id, compiled);
    }

    /// Clear rule cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Execute linter-based validation for rules with `lint_select`.
    ///
    /// Delegates to `YamlRuleExecutor` for linter detection and execution,
    /// then converts `LintViolation`s to [`RuleViolation`]s.
    pub async fn execute_lint_rule(
        &self,
        rule_id: &str,
        lint_select: &[String],
        context: &RuleContext,
        custom_message: Option<&str>,
        severity: Severity,
        category: ViolationCategory,
    ) -> Result<RuleResult> {
        use crate::linters::YamlRuleExecutor;
        use crate::rules::yaml_loader::ValidatedRule;

        let start_time = std::time::Instant::now();

        let lint_adapter_rule = ValidatedRule {
            id: rule_id.to_owned(),
            name: String::new(),
            category: String::new(),
            severity: String::new(),
            enabled: true,
            description: String::new(),
            rationale: String::new(),
            engine: String::new(),
            config: serde_json::Value::Null,
            rule_definition: serde_json::Value::Null,
            fixes: Vec::new(),
            lint_select: lint_select.to_vec(),
            message: custom_message.map(String::from),
            selectors: Vec::new(),
            ast_query: None,
            metrics: None,
            filters: None,
        };

        let files: Vec<std::path::PathBuf> = context
            .file_contents
            .keys()
            .map(std::path::PathBuf::from)
            .collect();
        let file_refs: Vec<&std::path::Path> =
            files.iter().map(std::path::PathBuf::as_path).collect();

        let lint_violations =
            YamlRuleExecutor::execute_rule(&lint_adapter_rule, &file_refs).await?;

        let violations: Vec<RuleViolation> = lint_violations
            .into_iter()
            .map(|lv| {
                let msg = custom_message
                    .map_or_else(|| lv.message.clone(), |m| format!("{m}: {}", lv.message));
                RuleViolation::new(rule_id, category, severity, msg)
                    .with_file(std::path::PathBuf::from(&lv.file))
                    .with_location(lv.line, lv.column)
                    .with_context(format!("Linter: {} ({})", lv.rule, lv.category))
            })
            .collect();

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(RuleResult {
            violations,
            execution_time_ms: execution_time,
        })
    }

    /// Check if a rule uses `lint_select` (linter-based validation)
    #[must_use]
    pub fn is_lint_rule(rule_definition: &serde_json::Value) -> bool {
        rule_definition
            .get("lint_select")
            .and_then(|v| v.as_array())
            .is_some_and(|arr| !arr.is_empty())
    }
}

impl Default for HybridRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HybridRuleEngine {
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
            validator_engine: self.validator_engine.clone(),
            cache: self.cache.clone(),
        }
    }
}

/// Trait for rule engines.
///
/// # Example
///
/// ```rust,no_run
/// use async_trait::async_trait;
/// use mcb_validate::engines::hybrid_engine::{RuleEngine, RuleContext, RuleViolation};
/// use mcb_validate::Result;
///
/// struct MyRuleEngine;
///
/// #[async_trait]
/// impl RuleEngine for MyRuleEngine {
///     async fn execute(
///         &self,
///         rule_definition: &serde_json::Value,
///         context: &RuleContext,
///     ) -> Result<Vec<RuleViolation>> {
///         Ok(vec![])
///     }
/// }
/// ```
#[async_trait]
pub trait RuleEngine: Send + Sync {
    /// Execute the rule against the provided context
    async fn execute(
        &self,
        rule_definition: &serde_json::Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>>;
}
