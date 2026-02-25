//! Unified Rule Registry
//!
//! Bridges Rust validators, YAML rules, and AST engines into a single
//! discovery and execution surface. This is the top-level entry point
//! for running **all** validation rules regardless of their origin.
//!
//! ## Architecture
//!
//! ```text
//! UnifiedRuleRegistry
//!   ├── ValidatorRegistry  (18 Rust validators via Validator trait)
//!   ├── YamlRuleLoader     (40+ embedded YAML rules)
//!   └── HybridRuleEngine   (5 execution engines for YAML rules)
//! ```

use std::path::PathBuf;

use crate::embedded_rules::EmbeddedRules;
use crate::engines::hybrid_engine::{HybridRuleEngine, RuleContext, RuleEngineType};
use crate::filters::LanguageId;
use crate::rules::yaml_loader::{ValidatedRule, YamlRuleLoader};
use crate::traits::validator::ValidatorRegistry;
use crate::traits::violation::Violation;
use crate::validators::declarative_support::build_substitution_variables;
use crate::{Result, ValidationConfig};

// ---------------------------------------------------------------------------
// RuleInfo — unified metadata for any rule regardless of origin
// ---------------------------------------------------------------------------

/// Origin of a rule (Rust compiled validator or YAML-defined rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleOrigin {
    /// Compiled Rust validator (implements `Validator` trait).
    Rust,
    /// YAML-defined rule executed via `HybridRuleEngine`.
    Yaml,
}

/// Unified metadata for a single rule.
#[derive(Debug, Clone)]
pub struct RuleInfo {
    /// Unique rule identifier (e.g. `"clean_architecture"` or `"CA001"`).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Category string (e.g. `"Architecture"`, `"quality"`).
    pub category: String,
    /// Severity level.
    pub severity: String,
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Short description.
    pub description: String,
    /// Where the rule comes from.
    pub origin: RuleOrigin,
    /// Languages this rule applies to (empty = all).
    pub languages: Vec<String>,
}

// ---------------------------------------------------------------------------
// UnifiedRuleRegistry
// ---------------------------------------------------------------------------

/// Unified registry that discovers and executes **all** validation rules.
///
/// Wraps:
/// - [`ValidatorRegistry`] — 18 compiled Rust validators
/// - [`YamlRuleLoader`] — 40+ embedded YAML rules
/// - [`HybridRuleEngine`] — 5 execution engines for YAML rules
pub struct UnifiedRuleRegistry {
    /// Compiled Rust validators.
    rust_registry: ValidatorRegistry,
    /// Loaded YAML rules.
    yaml_rules: Vec<ValidatedRule>,
    /// Engine for executing YAML rules.
    hybrid_engine: HybridRuleEngine,
    /// Workspace root for context building.
    _workspace_root: PathBuf,
}

impl UnifiedRuleRegistry {
    /// Build a unified registry for the given workspace root.
    ///
    /// Loads the standard 18 Rust validators and all embedded YAML rules.
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML rule loader fails to initialise.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Result<Self> {
        let root = workspace_root.into();
        let rust_registry = ValidatorRegistry::standard_for(&root);

        // Load embedded YAML rules
        let embedded: Vec<(&str, &str)> = EmbeddedRules::all_yaml().into_iter().collect();
        let variables = build_substitution_variables(&root);
        let mut loader = YamlRuleLoader::from_embedded_with_variables(&embedded, Some(variables))?;
        let yaml_rules = loader.load_embedded_rules()?;

        let hybrid_engine = HybridRuleEngine::new();

        Ok(Self {
            rust_registry,
            yaml_rules,
            hybrid_engine,
            _workspace_root: root,
        })
    }

    // -----------------------------------------------------------------------
    // Discovery
    // -----------------------------------------------------------------------

    /// List metadata for **all** rules (Rust + YAML).
    #[must_use]
    pub fn list_all_rules(&self) -> Vec<RuleInfo> {
        let mut rules = Vec::new();

        // Rust validators
        for validator in self.rust_registry.validators() {
            let langs: Vec<String> = validator
                .supported_languages()
                .iter()
                .map(|l| format!("{l:?}"))
                .collect();
            rules.push(RuleInfo {
                id: validator.name().to_owned(),
                name: validator.name().to_owned(),
                category: String::from("rust-validator"),
                severity: String::from("error"),
                enabled: validator.enabled_by_default(),
                description: validator.description().to_owned(),
                origin: RuleOrigin::Rust,
                languages: langs,
            });
        }

        // YAML rules (excluding templates)
        for rule in &self.yaml_rules {
            if !rule.enabled {
                continue;
            }
            let langs: Vec<String> = rule
                .filters
                .as_ref()
                .and_then(|f| f.languages.clone())
                // INTENTIONAL: Language filter extraction; empty filter means no filtering
                .unwrap_or_default();
            rules.push(RuleInfo {
                id: rule.id.clone(),
                name: rule.name.clone(),
                category: rule.category.clone(),
                severity: rule.severity.clone(),
                enabled: rule.enabled,
                description: rule.description.clone(),
                origin: RuleOrigin::Yaml,
                languages: langs,
            });
        }

        rules
    }

    /// Number of Rust validators.
    #[must_use]
    pub fn rust_validator_count(&self) -> usize {
        self.rust_registry.validators().len()
    }

    /// Number of loaded YAML rules (enabled only).
    #[must_use]
    pub fn yaml_rule_count(&self) -> usize {
        self.yaml_rules.iter().filter(|r| r.enabled).count()
    }

    /// Total rule count (Rust + enabled YAML).
    #[must_use]
    pub fn total_rule_count(&self) -> usize {
        self.rust_validator_count() + self.yaml_rule_count()
    }

    // -----------------------------------------------------------------------
    // Execution
    // -----------------------------------------------------------------------

    /// Execute **all** rules (Rust validators + YAML rules).
    ///
    /// Rust validators run synchronously via `ValidatorRegistry::validate_all`.
    /// YAML rules run via `HybridRuleEngine::execute_rule`.
    ///
    /// # Errors
    ///
    /// Returns an error if the Rust validation context cannot be built.
    pub fn execute_all(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
        let mut all_violations: Vec<Box<dyn Violation>> = Vec::new();

        // 1. Run Rust validators
        mcb_domain::info!(
            "unified_registry",
            &format!(
                "Running Rust validators (count={})",
                self.rust_validator_count()
            )
        );
        match self.rust_registry.validate_all(config) {
            Ok(violations) => {
                mcb_domain::info!(
                    "unified_registry",
                    &format!("Rust validators produced {} violations", violations.len())
                );
                all_violations.extend(violations);
            }
            Err(e) => {
                mcb_domain::warn!("unified_registry", "Rust validator registry failed", &e);
                return Err(e);
            }
        }

        // 2. Run YAML rules via HybridRuleEngine (synchronous wrapper)
        mcb_domain::info!(
            "unified_registry",
            &format!("Running YAML rules (count={})", self.yaml_rule_count())
        );
        let yaml_violations = self.execute_yaml_rules_sync(config)?;
        mcb_domain::info!(
            "unified_registry",
            &format!("YAML rules produced {} violations", yaml_violations.len())
        );
        all_violations.extend(yaml_violations);

        Ok(all_violations)
    }

    /// Execute rules filtered by category string.
    ///
    /// For Rust validators, the category is matched against the validator name.
    /// For YAML rules, the category field is compared directly.
    ///
    /// # Errors
    ///
    /// Returns an error if the Rust validation context cannot be built.
    pub fn execute_by_category(
        &self,
        category: &str,
        config: &ValidationConfig,
    ) -> Result<Vec<Box<dyn Violation>>> {
        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        // Rust validators — match by name (category is implicit in name)
        let matching_names: Vec<&str> = self
            .rust_registry
            .validators()
            .iter()
            .filter(|v| v.name().contains(category))
            .map(|v| v.name())
            .collect();

        if !matching_names.is_empty() {
            match self.rust_registry.validate_named(config, &matching_names) {
                Ok(v) => violations.extend(v),
                Err(e) => mcb_domain::warn!("unified_registry", "Named Rust validation failed", &e),
            }
        }

        // YAML rules — match by category field
        let yaml_violations = self.execute_yaml_rules_filtered_sync(config, |rule| {
            rule.category.eq_ignore_ascii_case(category)
        })?;
        violations.extend(yaml_violations);

        Ok(violations)
    }

    /// Execute rules filtered by language.
    ///
    /// For Rust validators, checks `supported_languages()`.
    /// For YAML rules, checks the `filters.languages` field.
    ///
    /// # Errors
    ///
    /// Returns an error if the Rust validation context cannot be built.
    pub fn execute_by_language(
        &self,
        lang: LanguageId,
        config: &ValidationConfig,
    ) -> Result<Vec<Box<dyn Violation>>> {
        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        // Rust validators — filter by supported_languages
        let matching_names: Vec<&str> = self
            .rust_registry
            .validators()
            .iter()
            .filter(|v| {
                let langs = v.supported_languages();
                langs.is_empty() || langs.contains(&lang)
            })
            .map(|v| v.name())
            .collect();

        if !matching_names.is_empty() {
            match self.rust_registry.validate_named(config, &matching_names) {
                Ok(v) => violations.extend(v),
                Err(e) => mcb_domain::warn!(
                    "unified_registry",
                    "Language-filtered Rust validation failed",
                    &e
                ),
            }
        }

        // YAML rules — filter by filters.languages
        let lang_str = format!("{lang:?}").to_lowercase();
        let yaml_violations = self.execute_yaml_rules_filtered_sync(config, |rule| {
            rule.filters
                .as_ref()
                .and_then(|f| f.languages.as_ref())
                .is_none_or(|langs| langs.iter().any(|l| l.eq_ignore_ascii_case(&lang_str)))
        })?;
        violations.extend(yaml_violations);

        Ok(violations)
    }

    /// Access the underlying Rust validator registry.
    #[must_use]
    pub fn rust_registry(&self) -> &ValidatorRegistry {
        &self.rust_registry
    }

    /// Access the loaded YAML rules.
    #[must_use]
    pub fn yaml_rules(&self) -> &[ValidatedRule] {
        &self.yaml_rules
    }

    /// Access the hybrid engine.
    #[must_use]
    pub fn hybrid_engine(&self) -> &HybridRuleEngine {
        &self.hybrid_engine
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Execute all enabled YAML rules synchronously.
    ///
    /// Uses `tokio::runtime::Handle::current()` if inside a tokio context,
    /// otherwise creates a small blocking runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if a tokio runtime could not be created when not inside one.
    fn execute_yaml_rules_sync(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<Box<dyn Violation>>> {
        self.execute_yaml_rules_filtered_sync(config, |_| true)
    }

    /// Execute YAML rules matching a predicate, synchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if a tokio runtime could not be created when not inside one.
    fn execute_yaml_rules_filtered_sync<F>(
        &self,
        config: &ValidationConfig,
        predicate: F,
    ) -> Result<Vec<Box<dyn Violation>>>
    where
        F: Fn(&ValidatedRule) -> bool,
    {
        let context = Self::build_rule_context(config);
        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        for rule in &self.yaml_rules {
            if !rule.enabled || !predicate(rule) {
                continue;
            }

            let engine_type = if rule.engine.is_empty() {
                RuleEngineType::Auto
            } else {
                match rule.engine.as_str() {
                    "rete" | "rust-rule-engine" => RuleEngineType::RustRuleEngine,
                    "rusty-rules" => RuleEngineType::RustyRules,
                    "expression" => RuleEngineType::Expression,
                    _ => RuleEngineType::Auto,
                }
            };

            // Execute via blocking runtime
            let result = Self::block_on_async(self.hybrid_engine.execute_rule(
                &rule.id,
                engine_type,
                &rule.rule_definition,
                &context,
            ))?;

            match result {
                Ok(rule_result) => {
                    for v in rule_result.violations {
                        violations.push(Box::new(v));
                    }
                }
                Err(e) => {
                    mcb_domain::warn!(
                        "unified_registry",
                        "YAML rule execution failed, skipping",
                        &format!("rule_id={} error={}", rule.id, e)
                    );
                }
            }
        }

        Ok(violations)
    }

    /// Build a `RuleContext` from `ValidationConfig`.
    fn build_rule_context(config: &ValidationConfig) -> RuleContext {
        use std::collections::HashMap;
        use std::sync::Arc;

        RuleContext {
            workspace_root: config.workspace_root.clone(),
            config: config.clone(),
            ast_data: HashMap::new(),
            cargo_data: HashMap::new(),
            file_contents: HashMap::new(),
            facts: Arc::new(Vec::new()),
            graph: Arc::new(crate::graph::DependencyGraph::new()),
        }
    }

    /// Run an async future to completion, handling both tokio and non-tokio contexts.
    ///
    /// # Errors
    ///
    /// Returns an error if a new tokio runtime could not be created when not inside one.
    fn block_on_async<F, T>(future: F) -> Result<T>
    where
        F: std::future::Future<Output = T>,
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            Ok(tokio::task::block_in_place(|| handle.block_on(future)))
        } else {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    crate::ValidationError::Config(format!(
                        "failed to create tokio runtime for YAML rule execution: {e}"
                    ))
                })?;
            Ok(rt.block_on(future))
        }
    }
}
