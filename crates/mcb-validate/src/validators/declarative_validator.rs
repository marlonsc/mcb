//! Declarative rule validator that executes embedded YAML rules.

use std::path::{Path, PathBuf};

use derive_more::Display;
use regex::Regex;
use tracing::warn;

use crate::Result;
use crate::ValidationConfig;
use crate::config::FileConfig;
use crate::embedded_rules::EmbeddedRules;
use crate::filters::LanguageId;
use crate::linters::YamlRuleExecutor;
use crate::metrics::{MetricThresholds, MetricViolation, RcaAnalyzer};
use crate::rules::yaml_loader::{ValidatedRule, YamlRuleLoader};
use crate::scan::for_each_scan_file;
use crate::traits::validator::Validator;
use crate::traits::violation::{Severity, Violation, ViolationCategory};

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

    fn load_rules(&self) -> Result<Vec<ValidatedRule>> {
        let variables = Self::build_substitution_variables(&self.workspace_root);
        let file_config = FileConfig::load(&self.workspace_root);
        let rules_path = self.workspace_root.join(&file_config.general.rules_path);

        let mut loader = YamlRuleLoader::with_variables(rules_path, Some(variables))?;
        loader.set_embedded_rules(EmbeddedRules::all_yaml());
        loader.load_all_rules_sync()
    }

    fn build_substitution_variables(workspace_root: &PathBuf) -> serde_yaml::Value {
        let file_config = FileConfig::load(workspace_root);
        let variables_val = serde_yaml::to_value(&file_config.rules.naming)
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
        let mut variables = variables_val.as_mapping().cloned().unwrap_or_default();

        // Inject Clean Architecture paths
        let ca_val = serde_yaml::to_value(&file_config.rules.clean_architecture)
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
        if let Some(ca_map) = ca_val.as_mapping() {
            for (k, v) in ca_map {
                variables.insert(k.clone(), v.clone());
            }
        }

        let crates = [
            "domain",
            "application",
            "providers",
            "infrastructure",
            "server",
            "validate",
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

    fn collect_files(
        &self,
        config: &ValidationConfig,
        language: Option<LanguageId>,
    ) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Err(e) = for_each_scan_file(config, language, true, |entry, _src_dir| {
            files.push(entry.absolute_path.to_path_buf());
            Ok(())
        }) {
            warn!(error = %e, "Failed to scan workspace for files");
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

        // Spawn a dedicated thread with its own runtime so we never
        // call `block_on` or `block_in_place` inside the caller's
        // async context (which panics on both single- and multi-threaded
        // tokio runtimes).
        let run_on_dedicated =
            |fut: std::pin::Pin<Box<dyn std::future::Future<Output = _> + Send>>| -> _ {
                std::thread::scope(|s| {
                    s.spawn(|| {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .expect("failed to build lint runtime");
                        rt.block_on(fut)
                    })
                    .join()
                    .expect("lint thread panicked")
                })
            };

        for rule in &lint_rules {
            match run_on_dedicated(Box::pin(YamlRuleExecutor::execute_rule(rule, &file_refs))) {
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

        let filter_executor =
            crate::filters::rule_filters::RuleFilterExecutor::new(self.workspace_root.clone());
        let workspace_deps = match filter_executor.parse_workspace_dependencies() {
            Ok(deps) => deps,
            Err(e) => {
                warn!(error = %e, "Failed to parse workspace dependencies for regex rules");
                return Vec::new();
            }
        };

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

            let mut ignore_compiled: Vec<Regex> = Vec::new();
            if let Some(ignore_arr) = rule
                .config
                .get("ignore_patterns")
                .and_then(|v| v.as_array())
            {
                for v in ignore_arr {
                    if let Some(pat) = v.as_str() {
                        if let Ok(rx) = Regex::new(pat) {
                            ignore_compiled.push(rx);
                        } else {
                            warn!(rule_id = %rule.id, "Invalid ignore pattern regex");
                        }
                    }
                }
            }

            for file in files {
                // Check filters
                if let Some(filters) = &rule.filters {
                    let res = filter_executor
                        .should_execute_rule(filters, file, None, &workspace_deps)
                        .unwrap_or(false);
                    if !res {
                        continue;
                    }
                }

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
                    // Check ignore patterns
                    if ignore_compiled.iter().any(|irx| irx.is_match(line)) {
                        continue;
                    }

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
        true
    }

    fn validate(&self, config: &ValidationConfig) -> crate::Result<Vec<Box<dyn Violation>>> {
        let rules = self.load_rules()?;
        let files = self.collect_files(config, Some(LanguageId::Rust));

        let mut violations = Vec::new();
        violations.extend(self.validate_metrics_rules(&rules, &files));
        violations.extend(self.validate_lint_select_rules(&rules, &files));
        violations.extend(self.validate_regex_rules(&rules, &files));
        violations.extend(self.validate_path_rules(&rules, &files));
        violations.extend(self.validate_ast_rules(&rules));
        Ok(violations)
    }
}

impl DeclarativeValidator {
    /// Validates path-based rules that check file location (not content).
    ///
    /// Rules with `engine: path` define filters that identify files in wrong locations.
    /// If a file matches the rule's filters, that IS the violation (the file exists
    /// in a disallowed location).
    fn validate_path_rules(
        &self,
        rules: &[ValidatedRule],
        files: &[PathBuf],
    ) -> Vec<Box<dyn Violation>> {
        let path_rules: Vec<&ValidatedRule> = rules
            .iter()
            .filter(|r| r.enabled && r.engine == "path")
            .collect();

        if path_rules.is_empty() {
            return Vec::new();
        }

        let filter_executor =
            crate::filters::rule_filters::RuleFilterExecutor::new(self.workspace_root.clone());
        let workspace_deps = match filter_executor.parse_workspace_dependencies() {
            Ok(deps) => deps,
            Err(e) => {
                warn!(error = %e, "Failed to parse workspace dependencies for path rules");
                return Vec::new();
            }
        };

        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        for rule in &path_rules {
            for file in files {
                if let Some(filters) = &rule.filters {
                    let should_exec = filter_executor
                        .should_execute_rule(filters, file, None, &workspace_deps)
                        .unwrap_or(false);

                    if should_exec {
                        violations.push(Box::new(PatternMatchViolation {
                            rule_id: rule.id.clone(),
                            file_path: file.clone(),
                            line: 0,
                            message: rule.message.clone().unwrap_or_else(|| {
                                format!(
                                    "[{}] File placement violation: {}",
                                    rule.id, rule.description
                                )
                            }),
                            severity: parse_severity(&rule.severity),
                            category: parse_category(&rule.category),
                        }));
                    }
                }
            }
        }
        violations
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
