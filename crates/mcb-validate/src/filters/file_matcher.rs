//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! File Pattern Matching
//!
//! Matches file paths against glob patterns for rule filtering.
//! Supports inclusion and exclusion patterns (including `!`-prefixed exclusions).
//!
//! # Example
//!
//! ```
//! # use mcb_validate::filters::file_matcher::FilePatternMatcher;
//! # use std::path::Path;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let matcher = FilePatternMatcher::new(&["src/**/*.rs".to_string()], &["**/target/**".to_string()])?;
//! assert!(matcher.should_include(Path::new("src/lib.rs")));
//! assert!(!matcher.should_include(Path::new("target/debug/lib.rs")));
//! # Ok(())
//! # }
//! ```

use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

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
    /// * `exclude_patterns` - Patterns that files must NOT match (e.g. `["**/target/**", "**/*_test.rs"]`)
    ///
    /// # Errors
    ///
    /// Returns an error if any glob pattern is invalid.
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn parse_patterns(patterns: &[String]) -> (Vec<String>, Vec<String>) {
        let mut includes = Vec::new();
        let mut excludes = Vec::new();

        for pattern in patterns {
            if let Some(stripped) = pattern.strip_prefix('!') {
                excludes.push(stripped.to_owned());
            } else {
                includes.push(pattern.clone());
            }
        }

        (includes, excludes)
    }

    /// Create matcher from mixed patterns
    ///
    /// # Errors
    ///
    /// Returns an error if any glob pattern is invalid.
    pub fn from_mixed_patterns(patterns: &[String]) -> Result<Self, globset::Error> {
        let (includes, excludes) = Self::parse_patterns(patterns);
        Self::new(&includes, &excludes)
    }
}

impl Default for FilePatternMatcher {
    /// Returns a matcher that includes all paths and excludes none.
    fn default() -> Self {
        Self {
            includes: GlobSet::empty(),
            excludes: GlobSet::empty(),
        }
    }
}
