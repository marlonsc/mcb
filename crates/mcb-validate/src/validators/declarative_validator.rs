//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Declarative rule validator that executes embedded YAML rules.

use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use regex::Regex;

use crate::Result;
use crate::ValidationConfig;
use crate::ast::{AstSelectorEngine, TreeSitterQueryExecutor};
use crate::config::FileConfig;
use crate::embedded_rules::EmbeddedRules;
use crate::filters::LanguageId;
use crate::filters::dependency_parser::WorkspaceDependencies;
use crate::filters::rule_filters::RuleFilterExecutor;
use crate::linters::YamlRuleExecutor;
use crate::metrics::{MetricThresholds, MetricViolation, RcaAnalyzer};
use crate::rules::yaml_loader::{ValidatedRule, YamlRuleLoader};
use crate::scan::for_each_scan_file;
use crate::validators::declarative_support::{
    PatternMatchViolation, build_substitution_variables, parse_category, parse_severity,
    validate_path_rules,
};
use mcb_domain::ports::validation::Validator;
use mcb_domain::ports::validation::Violation;
use mcb_utils::utils::regex::compile_regex;

/// Run `f` with the optional `ValidationRunContext` set on the current thread.
fn with_ctx<T>(
    ctx: &Option<std::sync::Arc<crate::run_context::ValidationRunContext>>,
    f: impl FnOnce() -> T,
) -> T {
    match ctx {
        Some(c) => crate::run_context::ValidationRunContext::with_active(c, f),
        None => f(),
    }
}

struct CompiledRegexRule<'a> {
    rule: &'a ValidatedRule,
    compiled: Vec<Regex>,
    ignore_compiled: Vec<Regex>,
}

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
        if !self
            .workspace_root
            .join(&file_config.general.rules_path)
            .exists()
        {
            loader.set_embedded_rules(EmbeddedRules::all_yaml());
        }

        let mut unique = HashSet::new();
        let mut rules = Vec::new();
        for rule in loader.load_all_rules_sync()? {
            if unique.insert(rule.id.clone()) {
                rules.push(rule);
            }
        }
        Ok(rules)
    }

    fn collect_files(config: &ValidationConfig, language: Option<LanguageId>) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Err(e) = for_each_scan_file(config, language, true, |entry, _src_dir| {
            files.push(entry.absolute_path.clone());
            Ok(())
        }) {
            mcb_domain::warn!(
                "validate",
                "Failed to scan workspace for files",
                &e.to_string()
            );
        }
        files
    }

    fn validate_metrics_rules(
        rules: &[ValidatedRule],
        files: &[PathBuf],
    ) -> Vec<Box<dyn Violation>> {
        let metrics_rules: Vec<(&ValidatedRule, MetricThresholds)> = rules
            .iter()
            .filter(|r| r.enabled && r.metrics.is_some())
            .filter_map(|rule| {
                rule.metrics
                    .as_ref()
                    .map(|cfg| (rule, MetricThresholds::from_metrics_config(cfg)))
            })
            .collect();

        if metrics_rules.is_empty() {
            return Vec::new();
        }

        let analyzer = RcaAnalyzer::new();
        let per_file: Vec<Vec<Box<dyn Violation>>> = files
            .par_iter()
            .map(|file| match analyzer.analyze_file(file) {
                Ok(functions) => {
                    let mut local: Vec<Box<dyn Violation>> = Vec::new();
                    for (rule, thresholds) in &metrics_rules {
                        mcb_domain::trace!(
                            "declarative",
                            "Metrics check",
                            &format!("rule={} file={}", rule.id, file.display())
                        );
                        let rule_violations: Vec<MetricViolation> =
                            RcaAnalyzer::find_violations_in_functions(file, &functions, thresholds);
                        local.extend(
                            rule_violations
                                .into_iter()
                                .map(|v| Box::new(v) as Box<dyn Violation>),
                        );
                    }
                    local
                }
                Err(e) => {
                    mcb_domain::warn!(
                        "validate",
                        "Metrics analysis failed",
                        &format!("file = {}, error = {:?}", file.display(), e)
                    );
                    Vec::new()
                }
            })
            .collect();

        per_file.into_iter().flatten().collect()
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

        let has_python_files = files.iter().any(|file| {
            file.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    matches!(ext.to_ascii_lowercase().as_str(), "py" | "pyi" | "pyw")
                })
        });
        let has_ruff = Self::command_is_available("ruff");
        let has_cargo = Self::command_is_available("cargo");

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
            let needs_clippy = rule
                .lint_select
                .iter()
                .any(|code| code.starts_with("clippy::"));
            let needs_ruff = rule
                .lint_select
                .iter()
                .any(|code| !code.starts_with("clippy::"));

            if (needs_clippy && !has_cargo) || (needs_ruff && (!has_ruff || !has_python_files)) {
                mcb_domain::trace!(
                    "declarative",
                    "Skipping lint-select rule",
                    &format!(
                        "rule={} needs_clippy={} needs_ruff={} has_cargo={} has_ruff={} has_python_files={}",
                        rule.id, needs_clippy, needs_ruff, has_cargo, has_ruff, has_python_files
                    )
                );
                continue;
            }

            if needs_clippy
                && rule
                    .lint_select
                    .iter()
                    .all(|code| code == "clippy::unwrap_used")
            {
                mcb_domain::trace!(
                    "declarative",
                    "Skipping clippy unwrap rule",
                    &format!("rule={}", rule.id)
                );
                continue;
            }

            mcb_domain::trace!(
                "declarative",
                "Running lint-select rule",
                &format!("rule={}", rule.id)
            );
            match run_on_dedicated(Box::pin(YamlRuleExecutor::execute_rule(rule, &file_refs))) {
                Ok(lint_violations) => {
                    mcb_domain::trace!(
                        "declarative",
                        "Lint rule done",
                        &format!("rule={} violations={}", rule.id, lint_violations.len())
                    );
                    violations.extend(lint_violations.into_iter().map(|mut v| {
                        v.ensure_file_path();
                        Box::new(v) as Box<dyn Violation>
                    }));
                }
                Err(error) => {
                    mcb_domain::warn!(
                        "validate",
                        "Lint rule execution failed",
                        &format!("rule_id = {}, error = {}", rule.id, error)
                    );
                }
            }
        }

        violations
    }

    fn command_is_available(command: &str) -> bool {
        std::process::Command::new(command)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    fn validate_regex_rules(
        rules: &[ValidatedRule],
        files: &[PathBuf],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: &WorkspaceDependencies,
    ) -> Vec<Box<dyn Violation>> {
        let regex_rules: Vec<CompiledRegexRule<'_>> = rules
            .iter()
            .filter(|r| Self::is_regex_rule(r))
            .filter_map(|rule| {
                let compiled = Self::compile_rule_patterns(rule);
                if compiled.is_empty() {
                    return None;
                }

                Some(CompiledRegexRule {
                    rule,
                    compiled,
                    ignore_compiled: Self::compile_ignore_patterns(rule),
                })
            })
            .collect();

        if regex_rules.is_empty() {
            return Vec::new();
        }

        let per_file: Vec<Vec<Box<dyn Violation>>> = files
            .par_iter()
            .map(|file| {
                let cached = crate::run_context::ValidationRunContext::active()
                    .and_then(|ctx| ctx.read_cached(file).ok());
                let content = if let Some(cached) = cached.as_ref() {
                    Cow::Borrowed(cached.as_ref())
                } else {
                    match std::fs::read_to_string(file) {
                        Ok(c) => Cow::Owned(c),
                        Err(e) => {
                            mcb_domain::warn!(
                                "validate",
                                "Failed to read file for regex validation",
                                &format!("file = {}, error = {:?}", file.display(), e)
                            );
                            return Vec::new();
                        }
                    }
                };

                let mut file_violations: Vec<Box<dyn Violation>> = Vec::new();
                for rule in &regex_rules {
                    if !Self::should_execute_on_file(
                        filter_executor,
                        workspace_deps,
                        rule.rule,
                        file,
                        Some(content.as_ref()),
                    ) {
                        continue;
                    }

                    file_violations.extend(Self::collect_regex_violations_for_content(
                        rule.rule,
                        file,
                        content.as_ref(),
                        &rule.compiled,
                        &rule.ignore_compiled,
                    ));
                }
                file_violations
            })
            .collect();

        per_file.into_iter().flatten().collect()
    }

    fn validate_ast_selector_rules(
        rules: &[ValidatedRule],
        files: &[PathBuf],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: &WorkspaceDependencies,
    ) -> mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>> {
        let ast_rules: Vec<&ValidatedRule> = rules
            .iter()
            .filter(|r| r.enabled && (!r.selectors.is_empty() || r.ast_query.is_some()))
            .collect();

        if ast_rules.is_empty() {
            return Ok(Vec::new());
        }

        let file_results: Vec<
            mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>>,
        > = files
            .par_iter()
            .map(|file| {
                let cached = crate::run_context::ValidationRunContext::active()
                    .and_then(|ctx| ctx.read_cached(file).ok());
                let source = if let Some(cached) = cached.as_ref() {
                    Cow::Borrowed(cached.as_ref())
                } else {
                    match std::fs::read_to_string(file) {
                        Ok(content) => Cow::Owned(content),
                        Err(e) => {
                            mcb_domain::warn!(
                                "validate",
                                "Failed to read file for AST validation",
                                &format!("file = {}, error = {:?}", file.display(), e)
                            );
                            return Ok(Vec::new());
                        }
                    }
                };

                let mut local: Vec<Box<dyn Violation>> = Vec::new();
                for rule in &ast_rules {
                    if !Self::should_execute_on_file(
                        filter_executor,
                        workspace_deps,
                        rule,
                        file,
                        Some(source.as_ref()),
                    ) {
                        continue;
                    }

                    let selector_matches =
                        AstSelectorEngine::execute_on_source(rule, file, source.as_ref());
                    local.extend(selector_matches.into_iter().map(|matched| {
                        Self::build_pattern_match_violation(rule, matched.file_path, matched.line)
                    }));

                    let query_matches =
                        TreeSitterQueryExecutor::execute_on_source(rule, file, source.as_bytes())?;
                    local.extend(query_matches.into_iter().map(|matched| {
                        Self::build_pattern_match_violation(rule, matched.file_path, matched.line)
                    }));
                }

                Ok(local)
            })
            .collect();

        let mut violations: Vec<Box<dyn Violation>> = Vec::new();
        for file_result in file_results {
            violations.extend(file_result?);
        }

        Ok(violations)
    }

    fn is_regex_rule(rule: &ValidatedRule) -> bool {
        let uses_regex_engine = rule.lint_select.is_empty()
            && rule.metrics.is_none()
            && rule.selectors.is_empty()
            && rule.ast_query.is_none();
        rule.enabled && uses_regex_engine && Self::has_rule_patterns(rule)
    }

    fn has_rule_patterns(rule: &ValidatedRule) -> bool {
        Self::effective_regex_config(rule)
            .get("patterns")
            .and_then(|v| v.as_object())
            .is_some()
    }

    fn effective_regex_config(rule: &ValidatedRule) -> &serde_json::Value {
        let top_level_has_patterns = rule
            .config
            .get("patterns")
            .and_then(|v| v.as_object())
            .is_some();
        let top_level_has_ignore = rule
            .config
            .get("ignore_patterns")
            .and_then(|v| v.as_array())
            .is_some();

        if top_level_has_patterns || top_level_has_ignore {
            return &rule.config;
        }

        rule.rule_definition.get("config").unwrap_or(&rule.config)
    }

    fn compile_rule_patterns(rule: &ValidatedRule) -> Vec<Regex> {
        let Some(patterns_obj) = Self::effective_regex_config(rule)
            .get("patterns")
            .and_then(|v| v.as_object())
        else {
            return Vec::new();
        };

        patterns_obj
            .iter()
            .filter_map(|(name, val)| {
                let pat = val.as_str()?;
                match compile_regex(pat) {
                    Ok(rx) => Some(rx),
                    Err(e) => {
                        mcb_domain::warn!(
                            "validate",
                            "Malformed regex pattern in rule",
                            &format!(
                                "rule_id = {}, pattern_name = {}, error = {:?}",
                                rule.id, name, e
                            )
                        );
                        None
                    }
                }
            })
            .collect()
    }

    fn compile_ignore_patterns(rule: &ValidatedRule) -> Vec<Regex> {
        let Some(ignore_arr) = Self::effective_regex_config(rule)
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
                        mcb_domain::warn!("validate", "Invalid ignore pattern regex", &rule.id);
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
        file_content: Option<&str>,
    ) -> bool {
        let Some(filters) = &rule.filters else {
            return true;
        };

        filter_executor
            .should_execute_rule(filters, file, file_content, workspace_deps)
            .unwrap_or(false)
    }

    fn collect_regex_violations_for_content(
        rule: &ValidatedRule,
        file: &Path,
        content: &str,
        compiled: &[Regex],
        ignore_compiled: &[Regex],
    ) -> Vec<Box<dyn Violation>> {
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
}

impl Validator for DeclarativeValidator {
    fn name(&self) -> &'static str {
        mcb_utils::constants::validate::VALIDATOR_DECLARATIVE
    }

    fn description(&self) -> &'static str {
        "Executes embedded YAML declarative rules"
    }

    fn enabled_by_default(&self) -> bool {
        true
    }

    fn validate(
        &self,
        config: &ValidationConfig,
    ) -> mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>> {
        let t_total = std::time::Instant::now();
        let t = std::time::Instant::now();
        let rules = self.load_rules()?;
        let enabled_count = rules.iter().filter(|r| r.enabled).count();
        mcb_domain::debug!(
            "declarative",
            "Rules loaded",
            &format!(
                "total={} enabled={} elapsed={:.2?}",
                rules.len(),
                enabled_count,
                t.elapsed()
            )
        );

        let t = std::time::Instant::now();
        let files = Self::collect_files(config, Some(LanguageId::Rust));
        mcb_domain::debug!(
            "declarative",
            "Files collected for declarative rules",
            &format!("file_count={} elapsed={:.2?}", files.len(), t.elapsed())
        );

        let t = std::time::Instant::now();
        let filter_executor = RuleFilterExecutor::new(self.workspace_root.clone());
        let workspace_deps = match filter_executor.parse_workspace_dependencies() {
            Ok(deps) => Some(deps),
            Err(e) => {
                mcb_domain::warn!(
                    "validate",
                    "Failed to parse workspace dependencies for declarative rules",
                    &e.to_string()
                );
                None
            }
        };
        mcb_domain::debug!(
            "declarative",
            "Workspace dependencies parsed",
            &format!(
                "available={} elapsed={:.2?}",
                workspace_deps.is_some(),
                t.elapsed()
            )
        );

        // Capture the active ValidationRunContext so spawned threads can use caches.
        let ctx = crate::run_context::ValidationRunContext::active_or_build(config).ok();

        let (metrics_v, ast_result, regex_v, path_v) = std::thread::scope(|s| {
            let t_metrics = s.spawn(|| {
                with_ctx(&ctx, || {
                    let t = std::time::Instant::now();
                    let v = Self::validate_metrics_rules(&rules, &files);
                    mcb_domain::debug!(
                        "declarative",
                        "Metrics slice done",
                        &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
                    );
                    v
                })
            });
            let t_ast = s.spawn(|| {
                with_ctx(&ctx, || {
                    let t = std::time::Instant::now();
                    let v = workspace_deps.as_ref().map_or_else(
                        || Ok(Vec::new()),
                        |deps| {
                            Self::validate_ast_selector_rules(
                                &rules,
                                &files,
                                &filter_executor,
                                deps,
                            )
                        },
                    );
                    mcb_domain::debug!(
                        "declarative",
                        "AST selector slice done",
                        &format!(
                            "violations={} elapsed={:.2?}",
                            v.as_ref().map_or(0, Vec::len),
                            t.elapsed()
                        )
                    );
                    v
                })
            });
            let t_regex = s.spawn(|| {
                with_ctx(&ctx, || {
                    let t = std::time::Instant::now();
                    let v = workspace_deps.as_ref().map_or_else(Vec::new, |deps| {
                        Self::validate_regex_rules(&rules, &files, &filter_executor, deps)
                    });
                    mcb_domain::debug!(
                        "declarative",
                        "Regex slice done",
                        &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
                    );
                    v
                })
            });
            let t_path = s.spawn(|| {
                with_ctx(&ctx, || {
                    let t = std::time::Instant::now();
                    let v = workspace_deps.as_ref().map_or_else(Vec::new, |deps| {
                        validate_path_rules(&rules, &files, &filter_executor, deps)
                    });
                    mcb_domain::debug!(
                        "declarative",
                        "Path rules slice done",
                        &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
                    );
                    v
                })
            });
            (
                t_metrics.join().unwrap_or_default(),
                t_ast.join().unwrap_or_else(|_| Ok(Vec::new())),
                t_regex.join().unwrap_or_default(),
                t_path.join().unwrap_or_default(),
            )
        });

        let t = std::time::Instant::now();
        let lint_v = Self::validate_lint_select_rules(&rules, &files);
        mcb_domain::debug!(
            "declarative",
            "Lint-select slice done",
            &format!("violations={} elapsed={:.2?}", lint_v.len(), t.elapsed())
        );

        let mut violations = metrics_v;
        violations.extend(lint_v);
        violations.extend(ast_result?);
        violations.extend(regex_v);
        violations.extend(path_v);

        mcb_domain::debug!(
            "declarative",
            "Declarative validator done",
            &format!(
                "violations={} elapsed={:.2?}",
                violations.len(),
                t_total.elapsed()
            )
        );

        Ok(violations)
    }
}

mcb_domain::register_validator!(
    mcb_utils::constants::validate::VALIDATOR_DECLARATIVE,
    "Executes embedded YAML declarative rules",
    |root| {
        Ok(Box::new(DeclarativeValidator::new(root))
            as Box<dyn mcb_domain::ports::validation::Validator>)
    }
);
