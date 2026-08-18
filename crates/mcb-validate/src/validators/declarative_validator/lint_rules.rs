//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Lint-select rule slice for the declarative validator.

use std::path::{Path, PathBuf};

use super::DeclarativeValidator;
use crate::linters::YamlRuleExecutor;
use crate::rules::yaml_loader::ValidatedRule;
use mcb_domain::ports::validation::Violation;

/// Availability of external linters and relevant source files for lint-select rules.
struct LintAvailability {
    has_python_files: bool,
    has_ruff: bool,
    has_cargo: bool,
}

/// Decide whether a lint-select rule should be skipped given tool/file availability.
fn lint_rule_should_skip(rule: &ValidatedRule, avail: &LintAvailability) -> bool {
    let needs_clippy = rule
        .lint_select
        .iter()
        .any(|code| code.starts_with("clippy::"));
    let needs_ruff = rule
        .lint_select
        .iter()
        .any(|code| !code.starts_with("clippy::"));

    if (needs_clippy && !avail.has_cargo)
        || (needs_ruff && (!avail.has_ruff || !avail.has_python_files))
    {
        mcb_domain::trace!(
            "declarative",
            "Skipping lint-select rule",
            &format!(
                "rule={} needs_clippy={} needs_ruff={} has_cargo={} has_ruff={} has_python_files={}",
                rule.id,
                needs_clippy,
                needs_ruff,
                avail.has_cargo,
                avail.has_ruff,
                avail.has_python_files
            )
        );
        return true;
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
        return true;
    }

    false
}

impl DeclarativeValidator {
    // Rust-only: non-Rust lint selectors (e.g. Ruff) receive no matching files.
    pub(crate) fn validate_lint_select_rules(
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
        let avail = LintAvailability {
            has_python_files,
            has_ruff,
            has_cargo,
        };
        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        for rule in &lint_rules {
            if lint_rule_should_skip(rule, &avail) {
                continue;
            }

            mcb_domain::trace!(
                "declarative",
                "Running lint-select rule",
                &format!("rule={}", rule.id)
            );
            Self::run_lint_rule(rule, &file_refs, &mut violations);
        }

        violations
    }

    /// Execute a single lint-select rule on a dedicated runtime, appending its violations.
    ///
    /// Runs on a dedicated thread with its own current-thread runtime to avoid calling
    /// `block_on`/`block_in_place` inside the caller's async context (which would panic).
    fn run_lint_rule(
        rule: &ValidatedRule,
        file_refs: &[&Path],
        violations: &mut Vec<Box<dyn Violation>>,
    ) {
        let result = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|error| format!("failed to build lint runtime: {error}"))?;
                    runtime
                        .block_on(YamlRuleExecutor::execute_rule(rule, file_refs))
                        .map_err(|error| format!("lint execution failed: {error}"))
                })
                .join()
                .map_err(|_| "lint thread panicked".to_owned())
                .and_then(std::convert::identity)
        });

        match result {
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

    fn command_is_available(command: &str) -> bool {
        std::process::Command::new(command)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }
}
