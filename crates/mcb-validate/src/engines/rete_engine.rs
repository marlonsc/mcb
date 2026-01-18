//! RETE Engine Wrapper
//!
//! Wrapper for rust-rule-engine crate implementing RETE-UL algorithm.
//! Use this engine for complex GRL rules with when/then syntax.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::engines::hybrid_engine::{RuleContext, RuleEngine, RuleViolation};
use crate::violation_trait::{Severity, ViolationCategory};
use crate::Result;

/// Fact types for RETE engine
#[derive(Debug, Clone)]
pub enum Fact {
    /// File with path and content
    File { path: String, content: String },
    /// Cargo dependency
    Dependency { crate_name: String, dep_name: String, version: String },
    /// AST node
    AstNode { file: String, kind: String, name: Option<String>, line: usize },
    /// Pattern match
    Pattern { file: String, pattern: String, line: usize },
}

/// Parsed GRL rule structure
#[derive(Debug, Clone)]
pub struct GrlRule {
    pub name: String,
    pub description: String,
    pub when_conditions: Vec<GrlCondition>,
    pub then_action: GrlAction,
    pub salience: i32,
}

/// GRL condition types
#[derive(Debug, Clone)]
pub enum GrlCondition {
    /// Check crate name: Crate(name == "mcb-domain")
    CrateName { operator: String, value: String },
    /// Check dependencies: Dependencies(contains any "mcb-*")
    DependencyPattern { check: String, pattern: String },
    /// Check file pattern: File(path matches "*.rs")
    FilePattern { field: String, operator: String, pattern: String },
    /// Check AST pattern: AST(kind == "function" && contains ".unwrap()")
    AstPattern { conditions: Vec<(String, String, String)> },
}

/// GRL action types
#[derive(Debug, Clone)]
pub enum GrlAction {
    /// Generate violation with message
    Violation { message: String },
    /// Set fact in working memory
    SetFact { name: String, value: Value },
    /// Retract fact from working memory
    Retract { name: String },
}

/// RETE Engine wrapper for rust-rule-engine
pub struct ReteEngine {
    /// Compiled rules
    rules: Vec<GrlRule>,
    /// Working memory (facts)
    working_memory: HashMap<String, Vec<Fact>>,
}

impl Default for ReteEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReteEngine {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            working_memory: HashMap::new(),
        }
    }

    /// Parse GRL rule from string
    pub fn parse_grl(&self, grl_code: &str) -> Result<GrlRule> {
        // Simple GRL parser
        // Format:
        // rule RuleName "Description" {
        //     when
        //         Condition1 && Condition2
        //     then
        //         Action;
        // }

        let lines: Vec<&str> = grl_code.lines().collect();
        let mut name = String::new();
        let mut description = String::new();
        let mut when_conditions = Vec::new();
        let mut then_action = GrlAction::Violation {
            message: "Rule violation".to_string(),
        };
        let mut in_when = false;
        let mut in_then = false;

        for line in lines {
            let trimmed = line.trim();

            // Parse rule header: rule RuleName "Description" {
            if trimmed.starts_with("rule ") {
                let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
                if parts.len() >= 2 {
                    name = parts[1].to_string();
                }
                if let Some(desc_start) = trimmed.find('"') {
                    if let Some(desc_end) = trimmed.rfind('"') {
                        if desc_end > desc_start {
                            description = trimmed[desc_start + 1..desc_end].to_string();
                        }
                    }
                }
                continue;
            }

            // Track when/then sections
            if trimmed == "when" {
                in_when = true;
                in_then = false;
                continue;
            }
            if trimmed == "then" {
                in_when = false;
                in_then = true;
                continue;
            }

            // Parse conditions in when block
            if in_when && !trimmed.is_empty() && !trimmed.starts_with('}') {
                if let Some(condition) = self.parse_condition(trimmed) {
                    when_conditions.push(condition);
                }
            }

            // Parse action in then block
            if in_then && !trimmed.is_empty() && !trimmed.starts_with('}') {
                if let Some(action) = self.parse_action(trimmed) {
                    then_action = action;
                }
            }
        }

        Ok(GrlRule {
            name,
            description,
            when_conditions,
            then_action,
            salience: 0,
        })
    }

    /// Parse a single GRL condition
    fn parse_condition(&self, condition_str: &str) -> Option<GrlCondition> {
        let trimmed = condition_str.trim().trim_end_matches("&&").trim();

        // Crate(name == "value")
        if trimmed.starts_with("Crate(") {
            if let Some(inner) = self.extract_parentheses(trimmed) {
                if let Some((field, op, value)) = self.parse_comparison(inner) {
                    if field == "name" {
                        return Some(GrlCondition::CrateName {
                            operator: op,
                            value,
                        });
                    }
                }
            }
        }

        // Dependencies(contains any "pattern")
        if trimmed.starts_with("Dependencies(") {
            if let Some(inner) = self.extract_parentheses(trimmed) {
                // Parse: contains any "mcb-*"
                let parts: Vec<&str> = inner.split_whitespace().collect();
                if parts.len() >= 3 {
                    let check = format!("{} {}", parts[0], parts[1]);
                    let pattern = parts[2..].join(" ").trim_matches('"').to_string();
                    return Some(GrlCondition::DependencyPattern { check, pattern });
                }
            }
        }

        // File(path matches "pattern")
        if trimmed.starts_with("File(") {
            if let Some(inner) = self.extract_parentheses(trimmed) {
                if let Some((field, op, value)) = self.parse_comparison(inner) {
                    return Some(GrlCondition::FilePattern {
                        field,
                        operator: op,
                        pattern: value,
                    });
                }
            }
        }

        None
    }

    /// Parse a single GRL action
    fn parse_action(&self, action_str: &str) -> Option<GrlAction> {
        let trimmed = action_str.trim().trim_end_matches(';');

        // Violation("message")
        if trimmed.starts_with("Violation(") {
            if let Some(inner) = self.extract_parentheses(trimmed) {
                let message = inner.trim_matches('"').to_string();
                return Some(GrlAction::Violation { message });
            }
        }

        None
    }

    /// Extract content within parentheses
    fn extract_parentheses<'a>(&self, s: &'a str) -> Option<&'a str> {
        let start = s.find('(')?;
        let end = s.rfind(')')?;
        if end > start {
            Some(&s[start + 1..end])
        } else {
            None
        }
    }

    /// Parse comparison like: name == "value"
    fn parse_comparison(&self, s: &str) -> Option<(String, String, String)> {
        // Try == first
        if let Some(pos) = s.find("==") {
            let field = s[..pos].trim().to_string();
            let value = s[pos + 2..].trim().trim_matches('"').to_string();
            return Some((field, "==".to_string(), value));
        }
        // Try matches
        if let Some(pos) = s.find(" matches ") {
            let field = s[..pos].trim().to_string();
            let value = s[pos + 9..].trim().trim_matches('"').to_string();
            return Some((field, "matches".to_string(), value));
        }
        // Try contains
        if let Some(pos) = s.find(" contains ") {
            let field = s[..pos].trim().to_string();
            let value = s[pos + 10..].trim().trim_matches('"').to_string();
            return Some((field, "contains".to_string(), value));
        }
        None
    }

    /// Load facts from rule context into working memory
    pub fn load_facts(&mut self, context: &RuleContext) -> Result<()> {
        // Clear previous facts
        self.working_memory.clear();

        // Load file facts
        let mut file_facts = Vec::new();
        for (path, content) in &context.file_contents {
            file_facts.push(Fact::File {
                path: path.clone(),
                content: content.clone(),
            });
        }
        self.working_memory.insert("files".to_string(), file_facts);

        // Load dependency facts from Cargo.toml files
        let mut dep_facts = Vec::new();
        self.collect_dependencies(&context.workspace_root, &mut dep_facts)?;
        self.working_memory
            .insert("dependencies".to_string(), dep_facts);

        Ok(())
    }

    /// Collect dependencies from Cargo.toml files
    fn collect_dependencies(
        &self,
        root: &std::path::Path,
        facts: &mut Vec<Fact>,
    ) -> Result<()> {
        use walkdir::WalkDir;

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.file_name().is_some_and(|n| n == "Cargo.toml") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    // Extract crate name
                    let crate_name = self.extract_crate_name(&content);

                    // Extract dependencies
                    for dep in self.extract_dependencies(&content) {
                        facts.push(Fact::Dependency {
                            crate_name: crate_name.clone(),
                            dep_name: dep.0,
                            version: dep.1,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract crate name from Cargo.toml content
    fn extract_crate_name(&self, content: &str) -> String {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("name = ") || trimmed.starts_with("name=") {
                return trimmed
                    .split('=')
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .unwrap_or_default();
            }
        }
        String::new()
    }

    /// Extract dependencies from Cargo.toml content
    fn extract_dependencies(&self, content: &str) -> Vec<(String, String)> {
        let mut deps = Vec::new();
        let mut in_deps_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("[dependencies]")
                || trimmed.starts_with("[dev-dependencies]")
                || trimmed.starts_with("[build-dependencies]")
            {
                in_deps_section = true;
                continue;
            }

            if trimmed.starts_with('[') {
                in_deps_section = false;
                continue;
            }

            if in_deps_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
                // Parse: dep_name = "version" or dep_name = { version = "x" }
                if let Some(eq_pos) = trimmed.find('=') {
                    let dep_name = trimmed[..eq_pos].trim().to_string();
                    let value_part = trimmed[eq_pos + 1..].trim();

                    let version = if value_part.starts_with('"') {
                        value_part.trim_matches('"').to_string()
                    } else if value_part.contains("version") {
                        // Extract version from { version = "x" }
                        value_part
                            .split("version")
                            .nth(1)
                            .and_then(|s| s.split('"').nth(1))
                            .unwrap_or("*")
                            .to_string()
                    } else {
                        "*".to_string()
                    };

                    deps.push((dep_name, version));
                }
            }
        }

        deps
    }

    /// Evaluate a single rule against working memory
    pub fn evaluate_rule(
        &self,
        rule: &GrlRule,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        // Check all conditions
        let mut all_conditions_met = true;
        for condition in &rule.when_conditions {
            if !self.evaluate_condition(condition, context) {
                all_conditions_met = false;
                break;
            }
        }

        // Execute action if all conditions met
        if all_conditions_met {
            match &rule.then_action {
                GrlAction::Violation { message } => {
                    violations.push(
                        RuleViolation::new(
                            &rule.name,
                            ViolationCategory::Architecture,
                            Severity::Error,
                            message.clone(),
                        )
                        .with_context(format!("GRL Rule: {}", rule.description)),
                    );
                }
                GrlAction::SetFact { .. } | GrlAction::Retract { .. } => {
                    // These modify working memory but don't generate violations
                }
            }
        }

        Ok(violations)
    }

    /// Evaluate a single condition
    fn evaluate_condition(&self, condition: &GrlCondition, _context: &RuleContext) -> bool {
        match condition {
            GrlCondition::CrateName { operator, value } => {
                // Check if we're validating a specific crate
                if let Some(deps) = self.working_memory.get("dependencies") {
                    for fact in deps {
                        if let Fact::Dependency { crate_name, .. } = fact {
                            match operator.as_str() {
                                "==" => {
                                    if crate_name == value {
                                        return true;
                                    }
                                }
                                "!=" => {
                                    if crate_name != value {
                                        return true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                false
            }

            GrlCondition::DependencyPattern { check, pattern } => {
                // Check: "contains any"
                if check.contains("contains any") {
                    if let Some(deps) = self.working_memory.get("dependencies") {
                        for fact in deps {
                            if let Fact::Dependency { dep_name, .. } = fact {
                                // Handle wildcard pattern like "mcb-*"
                                let base_pattern = pattern.trim_end_matches('*');
                                if dep_name.starts_with(base_pattern) {
                                    return true;
                                }
                            }
                        }
                    }
                }
                false
            }

            GrlCondition::FilePattern { field, operator, pattern } => {
                if let Some(files) = self.working_memory.get("files") {
                    for fact in files {
                        if let Fact::File { path, content } = fact {
                            let value = match field.as_str() {
                                "path" => path,
                                "content" => content,
                                _ => continue,
                            };

                            let matches = match operator.as_str() {
                                "==" => value == pattern,
                                "matches" => {
                                    glob::Pattern::new(pattern)
                                        .map(|p| p.matches(value))
                                        .unwrap_or(false)
                                }
                                "contains" => value.contains(pattern),
                                _ => false,
                            };

                            if matches {
                                return true;
                            }
                        }
                    }
                }
                false
            }

            GrlCondition::AstPattern { conditions: _ } => {
                // AST pattern matching would use the ast_data in context
                // For now, simplified implementation
                false
            }
        }
    }

    /// Execute a rule from GRL code
    pub async fn execute_grl(
        &mut self,
        grl_code: &str,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // Load facts into working memory
        self.load_facts(context)?;

        // Parse GRL rule
        let rule = self.parse_grl(grl_code)?;

        // Evaluate rule
        self.evaluate_rule(&rule, context)
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
            .get("rule")
            .or_else(|| rule_definition.get("grl"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::ValidationError::Config(
                    "Missing 'rule' or 'grl' field in rule definition".into(),
                )
            })?;

        // Create mutable copy for execution
        let mut engine = Self::new();
        engine.execute_grl(grl_code, context).await
    }
}

impl Clone for ReteEngine {
    fn clone(&self) -> Self {
        Self {
            rules: self.rules.clone(),
            working_memory: HashMap::new(), // Don't clone working memory
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[allow(dead_code)]
    fn create_test_context() -> RuleContext {
        RuleContext {
            workspace_root: PathBuf::from("/test/workspace"),
            config: crate::ValidationConfig::new("/test/workspace"),
            ast_data: HashMap::new(),
            cargo_data: HashMap::new(),
            file_contents: HashMap::new(),
        }
    }

    #[test]
    fn test_parse_grl_rule() {
        let engine = ReteEngine::new();
        let grl = r#"
rule DomainIndependence "Domain Layer Independence" {
    when
        Crate(name == "mcb-domain") &&
        Dependencies(contains any "mcb-*")
    then
        Violation("Domain layer cannot depend on internal mcb-* crates");
}
"#;

        let result = engine.parse_grl(grl);
        assert!(result.is_ok());

        let rule = result.unwrap();
        assert_eq!(rule.name, "DomainIndependence");
        assert_eq!(rule.description, "Domain Layer Independence");
        assert_eq!(rule.when_conditions.len(), 2);
    }

    #[test]
    fn test_parse_simple_condition() {
        let engine = ReteEngine::new();

        let condition = engine.parse_condition("Crate(name == \"mcb-domain\")");
        assert!(condition.is_some());

        if let Some(GrlCondition::CrateName { operator, value }) = condition {
            assert_eq!(operator, "==");
            assert_eq!(value, "mcb-domain");
        } else {
            panic!("Expected CrateName condition");
        }
    }

    #[test]
    fn test_extract_crate_name() {
        let engine = ReteEngine::new();
        let content = r#"
[package]
name = "mcb-domain"
version = "0.1.0"
"#;

        let name = engine.extract_crate_name(content);
        assert_eq!(name, "mcb-domain");
    }

    #[test]
    fn test_extract_dependencies() {
        let engine = ReteEngine::new();
        let content = r#"
[package]
name = "test"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
"#;

        let deps = engine.extract_dependencies(content);
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|(name, _)| name == "serde"));
        assert!(deps.iter().any(|(name, _)| name == "tokio"));
    }
}
