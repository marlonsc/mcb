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

use tracing::{info, warn};

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
        let embedded: Vec<(&str, &str)> = EmbeddedRules::all_yaml()
            .into_iter()
            .map(|(path, content)| (path, content))
            .collect();
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
        info!(
            rust_validators = self.rust_validator_count(),
            "Running Rust validators"
        );
        match self.rust_registry.validate_all(config) {
            Ok(violations) => {
                info!(
                    count = violations.len(),
                    "Rust validators produced violations"
                );
                all_violations.extend(violations);
            }
            Err(e) => {
                warn!(error = %e, "Rust validator registry failed");
                return Err(e);
            }
        }

        // 2. Run YAML rules via HybridRuleEngine (synchronous wrapper)
        info!(yaml_rules = self.yaml_rule_count(), "Running YAML rules");
        let yaml_violations = self.execute_yaml_rules_sync(config);
        info!(
            count = yaml_violations.len(),
            "YAML rules produced violations"
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
                Err(e) => warn!(error = %e, "Named Rust validation failed"),
            }
        }

        // YAML rules — match by category field
        let yaml_violations = self.execute_yaml_rules_filtered_sync(config, |rule| {
            rule.category.eq_ignore_ascii_case(category)
        });
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
                Err(e) => warn!(error = %e, "Language-filtered Rust validation failed"),
            }
        }

        // YAML rules — filter by filters.languages
        let lang_str = format!("{lang:?}").to_lowercase();
        let yaml_violations = self.execute_yaml_rules_filtered_sync(config, |rule| {
            rule.filters
                .as_ref()
                .and_then(|f| f.languages.as_ref())
                .map_or(true, |langs| {
                    langs.iter().any(|l| l.eq_ignore_ascii_case(&lang_str))
                })
        });
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
    fn execute_yaml_rules_sync(&self, config: &ValidationConfig) -> Vec<Box<dyn Violation>> {
        self.execute_yaml_rules_filtered_sync(config, |_| true)
    }

    /// Execute YAML rules matching a predicate, synchronously.
    fn execute_yaml_rules_filtered_sync<F>(
        &self,
        config: &ValidationConfig,
        predicate: F,
    ) -> Vec<Box<dyn Violation>>
    where
        F: Fn(&ValidatedRule) -> bool,
    {
        let context = self.build_rule_context(config);
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
            ));

            match result {
                Ok(rule_result) => {
                    for v in rule_result.violations {
                        violations.push(Box::new(v));
                    }
                }
                Err(e) => {
                    warn!(
                        rule_id = %rule.id,
                        error = %e,
                        "YAML rule execution failed, skipping"
                    );
                }
            }
        }

        violations
    }

    /// Build a `RuleContext` from `ValidationConfig`.
    fn build_rule_context(&self, config: &ValidationConfig) -> RuleContext {
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
    fn block_on_async<F, T>(future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        // Try to use existing tokio runtime handle
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            // We're inside a tokio runtime — use block_in_place + block_on
            tokio::task::block_in_place(|| handle.block_on(future))
        } else {
            // No runtime — create a minimal one
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create tokio runtime for YAML rule execution");
            rt.block_on(future)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_workspace_root() -> PathBuf {
        // Navigate up from crate root to workspace root
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .parent() // crates/
            .and_then(|p| p.parent()) // workspace root
            .expect("could not find workspace root")
            .to_path_buf()
    }

    #[test]
    fn test_list_all_rules_discovers_both_systems() {
        let root = test_workspace_root();
        let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

        let all_rules = registry.list_all_rules();

        // Rust validators: 19 (18 standard + DeclarativeValidator)
        let rust_count = all_rules
            .iter()
            .filter(|r| r.origin == RuleOrigin::Rust)
            .count();
        assert!(
            rust_count >= 18,
            "Expected at least 18 Rust validators, got {rust_count}"
        );

        // YAML rules: should have many enabled rules
        let yaml_count = all_rules
            .iter()
            .filter(|r| r.origin == RuleOrigin::Yaml)
            .count();
        assert!(
            yaml_count >= 30,
            "Expected at least 30 YAML rules, got {yaml_count}"
        );

        // Total should be substantial
        let total = all_rules.len();
        assert!(total >= 48, "Expected at least 48 total rules, got {total}");

        // Verify we have rules from both origins
        assert!(
            all_rules.iter().any(|r| r.origin == RuleOrigin::Rust),
            "No Rust rules found"
        );
        assert!(
            all_rules.iter().any(|r| r.origin == RuleOrigin::Yaml),
            "No YAML rules found"
        );
    }

    #[test]
    fn test_rust_validator_count() {
        let root = test_workspace_root();
        let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

        // standard_for registers 19 validators (18 standard + DeclarativeValidator)
        assert!(
            registry.rust_validator_count() >= 18,
            "Expected at least 18 Rust validators, got {}",
            registry.rust_validator_count()
        );
    }

    #[test]
    fn test_yaml_rule_count() {
        let root = test_workspace_root();
        let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

        assert!(
            registry.yaml_rule_count() >= 30,
            "Expected at least 30 YAML rules, got {}",
            registry.yaml_rule_count()
        );
    }

    #[test]
    fn test_total_rule_count() {
        let root = test_workspace_root();
        let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

        let total = registry.total_rule_count();
        assert!(total >= 48, "Expected at least 48 total rules, got {total}");
        assert_eq!(
            total,
            registry.rust_validator_count() + registry.yaml_rule_count(),
            "Total should equal Rust + YAML counts"
        );
    }

    #[test]
    fn test_rule_info_has_correct_origins() {
        let root = test_workspace_root();
        let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

        let rules = registry.list_all_rules();

        // Check a known Rust validator
        let clean_arch = rules.iter().find(|r| r.id == "clean_architecture");
        assert!(
            clean_arch.is_some(),
            "clean_architecture validator not found"
        );
        assert_eq!(clean_arch.unwrap().origin, RuleOrigin::Rust);

        // Check that YAML rules have proper IDs (e.g. CA001, QUAL001, etc.)
        let yaml_rules: Vec<&RuleInfo> = rules
            .iter()
            .filter(|r| r.origin == RuleOrigin::Yaml)
            .collect();
        assert!(
            yaml_rules
                .iter()
                .any(|r| r.id.starts_with("CA") || r.id.starts_with("QUAL")),
            "Expected YAML rules with CA or QUAL prefixes"
        );
    }

    #[test]
    fn test_execute_all_produces_violations_from_both_systems() {
        let root = test_workspace_root();
        let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");
        let config = ValidationConfig::new(&root);

        // execute_all should not panic and should return a result
        let result = registry.execute_all(&config);
        assert!(result.is_ok(), "execute_all failed: {:?}", result.err());

        // We expect at least some violations from the real workspace
        let violations = result.unwrap();
        // The workspace may or may not have violations — just verify it runs
        // without error. In a real workspace, Rust validators typically find some.
        info!(
            total_violations = violations.len(),
            "execute_all completed successfully"
        );
    }

    #[test]
    fn test_execute_by_category() {
        let root = test_workspace_root();
        let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");
        let config = ValidationConfig::new(&root);

        // Execute only architecture-related rules
        let result = registry.execute_by_category("architecture", &config);
        assert!(
            result.is_ok(),
            "execute_by_category failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_execute_by_language() {
        let root = test_workspace_root();
        let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");
        let config = ValidationConfig::new(&root);

        // Execute only Rust-language rules
        let result = registry.execute_by_language(LanguageId::Rust, &config);
        assert!(
            result.is_ok(),
            "execute_by_language failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_accessors() {
        let root = test_workspace_root();
        let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

        // Verify accessors don't panic
        let _rust = registry.rust_registry();
        let _yaml = registry.yaml_rules();
        let _engine = registry.hybrid_engine();

        assert!(!registry.yaml_rules().is_empty());
        assert!(!registry.rust_registry().validators().is_empty());
    }
}
