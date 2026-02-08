//! Rule Filter Executor
//!
//! Coordinates filtering of validation rules based on language, dependencies, and file patterns.
//! Prevents rules from running on irrelevant files for better performance and accuracy.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::dependency_parser::{CargoDependencyParser, WorkspaceDependencies};
use super::file_matcher::FilePatternMatcher;
use super::language_detector::LanguageDetector;

/// Filter configuration for a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFilters {
    /// Languages this rule applies to
    pub languages: Option<Vec<String>>,
    /// Dependencies that must be present for this rule to run
    pub dependencies: Option<Vec<String>>,
    /// File patterns this rule applies to (supports ! prefix for exclusions)
    pub file_patterns: Option<Vec<String>>,
}

impl RuleFilters {
    /// Check if filters are empty (no filtering)
    pub fn is_empty(&self) -> bool {
        self.languages.is_none() && self.dependencies.is_none() && self.file_patterns.is_none()
    }
}

/// Executor for rule filters
pub struct RuleFilterExecutor {
    language_detector: LanguageDetector,
    dependency_parser: CargoDependencyParser,
    file_matcher: FilePatternMatcher,
}

impl RuleFilterExecutor {
    /// Create a new filter executor
    pub fn new(workspace_root: std::path::PathBuf) -> Self {
        Self {
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
    pub async fn should_execute_rule(
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
        if let Some(patterns) = &filters.file_patterns
            && !self.file_matcher.matches_any(file_path, patterns)
        {
            return Ok(false);
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
