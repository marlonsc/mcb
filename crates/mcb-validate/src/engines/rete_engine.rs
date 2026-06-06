//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! RETE Engine Wrapper
//!
//! Wrapper for rust-rule-engine crate implementing RETE-UL algorithm.
//! Use this engine for complex GRL rules with when/then syntax.
//!
//! Uses `cargo_metadata` for reliable Cargo.toml/workspace parsing.
//! Fails fast if `cargo_metadata` is unavailable.

use async_trait::async_trait;
use cargo_metadata::MetadataCommand;
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value as RreValue};
use serde_json::Value;

use crate::Result;
use crate::engines::hybrid_engine::{RuleContext, RuleEngine, RuleViolation};
use mcb_domain::ports::validation::{Severity, ViolationCategory};
use mcb_utils::constants::validate::CARGO_TOML_FILENAME;
use mcb_utils::constants::validate::{
    DEFAULT_GRL_RULE_ID, DEFAULT_RETE_MESSAGE, GRL, YAML_FIELD_RULE,
};

/// RETE Engine wrapper for rust-rule-engine library
pub struct ReteEngine {
    /// The knowledge base containing rules
    kb: KnowledgeBase,
}

impl Default for ReteEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReteEngine {
    /// Create a new RETE engine instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            kb: KnowledgeBase::new("mcb-validate"),
        }
    }

    /// Load GRL rules into the knowledge base
    ///
    /// # Errors
    ///
    /// Returns an error if GRL parsing or rule loading fails.
    pub fn load_grl(&mut self, grl_code: &str) -> Result<()> {
        let rules = GRLParser::parse_rules(grl_code)
            .map_err(|e| crate::ValidationError::Config(format!("Failed to parse GRL: {e}")))?;

        for rule in rules {
            self.kb
                .add_rule(rule)
                .map_err(|e| crate::ValidationError::Config(format!("Failed to add rule: {e}")))?;
        }

        Ok(())
    }

    /// Build facts from rule context using `cargo_metadata`
    ///
    /// IMPORTANT: All facts MUST use "Facts." prefix to match GRL syntax.
    /// GRL conditions like `Facts.has_internal_dependencies == true` require
    /// facts to be set with `facts.set("Facts.has_internal_dependencies", ...)`.
    ///
    /// Requires `cargo_metadata` to succeed. Fails fast if unavailable.
    fn build_facts(context: &RuleContext) -> Result<Facts> {
        let facts = Facts::new();

        // Load file configuration to get internal_dep_prefix
        let file_config = crate::config::FileConfig::load(&context.workspace_root);
        let internal_dep_prefix = &file_config.general.internal_dep_prefix;

        // Use cargo_metadata for reliable workspace/package parsing
        let manifest_path = context.workspace_root.join(CARGO_TOML_FILENAME);

        // Try to get metadata from cargo
        let metadata_result = MetadataCommand::new()
            .manifest_path(&manifest_path)
            .no_deps() // We only need local workspace packages
            .exec();

        match metadata_result {
            Ok(metadata) if !metadata.packages.is_empty() => {
                Self::populate_package_facts(&facts, &metadata, internal_dep_prefix);
            }
            Ok(_metadata) => {
                // Empty metadata - fail fast
                return Err(crate::ValidationError::Config(
                    "cargo_metadata returned empty packages".into(),
                ));
            }
            Err(e) => {
                return Err(crate::ValidationError::Config(format!(
                    "cargo_metadata failed: {e}"
                )));
            }
        }

        // Add file facts
        for path in context.file_contents.keys() {
            let key = format!("Facts.file_{}_exists", path.replace(['/', '.'], "_"));
            facts.set(&key, RreValue::Boolean(true));
        }

        Ok(facts)
    }

    /// Populate dependency facts from cargo metadata packages.
    fn populate_package_facts(
        facts: &Facts,
        metadata: &cargo_metadata::Metadata,
        internal_dep_prefix: &str,
    ) {
        // Note: callers guarantee `!metadata.packages.is_empty()`.
        let root_name = metadata
            .root_package()
            .map(|p| p.name.to_string())
            .or_else(|| metadata.packages.first().map(|p| p.name.to_string()))
            .unwrap_or_else(|| mcb_utils::constants::FALLBACK_UNKNOWN.to_owned());

        facts.set("Facts.crate_name", RreValue::String(root_name));

        let mut internal_deps_count = 0;
        for package in &metadata.packages {
            for dep in &package.dependencies {
                let pkg_name = package.name.to_string();
                let dep_name = dep.name.clone();
                let key = format!("Facts.crate_{pkg_name}_depends_on_{dep_name}");
                facts.set(&key, RreValue::Boolean(true));

                if !internal_dep_prefix.is_empty() && dep_name.starts_with(internal_dep_prefix) {
                    internal_deps_count += 1;
                }
            }
        }

        facts.set(
            "Facts.internal_dependencies_count",
            RreValue::Number(f64::from(internal_deps_count)),
        );
        facts.set(
            "Facts.has_internal_dependencies",
            RreValue::Boolean(internal_deps_count > 0),
        );
    }

    /// Execute GRL rules against context and return violations
    ///
    /// # Errors
    ///
    /// Returns an error if rule loading or fact building fails.
    pub async fn execute_grl(
        &mut self,
        grl_code: &str,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // Load rules into knowledge base
        self.load_grl(grl_code)?;

        // Build facts from context
        let facts = Self::build_facts(context)?;

        // Initialize violation markers in facts (rules will set these when triggered)
        // Use Facts. prefix for GRL compatibility
        facts.set("Facts.violation_triggered", RreValue::Boolean(false));
        facts.set("Facts.violation_message", RreValue::String(String::new()));
        facts.set("Facts.violation_rule_name", RreValue::String(String::new()));

        // Create engine and execute
        let mut engine = RustRuleEngine::new(self.kb.clone());
        let result = engine
            .execute(&facts)
            .map_err(|e| crate::ValidationError::Config(format!("RETE execution failed: {e}")))?;

        // Convert results to violations
        let mut violations = Vec::new();

        // If any rules fired, check if they set violation markers
        if result.rules_fired > 0
            && let Some(RreValue::Boolean(true)) = facts.get("Facts.violation_triggered")
        {
            violations.push(Self::violation_from_facts(
                &facts,
                result.rules_fired,
                result.cycle_count,
            ));
        }

        Ok(violations)
    }

    /// Build a `RuleViolation` from the violation marker facts set by fired rules.
    fn violation_from_facts(
        facts: &Facts,
        rules_fired: usize,
        cycle_count: usize,
    ) -> RuleViolation {
        let message = Self::fact_string(facts, "Facts.violation_message")
            .unwrap_or_else(|| DEFAULT_RETE_MESSAGE.to_owned());
        let rule_name = Self::fact_string(facts, "Facts.violation_rule_name")
            .unwrap_or_else(|| DEFAULT_GRL_RULE_ID.to_owned());

        RuleViolation::new(
            &rule_name,
            ViolationCategory::Architecture,
            Severity::Error,
            message,
        )
        .with_context(format!(
            "GRL Rule Engine: {rules_fired} rules fired in {cycle_count} cycles"
        ))
    }

    /// Read a string-valued fact by key, if present.
    fn fact_string(facts: &Facts, key: &str) -> Option<String> {
        match facts.get(key) {
            Some(RreValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }
}

#[async_trait]
impl RuleEngine for ReteEngine {
    async fn execute(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // Extract GRL code from rule definition
        let grl_code = rule_definition
            .get(YAML_FIELD_RULE)
            .or_else(|| rule_definition.get(GRL))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::ValidationError::Config(
                    "Missing 'rule' or 'grl' field in rule definition".into(),
                )
            })?;

        // Create mutable engine for execution
        let mut engine = Self::new();
        engine.execute_grl(grl_code, context).await
    }
}

impl Clone for ReteEngine {
    fn clone(&self) -> Self {
        Self {
            kb: KnowledgeBase::new("mcb-validate"),
        }
    }
}
