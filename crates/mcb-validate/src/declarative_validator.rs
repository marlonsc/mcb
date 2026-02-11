//! Declarative rule validator that executes embedded YAML rules.

use std::path::{Path, PathBuf};

use anyhow::Result;
use regex::Regex;

use crate::ValidationConfig;
use crate::config::FileConfig;
use crate::embedded_rules::EmbeddedRules;
use crate::linters::YamlRuleExecutor;
use crate::metrics::{MetricThresholds, MetricViolation, RcaAnalyzer};
use crate::rules::yaml_loader::{ValidatedRule, YamlRuleLoader};
use crate::scan::for_each_scan_rs_path;
use crate::validator_trait::Validator;
use crate::violation_trait::{Severity, Violation, ViolationCategory};

/// Executes embedded YAML declarative rules against the workspace.
///
/// Supports metrics, lint_select, regex pattern, and AST query execution slices.
pub struct DeclarativeValidator {
    /// Root directory of the workspace being validated (used by lint/regex execution slices).
    workspace_root: PathBuf,
}

impl DeclarativeValidator {
    /// Create a new declarative validator rooted at `workspace_root`.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    fn load_embedded_rules(&self) -> Result<Vec<ValidatedRule>> {
        let variables = Self::build_substitution_variables(&self.workspace_root);
        let embedded = EmbeddedRules::all_yaml();
        let mut loader = YamlRuleLoader::from_embedded_with_variables(&embedded, Some(variables))?;
        let rules = loader.load_embedded_rules()?;
        Ok(rules)
    }

    fn build_substitution_variables(workspace_root: &PathBuf) -> serde_yaml::Value {
        let file_config = FileConfig::load(workspace_root);
        let variables_val = serde_yaml::to_value(&file_config.rules.naming)
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
        let mut variables = variables_val.as_mapping().cloned().unwrap_or_default();

        let crates = [
            "domain",
            "application",
            "providers",
            "infrastructure",
            "server",
            "validate",
            "language_support",
            "ast_utils",
        ];
        for name in crates {
            let key = format!("{name}_crate");
            if let Some(val) = variables.get(serde_yaml::Value::String(key.clone()))
                && let Some(s) = val.as_str()
            {
                variables.insert(
                    serde_yaml::Value::String(format!("{name}_module")),
                    serde_yaml::Value::String(s.replace('-', "_")),
                );
            }
        }

        if let Some(domain_val) = variables.get(serde_yaml::Value::String("domain_crate".into()))
            && let Some(domain_str) = domain_val.as_str()
        {
            let prefix = if let Some(idx) = domain_str.find('-') {
                domain_str[0..idx].to_string()
            } else {
                domain_str.to_string()
            };
            variables.insert(
                serde_yaml::Value::String("project_prefix".into()),
                serde_yaml::Value::String(prefix),
            );
        }

        serde_yaml::Value::Mapping(variables)
    }

    fn collect_rs_files(&self, config: &ValidationConfig) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let _ = for_each_scan_rs_path(config, true, |path, _src_dir| {
            files.push(path.to_path_buf());
            Ok(())
        });
        files
    }

    fn validate_metrics_rules(
        &self,
        rules: &[ValidatedRule],
        files: &[PathBuf],
    ) -> Vec<Box<dyn Violation>> {
        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        let metrics_rules: Vec<&ValidatedRule> = rules
            .iter()
            .filter(|r| r.enabled && r.metrics.is_some())
            .collect();

        if metrics_rules.is_empty() {
            return violations;
        }

        for rule in &metrics_rules {
            let Some(metrics_config) = &rule.metrics else {
                continue;
            };

            let thresholds = MetricThresholds::from_metrics_config(metrics_config);
            let analyzer = RcaAnalyzer::with_thresholds(thresholds);

            for file in files {
                if let Ok(file_violations) = analyzer.find_violations(file) {
                    let typed: Vec<MetricViolation> = file_violations;
                    violations.extend(typed.into_iter().map(|v| Box::new(v) as Box<dyn Violation>));
                }
            }
        }

        violations
    }

    fn validate_lint_select_rules(
        &self,
        rules: &[ValidatedRule],
        files: &[PathBuf],
    ) -> Vec<Box<dyn Violation>> {
        let lint_rules: Vec<&ValidatedRule> = rules
            .iter()
            .filter(|r| r.enabled && !r.lint_select.is_empty())
            .collect();

        if lint_rules.is_empty() {
            return Vec::new();
        }

        let file_refs: Vec<&Path> = files.iter().map(PathBuf::as_path).collect();
        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return violations,
        };

        for rule in &lint_rules {
            match rt.block_on(YamlRuleExecutor::execute_rule(rule, &file_refs)) {
                Ok(lint_violations) => {
                    violations.extend(
                        lint_violations
                            .into_iter()
                            .map(|v| Box::new(v) as Box<dyn Violation>),
                    );
                }
                Err(_) => continue,
            }
        }

        violations
    }

    fn validate_regex_rules(
        &self,
        rules: &[ValidatedRule],
        files: &[PathBuf],
    ) -> Vec<Box<dyn Violation>> {
        let regex_rules: Vec<&ValidatedRule> = rules
            .iter()
            .filter(|r| {
                r.enabled
                    && r.lint_select.is_empty()
                    && r.metrics.is_none()
                    && r.selectors.is_empty()
                    && r.ast_query.is_none()
                    && r.config
                        .get("patterns")
                        .and_then(|v| v.as_object())
                        .is_some()
            })
            .collect();

        if regex_rules.is_empty() {
            return Vec::new();
        }

        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        for rule in &regex_rules {
            let Some(patterns_obj) = rule.config.get("patterns").and_then(|v| v.as_object()) else {
                continue;
            };

            let compiled: Vec<(&str, Regex)> = patterns_obj
                .iter()
                .filter_map(|(name, val)| {
                    val.as_str()
                        .and_then(|pat| Regex::new(pat).ok())
                        .map(|rx| (name.as_str(), rx))
                })
                .collect();

            if compiled.is_empty() {
                continue;
            }

            for file in files {
                let content = match std::fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                for (line_num, line) in content.lines().enumerate() {
                    for (_name, rx) in &compiled {
                        if rx.is_match(line) {
                            violations.push(Box::new(PatternMatchViolation {
                                rule_id: rule.id.clone(),
                                file_path: file.clone(),
                                line: line_num + 1,
                                message: rule.message.clone().unwrap_or_else(|| {
                                    format!("[{}] Pattern match: {}", rule.id, rule.description)
                                }),
                                severity: parse_severity(&rule.severity),
                                category: parse_category(&rule.category),
                            }));
                            break;
                        }
                    }
                }
            }
        }

        violations
    }

    fn validate_ast_rules(&self, rules: &[ValidatedRule]) -> Vec<Box<dyn Violation>> {
        let ast_rules: Vec<&ValidatedRule> = rules
            .iter()
            .filter(|r| r.enabled && (r.ast_query.is_some() || !r.selectors.is_empty()))
            .collect();

        if !ast_rules.is_empty() {
            eprintln!(
                "DeclarativeValidator: {} AST rules skipped (not yet implemented)",
                ast_rules.len()
            );
        }

        Vec::new()
    }
}

impl Validator for DeclarativeValidator {
    fn name(&self) -> &'static str {
        "declarative_rules"
    }

    fn description(&self) -> &'static str {
        "Executes embedded YAML declarative rules"
    }

    fn enabled_by_default(&self) -> bool {
        false
    }

    fn validate(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
        let rules = self.load_embedded_rules()?;
        let files = self.collect_rs_files(config);

        let mut violations = Vec::new();
        violations.extend(self.validate_metrics_rules(&rules, &files));
        violations.extend(self.validate_lint_select_rules(&rules, &files));
        violations.extend(self.validate_regex_rules(&rules, &files));
        violations.extend(self.validate_ast_rules(&rules));
        Ok(violations)
    }
}

#[derive(Debug)]
struct PatternMatchViolation {
    rule_id: String,
    file_path: PathBuf,
    line: usize,
    message: String,
    severity: Severity,
    category: ViolationCategory,
}

impl std::fmt::Display for PatternMatchViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.rule_id, self.message)
    }
}

impl Violation for PatternMatchViolation {
    fn id(&self) -> &str {
        &self.rule_id
    }

    fn category(&self) -> ViolationCategory {
        self.category
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn file(&self) -> Option<&PathBuf> {
        Some(&self.file_path)
    }

    fn line(&self) -> Option<usize> {
        Some(self.line)
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        _ => Severity::Info,
    }
}

fn parse_category(s: &str) -> ViolationCategory {
    match s.to_lowercase().as_str() {
        "architecture" | "clean-architecture" => ViolationCategory::Architecture,
        "quality" => ViolationCategory::Quality,
        "performance" => ViolationCategory::Performance,
        "testing" => ViolationCategory::Testing,
        "documentation" => ViolationCategory::Documentation,
        "naming" => ViolationCategory::Naming,
        "organization" => ViolationCategory::Organization,
        "solid" => ViolationCategory::Solid,
        "implementation" => ViolationCategory::Implementation,
        "refactoring" => ViolationCategory::Refactoring,
        _ => ViolationCategory::Quality,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_validator_loads_rules() {
        let validator = DeclarativeValidator::new("/nonexistent");
        let rules = validator.load_embedded_rules().unwrap();
        assert!(
            !rules.is_empty(),
            "Embedded rules should load without error"
        );
    }

    #[test]
    fn test_declarative_validator_empty_workspace() {
        let dir = std::env::temp_dir().join("mcb_decl_test_empty");
        let _ = std::fs::create_dir_all(&dir);
        let validator = DeclarativeValidator::new(&dir);
        let config = ValidationConfig::new(&dir);
        let result = validator.validate(&config).unwrap();
        assert!(
            result.is_empty(),
            "Empty workspace should produce no violations"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_lint_rules_exist_in_embedded() {
        let validator = DeclarativeValidator::new("/nonexistent");
        let rules = validator.load_embedded_rules().unwrap();
        let lint_rules: Vec<_> = rules.iter().filter(|r| !r.lint_select.is_empty()).collect();
        assert!(
            !lint_rules.is_empty(),
            "Should have at least one lint_select rule"
        );
    }

    #[test]
    fn test_ast_rules_exist_in_embedded() {
        let validator = DeclarativeValidator::new("/nonexistent");
        let rules = validator.load_embedded_rules().unwrap();
        let ast_rules: Vec<_> = rules
            .iter()
            .filter(|r| r.ast_query.is_some() || !r.selectors.is_empty())
            .collect();
        assert!(!ast_rules.is_empty(), "Should have at least one AST rule");
    }

    #[test]
    fn test_parse_severity_variants() {
        assert_eq!(parse_severity("error") as u8, Severity::Error as u8);
        assert_eq!(parse_severity("warning") as u8, Severity::Warning as u8);
        assert_eq!(parse_severity("info") as u8, Severity::Info as u8);
        assert_eq!(parse_severity("unknown") as u8, Severity::Info as u8);
    }

    #[test]
    fn test_parse_category_variants() {
        assert_eq!(
            parse_category("architecture") as u8,
            ViolationCategory::Architecture as u8
        );
        assert_eq!(
            parse_category("quality") as u8,
            ViolationCategory::Quality as u8
        );
        assert_eq!(
            parse_category("solid") as u8,
            ViolationCategory::Solid as u8
        );
        assert_eq!(
            parse_category("unknown") as u8,
            ViolationCategory::Quality as u8
        );
    }

    #[test]
    fn test_pattern_match_violation_display() {
        let v = PatternMatchViolation {
            rule_id: "TEST001".to_string(),
            file_path: PathBuf::from("test.rs"),
            line: 10,
            message: "found pattern".to_string(),
            severity: Severity::Warning,
            category: ViolationCategory::Quality,
        };
        assert_eq!(v.to_string(), "[TEST001] found pattern");
        assert_eq!(v.id(), "TEST001");
        assert_eq!(v.line(), Some(10));
        assert!(v.file().is_some());
    }
}
