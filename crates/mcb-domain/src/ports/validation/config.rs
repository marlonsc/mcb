//! Validation configuration and check runner.

use std::path::{Path, PathBuf};

use super::types::{ValidatorResult, Violation};

/// Configuration for a validation run.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// The absolute path to the workspace root.
    pub workspace_root: PathBuf,
    /// Additional source paths to include in the scan.
    pub additional_src_paths: Vec<PathBuf>,
    /// Glob patterns for excluding files or directories.
    pub exclude_patterns: Vec<String>,
}

impl ValidationConfig {
    /// Create a new validation configuration for the specified workspace.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let raw: PathBuf = workspace_root.into();
        let canonical = std::fs::canonicalize(&raw).unwrap_or(raw);
        Self {
            workspace_root: canonical,
            additional_src_paths: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }

    /// Add an extra directory to include in the validation scan.
    #[must_use]
    pub fn with_additional_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.additional_src_paths.push(path.into());
        self
    }

    /// Add a glob pattern for excluding files or directories from the validation scan.
    #[must_use]
    pub fn with_exclude_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Check if a specific path should be excluded according to the configured patterns.
    #[must_use]
    pub fn should_exclude(&self, path: &Path) -> bool {
        let Some(path_str) = path.to_str() else {
            return false;
        };
        self.exclude_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }
}

/// Function type for a validation check.
pub type CheckFn<'a> = Box<dyn FnOnce() -> ValidatorResult<Vec<Box<dyn Violation>>> + 'a>;

/// A named validation check entry.
pub struct NamedCheck<'a> {
    /// The human-readable name of the check.
    pub name: &'static str,
    /// The logic of the check.
    pub run: CheckFn<'a>,
}

impl<'a> NamedCheck<'a> {
    /// Create a new named check.
    pub fn new(
        name: &'static str,
        run: impl FnOnce() -> ValidatorResult<Vec<Box<dyn Violation>>> + 'a,
    ) -> Self {
        Self {
            name,
            run: Box::new(run),
        }
    }
}

/// Runs a series of named checks and returns the combined violations.
///
/// # Errors
/// Returns a `ValidatorError` if any individual check fails or returns an error.
pub fn run_checks(
    validator_name: &str,
    checks: Vec<NamedCheck<'_>>,
) -> ValidatorResult<Vec<Box<dyn Violation>>> {
    let mut violations = Vec::new();
    for check in checks {
        let t = std::time::Instant::now();
        let v = (check.run)()?;
        crate::debug!(
            validator_name,
            &format!("{} done", check.name),
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);
    }
    Ok(violations)
}
