//! Rule Filter Executor
//!
//! Coordinates filtering of validation rules based on language, dependencies, and file patterns.
//! Prevents rules from running on irrelevant files for better performance and accuracy.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::dependency_parser::{CargoDependencyParser, WorkspaceDependencies};
use super::file_matcher::FilePatternMatcher;
use super::language_detector::LanguageDetector;

/// File/directory pattern filter for allow/deny/skip applicability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplicabilityFilter {
    /// Glob patterns matching file names or paths.
    pub file_patterns: Option<Vec<String>>,
    /// Glob patterns matching directory names or paths.
    pub directory_patterns: Option<Vec<String>>,
}

impl ApplicabilityFilter {
    /// Returns `true` when neither file nor directory patterns are defined.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.file_patterns.as_ref().is_none_or(|v| v.is_empty())
            && self
                .directory_patterns
                .as_ref()
                .is_none_or(|v| v.is_empty())
    }
}

/// Filter configuration for a rule.
///
/// Precedence: `skip` > `deny` > `allow`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFilters {
    /// Language identifiers the rule applies to (e.g. `["rust", "python"]`).
    pub languages: Option<Vec<String>>,
    /// Crate/package names whose presence activates this rule.
    pub dependencies: Option<Vec<String>>,
    /// Glob patterns for files the rule should match.
    pub file_patterns: Option<Vec<String>>,
    /// Paths explicitly allowed (overridden by `deny` and `skip`).
    pub allow: Option<ApplicabilityFilter>,
    /// Paths explicitly denied (overridden by `skip`).
    pub deny: Option<ApplicabilityFilter>,
    /// Paths unconditionally excluded from this rule.
    pub skip: Option<ApplicabilityFilter>,
}

impl RuleFilters {
    /// Returns `true` when no filter criteria are configured.
    pub fn is_empty(&self) -> bool {
        self.languages.is_none()
            && self.dependencies.is_none()
            && self.file_patterns.is_none()
            && self
                .allow
                .as_ref()
                .is_none_or(ApplicabilityFilter::is_empty)
            && self.deny.as_ref().is_none_or(ApplicabilityFilter::is_empty)
            && self.skip.as_ref().is_none_or(ApplicabilityFilter::is_empty)
    }
}

/// Executor for rule filters
pub struct RuleFilterExecutor {
    workspace_root: std::path::PathBuf,
    language_detector: LanguageDetector,
    dependency_parser: CargoDependencyParser,
    file_matcher: FilePatternMatcher,
}

impl RuleFilterExecutor {
    /// Create a new filter executor
    pub fn new(workspace_root: std::path::PathBuf) -> Self {
        Self {
            workspace_root: workspace_root.clone(),
            language_detector: LanguageDetector::new(),
            dependency_parser: CargoDependencyParser::new(workspace_root),
            file_matcher: FilePatternMatcher::default(),
        }
    }

    /// Check if a rule should execute on a given file
    ///
    /// # Arguments
    /// * `filters` - Filter configuration for the rule
    /// * `file_path` - Path to the file being checked
    /// * `file_content` - Optional content of the file (for language detection)
    /// * `workspace_deps` - Workspace dependency information
    ///
    /// # Returns
    /// true if the rule should execute on this file
    /// Check if a rule should execute on a given file
    ///
    /// # Arguments
    /// * `filters` - Filter configuration for the rule
    /// * `file_path` - Path to the file being checked
    /// * `file_content` - Optional content of the file (for language detection)
    /// * `workspace_deps` - Workspace dependency information
    ///
    /// # Returns
    /// true if the rule should execute on this file
    pub fn should_execute_rule(
        &self,
        filters: &RuleFilters,
        file_path: &Path,
        file_content: Option<&str>,
        workspace_deps: &WorkspaceDependencies,
    ) -> crate::Result<bool> {
        // If no filters are defined, rule always executes
        if filters.is_empty() {
            return Ok(true);
        }

        // Check language filter
        if let Some(languages) = &filters.languages
            && !self
                .language_detector
                .matches_languages(file_path, file_content, languages)
        {
            return Ok(false);
        }

        // Check dependency filter
        if let Some(required_deps) = &filters.dependencies
            && !self.check_dependencies(required_deps, file_path, workspace_deps)
        {
            return Ok(false);
        }

        // Check file pattern filter
        if let Some(patterns) = &filters.file_patterns {
            let rel_path = file_path
                .strip_prefix(&self.workspace_root)
                .unwrap_or(file_path);

            if !self.file_matcher.matches_any(rel_path, patterns) {
                // If relative path didn't match, check absolute path as fallback
                // This covers cases where patterns might be absolute or files outside workspace
                if rel_path == file_path || !self.file_matcher.matches_any(file_path, patterns) {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Check if required dependencies are present in the file's crate
    fn check_dependencies(
        &self,
        required_deps: &[String],
        file_path: &Path,
        workspace_deps: &WorkspaceDependencies,
    ) -> bool {
        if let Some(crate_deps) = workspace_deps.find_crate_deps(file_path) {
            required_deps
                .iter()
                .all(|dep| crate_deps.has_dependency(dep))
        } else {
            // If we can't determine the crate, assume dependencies are not present
            false
        }
    }

    /// Parse workspace dependencies (can be cached)
    pub fn parse_workspace_dependencies(&self) -> crate::Result<WorkspaceDependencies> {
        self.dependency_parser.parse_workspace_deps()
    }

    /// Create a file matcher for specific patterns
    pub fn create_file_matcher(&self, patterns: &[String]) -> crate::Result<FilePatternMatcher> {
        FilePatternMatcher::from_mixed_patterns(patterns)
            .map_err(|e| crate::ValidationError::Config(format!("Invalid file pattern: {e}")))
    }

    /// Get the language detector for direct use
    pub fn language_detector(&self) -> &LanguageDetector {
        &self.language_detector
    }

    /// Get the dependency parser for direct use
    pub fn dependency_parser(&self) -> &CargoDependencyParser {
        &self.dependency_parser
    }
}
