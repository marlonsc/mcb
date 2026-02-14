//! Declarative rule validator that executes embedded YAML rules.

use std::path::{Path, PathBuf};

use anyhow::Result;
use derive_more::Display;
use regex::Regex;
use tracing::warn;

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
        if let Err(e) = for_each_scan_rs_path(config, true, |path, _src_dir| {
            files.push(path.to_path_buf());
            Ok(())
        }) {
            warn!(error = %e, "Failed to scan workspace for Rust files");
        }
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
                match analyzer.find_violations(file) {
                    Ok(file_violations) => {
                        let typed: Vec<MetricViolation> = file_violations;
                        violations
                            .extend(typed.into_iter().map(|v| Box::new(v) as Box<dyn Violation>));
                    }
                    Err(e) => {
                        warn!(
                            rule_id = %rule.id,
                            file = %file.display(),
                            error = %e,
                            "Metrics analysis failed"
                        );
                    }
                }
            }
        }

        violations
    }

    // Rust-only: non-Rust lint selectors (e.g. Ruff) receive no matching files.
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

        let block_on: Box<dyn Fn(_) -> _> =
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                Box::new(move |fut| handle.block_on(fut))
            } else {
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => Box::new(move |fut| rt.block_on(fut)),
                    Err(e) => {
                        warn!(error = %e, "Failed to create Tokio runtime for lint execution");
                        return violations;
                    }
                }
            };

        for rule in &lint_rules {
            match block_on(YamlRuleExecutor::execute_rule(rule, &file_refs)) {
                Ok(lint_violations) => {
                    violations.extend(lint_violations.into_iter().map(|mut v| {
                        v.ensure_file_path();
                        Box::new(v) as Box<dyn Violation>
                    }));
                }
                Err(e) => {
                    warn!(
                        rule_id = %rule.id,
                        error = %e,
                        "Lint rule execution failed"
                    );
                }
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

            let mut compiled: Vec<(&str, Regex)> = Vec::new();
            for (name, val) in patterns_obj {
                if let Some(pat) = val.as_str() {
                    match Regex::new(pat) {
                        Ok(rx) => compiled.push((name.as_str(), rx)),
                        Err(e) => {
                            warn!(
                                rule_id = %rule.id,
                                pattern_name = %name,
                                error = %e,
                                "Malformed regex pattern in rule"
                            );
                        }
                    }
                }
            }

            if compiled.is_empty() {
                continue;
            }

            for file in files {
                let content = match std::fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!(
                            file = %file.display(),
                            error = %e,
                            "Failed to read file for regex validation"
                        );
                        continue;
                    }
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
            warn!(
                count = ast_rules.len(),
                "DeclarativeValidator: AST rules skipped (not yet implemented)"
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

#[derive(Debug, Display)]
#[display("[{rule_id}] {message}")]
struct PatternMatchViolation {
    rule_id: String,
    file_path: PathBuf,
    line: usize,
    message: String,
    severity: Severity,
    category: ViolationCategory,
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
