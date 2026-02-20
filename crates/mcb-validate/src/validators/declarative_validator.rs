//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Declarative rule validator that executes embedded YAML rules.

use std::path::{Path, PathBuf};

use regex::Regex;
use tracing::warn;

use crate::Result;
use crate::ValidationConfig;
use crate::config::FileConfig;
use crate::embedded_rules::EmbeddedRules;
use crate::filters::LanguageId;
use crate::filters::dependency_parser::WorkspaceDependencies;
use crate::filters::rule_filters::RuleFilterExecutor;
use crate::linters::YamlRuleExecutor;
use crate::metrics::{MetricThresholds, MetricViolation, RcaAnalyzer};
use crate::pattern_registry::compile_regex;
use crate::rules::yaml_loader::{ValidatedRule, YamlRuleLoader};
use crate::scan::for_each_scan_file;
use crate::traits::validator::Validator;
use crate::traits::violation::Violation;
use crate::validators::declarative_support::{
    PatternMatchViolation, build_substitution_variables, parse_category, parse_severity,
    validate_path_rules,
};

/// Executes embedded YAML declarative rules against the workspace.
///
/// Supports metrics, `lint_select`, regex pattern, and AST query execution slices.
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
        let variables = build_substitution_variables(&self.workspace_root);
        let file_config = FileConfig::load(&self.workspace_root);
        let rules_path = self.workspace_root.join(&file_config.general.rules_path);

        let mut loader = YamlRuleLoader::with_variables(rules_path, Some(variables))?;
        loader.set_embedded_rules(EmbeddedRules::all_yaml());
        loader.load_all_rules_sync()
    }

    fn collect_files(config: &ValidationConfig, language: Option<LanguageId>) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Err(e) = for_each_scan_file(config, language, true, |entry, _src_dir| {
            files.push(entry.absolute_path.clone());
            Ok(())
        }) {
            warn!(error = ?e, "Failed to scan workspace for files");
        }
        files
    }

    fn validate_metrics_rules(
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
                            error = ?e,
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
        let run_on_dedicated = |fut: std::pin::Pin<
            Box<
                dyn std::future::Future<Output = crate::Result<Vec<crate::linters::LintViolation>>>
                    + Send,
            >,
        >| {
            std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        let runtime = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .map_err(|error| format!("failed to build lint runtime: {error}"))?;
                        runtime
                            .block_on(fut)
                            .map_err(|error| format!("lint execution failed: {error}"))
                    })
                    .join()
                    .map_err(|_| "lint thread panicked".to_owned())
                    .and_then(std::convert::identity)
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
                Err(error) => {
                    warn!(
                        rule_id = %rule.id,
                        error = %error,
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
        let regex_rules: Vec<&ValidatedRule> =
            rules.iter().filter(|r| Self::is_regex_rule(r)).collect();

        if regex_rules.is_empty() {
            return Vec::new();
        }

        let filter_executor = RuleFilterExecutor::new(self.workspace_root.clone());
        let workspace_deps = match filter_executor.parse_workspace_dependencies() {
            Ok(deps) => deps,
            Err(e) => {
                warn!(error = ?e, "Failed to parse workspace dependencies for regex rules");
                return Vec::new();
            }
        };

        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        for rule in &regex_rules {
            let compiled = Self::compile_rule_patterns(rule);
            if compiled.is_empty() {
                continue;
            }

            let ignore_compiled = Self::compile_ignore_patterns(rule);

            for file in files {
                if !Self::should_execute_on_file(&filter_executor, &workspace_deps, rule, file) {
                    continue;
                }

                violations.extend(Self::collect_regex_violations_for_file(
                    rule,
                    file,
                    &compiled,
                    &ignore_compiled,
                ));
            }
        }

        violations
    }

    fn is_regex_rule(rule: &ValidatedRule) -> bool {
        let uses_regex_engine = rule.lint_select.is_empty()
            && rule.metrics.is_none()
            && rule.selectors.is_empty()
            && rule.ast_query.is_none();
        rule.enabled && uses_regex_engine && Self::has_rule_patterns(rule)
    }

    fn has_rule_patterns(rule: &ValidatedRule) -> bool {
        rule.config
            .get("patterns")
            .and_then(|v| v.as_object())
            .is_some()
    }

    fn compile_rule_patterns(rule: &ValidatedRule) -> Vec<Regex> {
        let Some(patterns_obj) = rule.config.get("patterns").and_then(|v| v.as_object()) else {
            return Vec::new();
        };

        patterns_obj
            .iter()
            .filter_map(|(name, val)| {
                let pat = val.as_str()?;
                match compile_regex(pat) {
                    Ok(rx) => Some(rx),
                    Err(e) => {
                        warn!(
                            rule_id = %rule.id,
                            pattern_name = %name,
                            error = ?e,
                            "Malformed regex pattern in rule"
                        );
                        None
                    }
                }
            })
            .collect()
    }

    fn compile_ignore_patterns(rule: &ValidatedRule) -> Vec<Regex> {
        let Some(ignore_arr) = rule
            .config
            .get("ignore_patterns")
            .and_then(|v| v.as_array())
        else {
            return Vec::new();
        };

        ignore_arr
            .iter()
            .filter_map(|v| {
                let pat = v.as_str()?;
                match compile_regex(pat) {
                    Ok(rx) => Some(rx),
                    Err(_) => {
                        warn!(rule_id = %rule.id, "Invalid ignore pattern regex");
                        None
                    }
                }
            })
            .collect()
    }

    fn should_execute_on_file(
        filter_executor: &RuleFilterExecutor,
        workspace_deps: &WorkspaceDependencies,
        rule: &ValidatedRule,
        file: &Path,
    ) -> bool {
        let Some(filters) = &rule.filters else {
            return true;
        };

        filter_executor
            .should_execute_rule(filters, file, None, workspace_deps)
            .unwrap_or(false)
    }

    fn collect_regex_violations_for_file(
        rule: &ValidatedRule,
        file: &Path,
        compiled: &[Regex],
        ignore_compiled: &[Regex],
    ) -> Vec<Box<dyn Violation>> {
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    file = %file.display(),
                    error = ?e,
                    "Failed to read file for regex validation"
                );
                return Vec::new();
            }
        };

        content
            .lines()
            .enumerate()
            .filter_map(|(line_num, line)| {
                if ignore_compiled.iter().any(|irx| irx.is_match(line)) {
                    return None;
                }
                if !compiled.iter().any(|rx| rx.is_match(line)) {
                    return None;
                }

                Some(Self::build_pattern_match_violation(
                    rule,
                    file.to_path_buf(),
                    line_num + 1,
                ))
            })
            .collect()
    }

    fn build_pattern_match_violation(
        rule: &ValidatedRule,
        file_path: PathBuf,
        line: usize,
    ) -> Box<dyn Violation> {
        Box::new(PatternMatchViolation {
            rule_id: rule.id.clone(),
            file_path,
            line,
            message: rule
                .message
                .clone()
                .unwrap_or_else(|| format!("[{}] Pattern match: {}", rule.id, rule.description)),
            severity: parse_severity(&rule.severity),
            category: parse_category(&rule.category),
        })
    }

    fn validate_ast_rules(rules: &[ValidatedRule]) -> Vec<Box<dyn Violation>> {
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
        let files = Self::collect_files(config, Some(LanguageId::Rust));

        let mut violations = Vec::new();
        violations.extend(Self::validate_metrics_rules(&rules, &files));
        violations.extend(Self::validate_lint_select_rules(&rules, &files));
        violations.extend(self.validate_regex_rules(&rules, &files));
        violations.extend(validate_path_rules(&self.workspace_root, &rules, &files));
        violations.extend(Self::validate_ast_rules(&rules));
        Ok(violations)
    }
}
