//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Declarative rule validator that executes embedded YAML rules.
//!
//! The execution slices live in sibling modules ([`metrics_rules`], [`lint_rules`],
//! [`regex_rules`], [`ast_rules`]); this module owns the struct, rule/file loading,
//! shared helpers, and the parallel orchestration.

mod ast_rules;
mod lint_rules;
mod metrics_rules;
mod regex_rules;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::Result;
use crate::ValidationConfig;
use crate::config::FileConfig;
use crate::embedded_rules::EmbeddedRules;
use crate::filters::LanguageId;
use crate::filters::dependency_parser::WorkspaceDependencies;
use crate::filters::rule_filters::RuleFilterExecutor;
use crate::rules::yaml_loader::{ValidatedRule, YamlRuleLoader};
use crate::scan::for_each_scan_file;
use crate::validators::declarative_support::{
    PatternMatchViolation, build_substitution_variables, parse_category, parse_severity,
    validate_path_rules,
};
use mcb_domain::ports::validation::Validator;
use mcb_domain::ports::validation::Violation;

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

    /// Whether `rule`'s filters permit execution on `file` (shared by regex/AST slices).
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

    /// Build a pattern-match violation (shared by regex/AST slices).
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

    /// Parse workspace dependencies via `filter_executor`, logging timing and a
    /// warning on failure, and returning `None` when parsing fails.
    fn parse_workspace_deps_logged(
        filter_executor: &RuleFilterExecutor,
    ) -> Option<WorkspaceDependencies> {
        let t = std::time::Instant::now();
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
        workspace_deps
    }

    /// Run the four declarative rule slices (metrics, AST, regex, path) in parallel.
    ///
    /// Returns `(metrics, ast_result, regex, path)`; a panicked slice degrades to an empty result.
    fn run_parallel_slices(
        rules: &[ValidatedRule],
        files: &[PathBuf],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: Option<&WorkspaceDependencies>,
        ctx: &Option<std::sync::Arc<crate::run_context::ValidationRunContext>>,
    ) -> ParallelSliceResults {
        std::thread::scope(|s| {
            let t_metrics = s.spawn(|| {
                with_ctx(ctx, || {
                    log_slice(
                        "Metrics slice done",
                        Self::validate_metrics_rules(rules, files),
                    )
                })
            });
            let t_ast = s.spawn(|| {
                with_ctx(ctx, || {
                    log_slice_result(
                        "AST selector slice done",
                        Self::ast_slice(rules, files, filter_executor, workspace_deps),
                    )
                })
            });
            let t_regex = s.spawn(|| {
                with_ctx(ctx, || {
                    log_slice(
                        "Regex slice done",
                        Self::regex_slice(rules, files, filter_executor, workspace_deps),
                    )
                })
            });
            let t_path = s.spawn(|| {
                with_ctx(ctx, || {
                    log_slice(
                        "Path rules slice done",
                        Self::path_slice(rules, files, filter_executor, workspace_deps),
                    )
                })
            });
            (
                join_slice(t_metrics, "Metrics validation thread panicked", Vec::new),
                join_slice(t_ast, "AST validation thread panicked", || Ok(Vec::new())),
                join_slice(t_regex, "Regex validation thread panicked", Vec::new),
                join_slice(t_path, "Path validation thread panicked", Vec::new),
            )
        })
    }

    /// AST-selector slice: empty when workspace dependencies are unavailable.
    fn ast_slice(
        rules: &[ValidatedRule],
        files: &[PathBuf],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: Option<&WorkspaceDependencies>,
    ) -> mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>> {
        workspace_deps.map_or_else(
            || Ok(Vec::new()),
            |deps| Self::validate_ast_selector_rules(rules, files, filter_executor, deps),
        )
    }

    /// Regex slice: empty when workspace dependencies are unavailable.
    fn regex_slice(
        rules: &[ValidatedRule],
        files: &[PathBuf],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: Option<&WorkspaceDependencies>,
    ) -> Vec<Box<dyn Violation>> {
        workspace_deps.map_or_else(Vec::new, |deps| {
            Self::validate_regex_rules(rules, files, filter_executor, deps)
        })
    }

    /// Path-rules slice: empty when workspace dependencies are unavailable.
    fn path_slice(
        rules: &[ValidatedRule],
        files: &[PathBuf],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: Option<&WorkspaceDependencies>,
    ) -> Vec<Box<dyn Violation>> {
        workspace_deps.map_or_else(Vec::new, |deps| {
            validate_path_rules(rules, files, filter_executor, deps)
        })
    }
}

/// Join a slice worker thread, returning `default()` (after logging `warn_msg`)
/// when the thread panicked.
fn join_slice<T>(
    handle: std::thread::ScopedJoinHandle<'_, T>,
    warn_msg: &str,
    default: impl FnOnce() -> T,
) -> T {
    handle.join().unwrap_or_else(|_| {
        mcb_domain::warn!("validate", warn_msg);
        default()
    })
}

/// Read file content from the active run-context cache, falling back to disk.
///
/// Returns `None` (after logging) when the file cannot be read; `context_label` names the
/// validation slice for the warning message.
fn read_cached_or_disk(file: &Path, context_label: &str) -> Option<std::sync::Arc<str>> {
    if let Some(cached) = crate::run_context::ValidationRunContext::active()
        .and_then(|ctx| ctx.read_cached(file).ok())
    {
        return Some(cached);
    }

    match std::fs::read_to_string(file) {
        Ok(content) => Some(std::sync::Arc::from(content.as_str())),
        Err(e) => {
            mcb_domain::warn!(
                "validate",
                "Failed to read file for validation",
                &format!(
                    "context = {context_label}, file = {}, error = {:?}",
                    file.display(),
                    e
                )
            );
            None
        }
    }
}

/// Result tuple of the four parallel declarative rule slices: `(metrics, ast, regex, path)`.
type ParallelSliceResults = (
    Vec<Box<dyn Violation>>,
    mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>>,
    Vec<Box<dyn Violation>>,
    Vec<Box<dyn Violation>>,
);

/// Emit a debug line with the slice violation count and pass the violations through.
fn log_slice(label: &str, v: Vec<Box<dyn Violation>>) -> Vec<Box<dyn Violation>> {
    mcb_domain::debug!("declarative", label, &format!("violations={}", v.len()));
    v
}

/// Like [`log_slice`] but for a fallible slice result.
fn log_slice_result(
    label: &str,
    v: mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>>,
) -> mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>> {
    mcb_domain::debug!(
        "declarative",
        label,
        &format!("violations={}", v.as_ref().map_or(0, Vec::len))
    );
    v
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
        Self::run_validation(self, config)
    }
}

impl DeclarativeValidator {
    /// Load declarative rules, logging the loaded/enabled counts and timing.
    fn load_rules_logged(
        &self,
    ) -> mcb_domain::ports::validation::ValidatorResult<Vec<ValidatedRule>> {
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
        Ok(rules)
    }

    /// Collect Rust files for declarative rules, logging the count and timing.
    fn collect_files_logged(config: &ValidationConfig) -> Vec<PathBuf> {
        let t = std::time::Instant::now();
        let files = Self::collect_files(config, Some(LanguageId::Rust));
        mcb_domain::debug!(
            "declarative",
            "Files collected for declarative rules",
            &format!("file_count={} elapsed={:.2?}", files.len(), t.elapsed())
        );
        files
    }

    /// Execute the full declarative validation pipeline.
    fn run_validation(
        &self,
        config: &ValidationConfig,
    ) -> mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>> {
        let t_total = std::time::Instant::now();
        let rules = self.load_rules_logged()?;
        let files = Self::collect_files_logged(config);

        let filter_executor = RuleFilterExecutor::new(self.workspace_root.clone());
        let workspace_deps = Self::parse_workspace_deps_logged(&filter_executor);

        // Capture the active ValidationRunContext so spawned threads can use caches.
        let ctx = crate::run_context::ValidationRunContext::active_or_build(config).ok();

        let (metrics_v, ast_result, regex_v, path_v) = Self::run_parallel_slices(
            &rules,
            &files,
            &filter_executor,
            workspace_deps.as_ref(),
            &ctx,
        );

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
