//! Rust Rule Engine Wrapper
//!
//! Wrapper for rust-rule-engine crate with RETE-UL algorithm
//! and GRL (Grule Rule Language) support.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::violation_trait::{ViolationCategory, Severity};
use crate::engines::hybrid_engine::RuleViolation;
use crate::Result;

use super::hybrid_engine::{RuleContext, RuleEngine};

/// Wrapper for rust-rule-engine
pub struct RustRuleEngineWrapper {
    // In a real implementation, this would hold the actual rust-rule-engine instance
    compiled_rules: HashMap<String, String>,
}

impl Default for RustRuleEngineWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl RustRuleEngineWrapper {
    pub fn new() -> Self {
        Self {
            compiled_rules: HashMap::new(),
        }
    }

    /// Compile GRL rule for execution
    pub fn compile_grl_rule(&mut self, rule_id: String, grl_code: &str) -> Result<()> {
        // In real implementation, this would compile the GRL code
        // For now, just store it
        self.compiled_rules.insert(rule_id, grl_code.to_string());
        Ok(())
    }

    /// Execute compiled GRL rule
    pub fn execute_compiled_rule(
        &self,
        rule_id: &str,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let grl_code = self.compiled_rules.get(rule_id)
            .ok_or_else(|| crate::ValidationError::Config(
                format!("Compiled rule not found: {}", rule_id)
            ))?;

        // Parse and execute GRL rules
        self.execute_grl_code(grl_code, context)
    }

    /// Execute GRL code directly
    fn execute_grl_code(&self, grl_code: &str, context: &RuleContext) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        // Parse GRL rules and execute them
        // This is a simplified implementation - real rust-rule-engine would handle this

        if grl_code.contains("DomainIndependence") {
            // Check domain layer independence
            violations.extend(self.check_domain_independence(context)?);
        }

        if grl_code.contains("NoUnwrap") {
            // Check for unwrap usage
            violations.extend(self.check_unwrap_usage(context)?);
        }

        if grl_code.contains("NoExpect") {
            // Check for expect usage
            violations.extend(self.check_expect_usage(context)?);
        }

        Ok(violations)
    }

    fn check_domain_independence(&self, context: &RuleContext) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        // Check Cargo.toml for domain crate
        let cargo_toml_path = context.workspace_root.join("crates/mcb-domain/Cargo.toml");
        if cargo_toml_path.exists() {
            let content = std::fs::read_to_string(&cargo_toml_path)?;
            if content.contains("mcb-") {
                violations.push(RuleViolation::new(
                    "CA001",
                    ViolationCategory::Architecture,
                    Severity::Error,
                    "Domain layer cannot depend on internal mcb-* crates"
                ).with_file(cargo_toml_path)
                 .with_context("Found forbidden dependency in Cargo.toml"));
            }
        }

        Ok(violations)
    }

    fn check_unwrap_usage(&self, context: &RuleContext) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        // Scan Rust files for unwrap usage
        self.scan_files_for_pattern(
            context,
            "**/*.rs",
            vec!["**/tests/**", "**/*test.rs"],
            ".unwrap()",
            "QUAL001",
            "Avoid .unwrap() in production code",
            &mut violations,
        )?;

        Ok(violations)
    }

    fn check_expect_usage(&self, context: &RuleContext) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        // Scan Rust files for expect usage
        self.scan_files_for_pattern(
            context,
            "**/*.rs",
            vec!["**/tests/**", "**/*test.rs"],
            ".expect(",
            "QUAL001",
            "Avoid .expect() in production code, use ? with context",
            &mut violations,
        )?;

        Ok(violations)
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_files_for_pattern(
        &self,
        context: &RuleContext,
        include_pattern: &str,
        exclude_patterns: Vec<&str>,
        search_pattern: &str,
        rule_id: &str,
        message: &str,
        violations: &mut Vec<RuleViolation>,
    ) -> Result<()> {
        use walkdir::WalkDir;
        use glob::Pattern;

        let include_glob = Pattern::new(include_pattern)
            .map_err(|e| crate::ValidationError::Config(format!("Invalid pattern: {}", e)))?;
        let exclude_globs: Vec<Pattern> = exclude_patterns
            .iter()
            .map(|p| Pattern::new(p))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| crate::ValidationError::Config(format!("Invalid exclude pattern: {}", e)))?;

        for entry in WalkDir::new(&context.workspace_root) {
            let entry = entry.map_err(|e| crate::ValidationError::Io(e.into()))?;
            let path = entry.path();

            if !include_glob.matches_path(path) {
                continue;
            }

            // Check exclusions
            let should_exclude = exclude_globs.iter().any(|glob| glob.matches_path(path));
            if should_exclude {
                continue;
            }

                    if let Ok(content) = std::fs::read_to_string(path) {
                        if content.contains(search_pattern) {
                            violations.push(RuleViolation::new(
                                rule_id,
                                ViolationCategory::Quality,
                                Severity::Error,
                                message
                            ).with_file(path.to_path_buf())
                             .with_context(format!("Found pattern: {}", search_pattern)));
                        }
                    }
        }

        Ok(())
    }
}

#[async_trait]
impl RuleEngine for RustRuleEngineWrapper {
    async fn execute(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // Extract GRL code from rule definition
        if let Some(grl_code) = rule_definition.get("grl").and_then(|v| v.as_str()) {
            self.execute_grl_code(grl_code, context)
        } else {
            // Try to extract from rule string
            let rule_str = serde_json::to_string(rule_definition)
                .map_err(|e| crate::ValidationError::Config(format!("JSON serialization error: {}", e)))?;
            self.execute_grl_code(&rule_str, context)
        }
    }
}

impl Clone for RustRuleEngineWrapper {
    fn clone(&self) -> Self {
        Self {
            compiled_rules: self.compiled_rules.clone(),
        }
    }
}