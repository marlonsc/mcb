//! Refactoring Completeness Validation
//!
//! Validates that refactorings are complete and not left halfway:
//! - Orphan imports (use statements pointing to deleted/moved modules)
//! - Duplicate definitions (same type in multiple locations)
//! - Missing test files for new source files
//! - Stale re-exports (pub use of items that don't exist)
//! - Deleted module references
//! - Dead code from refactoring

pub mod constants;
mod duplicates;
mod modules;
mod tests;
mod violation;

use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::RefactoringRulesConfig;
use crate::{Result, ValidationConfig};

pub use self::violation::RefactoringViolation;
use duplicates::validate_duplicate_definitions;
use modules::validate_mod_declarations;
use tests::validate_missing_test_files;

/// Refactoring completeness validator
pub struct RefactoringValidator {
    pub(crate) config: ValidationConfig,
    pub(crate) rules: RefactoringRulesConfig,
    pub(crate) generic_type_names: HashSet<String>,
    pub(crate) utility_types: HashSet<String>,
    pub(crate) known_migration_pairs: Vec<(String, String)>,
    pub(crate) skip_files: HashSet<String>,
    pub(crate) skip_dir_patterns: Vec<String>,
}

impl RefactoringValidator {
    /// Create a new refactoring validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.refactoring)
    }

    /// Create a validator with custom configuration
    #[must_use]
    pub fn with_config(config: ValidationConfig, rules: &RefactoringRulesConfig) -> Self {
        let generic_type_names: HashSet<String> =
            rules.generic_type_names.iter().cloned().collect();
        let utility_types: HashSet<String> = rules.utility_types.iter().cloned().collect();
        let skip_files: HashSet<String> = rules.skip_files.iter().cloned().collect();
        let skip_dir_patterns = rules.skip_dir_patterns.clone();

        let known_migration_pairs = rules
            .known_migration_pairs
            .iter()
            .filter_map(|pair| {
                if pair.len() == 2 {
                    Some((pair[0].clone(), pair[1].clone()))
                } else {
                    None
                }
            })
            .collect();

        Self {
            config,
            rules: rules.clone(),
            generic_type_names,
            utility_types,
            known_migration_pairs,
            skip_files,
            skip_dir_patterns,
        }
    }

    /// Run all refactoring validations
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning, reading, or regex compilation fails.
    pub fn validate_all(&self) -> Result<Vec<RefactoringViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();
        violations.extend(validate_duplicate_definitions(self)?);
        violations.extend(validate_missing_test_files(self)?);
        violations.extend(validate_mod_declarations(self)?);
        Ok(violations)
    }

    /// Get all crate directories
    pub(crate) fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();
        let crates_dir = self.config.workspace_root.join("crates");

        if crates_dir.exists() {
            for entry in std::fs::read_dir(&crates_dir)? {
                let entry = entry?;
                let path = entry.path();

                if self.should_skip_crate(&path) {
                    continue;
                }

                if path.is_dir() {
                    dirs.push(path);
                }
            }
        }

        Ok(dirs)
    }

    /// Check if a crate should be skipped based on configuration
    pub(crate) fn should_skip_crate(&self, crate_dir: &std::path::Path) -> bool {
        let Some(path_str) = crate_dir.to_str() else {
            return false;
        };
        self.rules
            .excluded_crates
            .iter()
            .any(|excluded| path_str.contains(excluded))
    }
}

crate::impl_validator!(
    RefactoringValidator,
    "refactoring",
    "Validates refactoring completeness (duplicate definitions, missing tests, stale references)"
);
