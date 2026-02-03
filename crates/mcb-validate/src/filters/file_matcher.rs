//! File Pattern Matching
//!
//! Matches file paths against glob patterns for rule filtering.
//! Supports inclusion and exclusion patterns.

use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

/// Matcher for file patterns using glob syntax
pub struct FilePatternMatcher {
    includes: GlobSet,
    excludes: GlobSet,
}

impl FilePatternMatcher {
    /// Create a new matcher from pattern lists
    ///
    /// # Arguments
    /// * `include_patterns` - Patterns that files must match (e.g., ["src/**/*.rs", "tests/**/*.rs"])
    /// * `exclude_patterns` - Patterns that files must NOT match (e.g., ["**/target/**", "**/*_test.rs"])
    pub fn new(
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> Result<Self, globset::Error> {
        let mut include_builder = GlobSetBuilder::new();
        for pattern in include_patterns {
            include_builder.add(Glob::new(pattern)?);
        }

        let mut exclude_builder = GlobSetBuilder::new();
        for pattern in exclude_patterns {
            exclude_builder.add(Glob::new(pattern)?);
        }

        Ok(Self {
            includes: include_builder.build()?,
            excludes: exclude_builder.build()?,
        })
    }

    /// Check if a file path matches the patterns
    ///
    /// # Arguments
    /// * `path` - File path to check
    /// * `patterns` - List of patterns to match against (supports ! prefix for exclusion)
    ///
    /// # Returns
    /// true if the path matches any of the patterns
    pub fn matches_any(&self, path: &Path, patterns: &[String]) -> bool {
        let (includes, excludes) = Self::parse_patterns(patterns);

        // First check exclusions (they take precedence)
        for exclude_pattern in &excludes {
            if let Ok(glob) = Glob::new(exclude_pattern)
                && glob.compile_matcher().is_match(path)
            {
                return false;
            }
        }

        // Then check inclusions
        for include_pattern in &includes {
            if let Ok(glob) = Glob::new(include_pattern)
                && glob.compile_matcher().is_match(path)
            {
                return true;
            }
        }

        false
    }

    /// Check if a file path should be included based on include/exclude sets
    ///
    /// # Arguments
    /// * `path` - File path to check
    ///
    /// # Returns
    /// true if the file should be included (matches includes and not excludes)
    pub fn should_include(&self, path: &Path) -> bool {
        // Must match at least one include pattern (if any includes are defined)
        let matches_include = if self.includes.is_empty() {
            true // No includes means everything is included
        } else {
            self.includes.is_match(path)
        };

        // Must not match any exclude pattern
        let matches_exclude = self.excludes.is_match(path);

        matches_include && !matches_exclude
    }

    /// Parse patterns with ! prefix for exclusions
    ///
    /// # Arguments
    /// * `patterns` - Mixed list of include and exclude patterns
    ///
    /// # Returns
    /// Tuple of (`include_patterns`, `exclude_patterns`)
    pub fn parse_patterns(patterns: &[String]) -> (Vec<String>, Vec<String>) {
        let mut includes = Vec::new();
        let mut excludes = Vec::new();

        for pattern in patterns {
            if pattern.starts_with('!') {
                excludes.push(pattern[1..].to_string());
            } else {
                includes.push(pattern.clone());
            }
        }

        (includes, excludes)
    }

    /// Create matcher from mixed patterns
    pub fn from_mixed_patterns(patterns: &[String]) -> Result<Self, globset::Error> {
        let (includes, excludes) = Self::parse_patterns(patterns);
        Self::new(&includes, &excludes)
    }
}

impl Default for FilePatternMatcher {
    fn default() -> Self {
        // Default: include everything, exclude nothing
        Self::new(&[], &[]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_includes() {
        let matcher = FilePatternMatcher::new(&["*.rs".to_string()], &[]).unwrap();

        assert!(matcher.should_include(Path::new("main.rs")));
        assert!(matcher.should_include(Path::new("lib.rs")));
        assert!(!matcher.should_include(Path::new("main.py")));
        assert!(!matcher.should_include(Path::new("README.md")));
    }

    #[test]
    fn test_includes_and_excludes() {
        let matcher = FilePatternMatcher::new(
            &["src/**/*.rs".to_string()],
            &["**/test/**".to_string(), "**/*_test.rs".to_string()],
        )
        .unwrap();

        assert!(matcher.should_include(Path::new("src/main.rs")));
        assert!(matcher.should_include(Path::new("src/utils/helper.rs")));
        assert!(!matcher.should_include(Path::new("src/tests/integration_test.rs")));
        assert!(!matcher.should_include(Path::new("src/utils_test.rs")));
        assert!(!matcher.should_include(Path::new("tests/main.rs")));
    }

    #[test]
    fn test_matches_any() {
        let matcher = FilePatternMatcher::default();

        let patterns = vec![
            "src/**/*.rs".to_string(),
            "!**/test/**".to_string(), // Excludes paths with "test" (singular) directory
            "tests/**/*.py".to_string(),
        ];

        assert!(matcher.matches_any(Path::new("src/main.rs"), &patterns));
        assert!(matcher.matches_any(Path::new("tests/test.py"), &patterns));
        // "src/test/main.rs" has singular "test", so it matches the exclusion
        assert!(!matcher.matches_any(Path::new("src/test/main.rs"), &patterns));
        assert!(!matcher.matches_any(Path::new("lib.py"), &patterns));
    }

    #[test]
    fn test_parse_patterns() {
        let patterns = vec![
            "src/**/*.rs".to_string(),
            "!**/test/**".to_string(),
            "tests/**/*.py".to_string(),
            "!**/*.tmp".to_string(),
        ];

        let (includes, excludes) = FilePatternMatcher::parse_patterns(&patterns);

        assert_eq!(includes, vec!["src/**/*.rs", "tests/**/*.py"]);
        assert_eq!(excludes, vec!["**/test/**", "**/*.tmp"]);
    }

    #[test]
    fn test_from_mixed_patterns() {
        let patterns = vec![
            "src/**/*.rs".to_string(),
            "!**/test_utils/**".to_string(), // Excludes test_utils directories
            "tests/**/*.py".to_string(),
        ];

        let matcher = FilePatternMatcher::from_mixed_patterns(&patterns).unwrap();

        assert!(matcher.should_include(Path::new("src/main.rs")));
        assert!(matcher.should_include(Path::new("tests/test.py")));
        // Exclusion applies to test_utils directories
        assert!(!matcher.should_include(Path::new("src/test_utils/helpers.rs")));
        assert!(!matcher.should_include(Path::new("lib.py")));
    }

    #[test]
    fn test_default_matcher() {
        let matcher = FilePatternMatcher::default();

        // Should include everything when no patterns are set
        assert!(matcher.should_include(Path::new("any/file.rs")));
        assert!(matcher.should_include(Path::new("any/file.py")));
        assert!(matcher.should_include(Path::new("any/file.txt")));
    }
}
