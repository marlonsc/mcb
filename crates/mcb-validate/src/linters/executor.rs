//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! YAML Rule Executor Module
//!
//! Executes YAML rules that use `lint_select` for linter-based validation.
//! Linters (Clippy for Rust, Ruff for Python) are auto-detected from the codes.

use std::path::{Path, PathBuf};

use super::constants::CLIPPY_PREFIX;
use super::engine::LinterEngine;
use super::types::{LintViolation, LinterType};
use crate::Result;
use crate::rules::yaml_loader::ValidatedRule;

/// Execute a YAML rule that uses `lint_select` for linter-based validation
///
/// This is the Phase 1 deliverable: YAML rule → linter → violations pipeline
pub struct YamlRuleExecutor;

impl YamlRuleExecutor {
    /// Execute a rule's `lint_select` codes against files
    ///
    /// # Arguments
    /// * `rule` - The validated YAML rule with `lint_select` codes
    /// * `files` - Files to check (Python for Ruff, Rust for Clippy)
    ///
    /// # Returns
    /// Violations that match the rule's `lint_select` codes
    /// # Errors
    ///
    /// Returns an error if the linter execution fails.
    pub async fn execute_rule(rule: &ValidatedRule, files: &[&Path]) -> Result<Vec<LintViolation>> {
        // Skip if no lint_select codes
        if rule.lint_select.is_empty() {
            return Ok(vec![]);
        }

        // Skip if rule is disabled
        if !rule.enabled {
            return Ok(vec![]);
        }

        // Determine which linters to use based on lint_select codes
        let linters = Self::detect_linters_from_codes(&rule.lint_select);

        if linters.is_empty() {
            return Ok(vec![]);
        }

        // Execute linters with the specific lint codes enabled
        // This is critical for Clippy lints like `clippy::unwrap_used` which are "allow" by default
        let engine = LinterEngine::with_lint_codes(linters, rule.lint_select.clone());
        let all_violations = engine.check_files(files).await?;

        // Filter violations to only include those matching lint_select codes
        let filtered: Vec<LintViolation> = all_violations
            .into_iter()
            .filter(|v| rule.lint_select.contains(&v.rule))
            .map(|mut v| {
                if let Some(ref msg) = rule.message {
                    v.message.clone_from(msg);
                }
                v.category.clone_from(&rule.category);
                v
            })
            .collect();

        Ok(filtered)
    }

    /// Execute a rule against a directory (scans for appropriate files)
    ///
    /// # Errors
    ///
    /// Returns an error if file collection or linter execution fails.
    pub async fn execute_rule_on_dir(
        rule: &ValidatedRule,
        dir: &Path,
    ) -> Result<Vec<LintViolation>> {
        // Collect files based on linter type
        let linters = Self::detect_linters_from_codes(&rule.lint_select);
        let mut files: Vec<PathBuf> = Vec::new();
        collect_matching_files(dir, &linters, &mut files)?;

        let file_refs: Vec<&Path> = files.iter().map(|p: &PathBuf| p.as_path()).collect();
        Self::execute_rule(rule, &file_refs).await
    }

    /// Detect which linters to use based on `lint_select` codes
    fn detect_linters_from_codes(codes: &[String]) -> Vec<LinterType> {
        let mut linters = Vec::new();

        for code in codes {
            if code.starts_with(CLIPPY_PREFIX) {
                if !linters.contains(&LinterType::Clippy) {
                    linters.push(LinterType::Clippy);
                }
            } else {
                // Ruff codes: F*, E*, W*, I*, B*, C*, D*, N*, S*, etc.
                if !linters.contains(&LinterType::Ruff) {
                    linters.push(LinterType::Ruff);
                }
            }
        }

        linters
    }
}

fn collect_matching_files(
    dir: &Path,
    linters: &[LinterType],
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir).map_err(crate::ValidationError::Io)? {
        let entry = entry.map_err(crate::ValidationError::Io)?;
        let path = entry.path();
        if path.is_dir() {
            collect_matching_files(&path, linters, files)?;
            continue;
        }

        if path.is_file() {
            let ext = path.extension().and_then(std::ffi::OsStr::to_str);
            if linters.iter().any(|lt| lt.matches_extension(ext)) {
                files.push(path);
            }
        }
    }

    Ok(())
}
