//! Refactoring Completeness Validation
//!
//! Validates that refactorings are complete and not left halfway:
//! - Orphan imports (use statements pointing to deleted/moved modules)
//! - Duplicate definitions (same type in multiple locations)
//! - Missing test files for new source files
//! - Stale re-exports (pub use of items that don't exist)
//! - Deleted module references
//! - Dead code from refactoring

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::config::RefactoringRulesConfig;
use crate::scan::{for_each_rs_under_root, for_each_scan_rs_path};
use crate::traits::violation::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

crate::define_violations! {
    dynamic_severity,
    ViolationCategory::Refactoring,
    pub enum RefactoringViolation {
        /// Import statement referencing non-existent module/item
        #[violation(
            id = "REF001",
            severity = Warning,
            message = "Orphan import at {file}:{line} - '{import_path}' - {suggestion}",
            suggestion = "{suggestion}"
        )]
        OrphanImport {
            file: PathBuf,
            line: usize,
            import_path: String,
            suggestion: String,
            severity: Severity,
        },
        /// Same type name defined in multiple locations (incomplete migration)
        #[violation(
            id = "REF002",
            severity = Warning,
            message = "Duplicate definition '{type_name}' in [{locations}] - {suggestion}",
            suggestion = "{suggestion}"
        )]
        DuplicateDefinition {
            type_name: String,
            locations: Vec<PathBuf>,
            suggestion: String,
            severity: Severity,
        },
        /// New source file without corresponding test file
        #[violation(
            id = "REF003",
            severity = Warning,
            message = "Missing test file for {source_file} - expected {expected_test}",
            suggestion = "Create test file {expected_test} for source {source_file}"
        )]
        MissingTestFile {
            source_file: PathBuf,
            expected_test: PathBuf,
            severity: Severity,
        },
        /// pub use/mod statement for item that doesn't exist
        #[violation(
            id = "REF004",
            severity = Warning,
            message = "Stale re-export at {file}:{line} - '{re_export}'",
            suggestion = "Remove or update the re-export '{re_export}'"
        )]
        StaleReExport {
            file: PathBuf,
            line: usize,
            re_export: String,
            severity: Severity,
        },
        /// File/module that was deleted but is still referenced
        #[violation(
            id = "REF005",
            severity = Warning,
            message = "Reference to deleted module at {referencing_file}:{line} - '{deleted_module}'",
            suggestion = "Remove the mod declaration for '{deleted_module}' or create the module file"
        )]
        DeletedModuleReference {
            referencing_file: PathBuf,
            line: usize,
            deleted_module: String,
            severity: Severity,
        },
        /// Dead code left from refactoring (unused after move)
        #[violation(
            id = "REF006",
            severity = Warning,
            message = "Dead {item_type} '{item_name}' from refactoring at {file}",
            suggestion = "Remove the unused {item_type} '{item_name}' or use it"
        )]
        RefactoringDeadCode {
            file: PathBuf,
            item_name: String,
            item_type: String,
            severity: Severity,
        },
    }
}

impl RefactoringViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

/// Refactoring completeness validator
pub struct RefactoringValidator {
    config: ValidationConfig,
    rules: RefactoringRulesConfig,
    generic_type_names: HashSet<String>,
    utility_types: HashSet<String>,
    known_migration_pairs: Vec<(String, String)>,
    skip_files: HashSet<String>,
    skip_dir_patterns: Vec<String>,
}

impl RefactoringValidator {
    /// Create a new refactoring validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.refactoring)
    }

    /// Create a validator with custom configuration
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
    pub fn validate_all(&self) -> Result<Vec<RefactoringViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();
        violations.extend(self.validate_duplicate_definitions()?);
        violations.extend(self.validate_missing_test_files()?);
        violations.extend(self.validate_mod_declarations()?);
        Ok(violations)
    }

    /// Check for same type defined in multiple locations
    pub fn validate_duplicate_definitions(&self) -> Result<Vec<RefactoringViolation>> {
        let mut violations = Vec::new();
        let definition_pattern =
            Regex::new(r"(?:pub\s+)?(?:struct|trait|enum)\s+([A-Z][a-zA-Z0-9_]*)(?:\s*<|\s*\{|\s*;|\s*\(|\s+where)")
                .unwrap();

        // Map: type_name -> Vec<file_path>
        let mut definitions: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }

            for_each_scan_rs_path(&self.config, false, |path, candidate_src_dir| {
                if candidate_src_dir != src_dir {
                    return Ok(());
                }

                let Some(path_str) = path.to_str() else {
                    return Ok(());
                };

                // Skip test files and archived directories
                if path_str.contains("/tests/")
                    || path_str.contains("_test.rs")
                    || path_str.contains(".archived")
                    || path_str.contains(".bak")
                {
                    return Ok(());
                }

                let content = std::fs::read_to_string(path)?;

                for cap in definition_pattern.captures_iter(&content) {
                    let type_name = cap.get(1).map_or("", |m| m.as_str());

                    // Skip generic names that are expected to appear in multiple places
                    if self.generic_type_names.contains(type_name) {
                        continue;
                    }

                    definitions
                        .entry(type_name.to_string())
                        .or_default()
                        .push(path.to_path_buf());
                }

                Ok(())
            })?;
        }

        // Find duplicates (same name in different files)
        for (type_name, locations) in definitions {
            if locations.len() > 1 {
                // Check if duplicates are in different crates
                let crates: HashSet<String> = locations
                    .iter()
                    .filter_map(|p| {
                        p.to_str()?
                            .split("/crates/")
                            .nth(1)
                            .and_then(|s| s.split('/').next())
                            .map(std::string::ToString::to_string)
                    })
                    .collect();

                if crates.len() > 1 {
                    // Cross-crate duplicate - categorize by pattern
                    let severity = self.categorize_duplicate_severity(&type_name, &crates);

                    let suggestion = match severity {
                        Severity::Info => format!(
                            "Type '{type_name}' exists in {crates:?}. This is a known migration pattern - consolidate when migration completes."
                        ),
                        Severity::Warning => format!(
                            "Type '{type_name}' is defined in {crates:?}. Consider consolidating to one location."
                        ),
                        Severity::Error => format!(
                            "Type '{type_name}' is unexpectedly defined in multiple crates: {crates:?}. This requires immediate consolidation."
                        ),
                    };

                    violations.push(RefactoringViolation::DuplicateDefinition {
                        type_name: type_name.clone(),
                        locations: locations.clone(),
                        suggestion,
                        severity,
                    });
                } else if locations.len() >= 2 {
                    // Same crate but duplicates - check if in different directories
                    let dirs: HashSet<String> = locations
                        .iter()
                        .filter_map(|p| {
                            let parent = p.parent()?;
                            let parent_str = parent.to_str()?;
                            Some(parent_str.to_string())
                        })
                        .collect();

                    // Only flag if duplicates are in different directories (not just mod.rs + impl.rs)
                    if dirs.len() >= 2 {
                        violations.push(RefactoringViolation::DuplicateDefinition {
                            type_name: type_name.clone(),
                            locations: locations.clone(),
                            suggestion: format!(
                                "Type '{}' is defined {} times in different directories within the same crate. Consolidate to single location.",
                                type_name,
                                locations.len()
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Categorize duplicate severity based on known patterns
    fn categorize_duplicate_severity(&self, type_name: &str, crates: &HashSet<String>) -> Severity {
        // Check if this is an intentionally duplicated utility type
        if self.utility_types.contains(type_name) {
            return Severity::Info;
        }

        // Check if the crates match a known migration pattern
        let crate_vec: Vec<&String> = crates.iter().collect();
        if crate_vec.len() == 2 {
            for (crate_a, crate_b) in &self.known_migration_pairs {
                if (crate_vec[0].as_str() == *crate_a && crate_vec[1].as_str() == *crate_b)
                    || (crate_vec[0].as_str() == *crate_b && crate_vec[1].as_str() == *crate_a)
                {
                    // This is a known migration pair - downgrade to Info
                    return Severity::Info;
                }
            }
        }

        // Check for patterns that suggest migration in progress
        // Types ending with Provider, Processor, etc. between known pairs
        let migration_type_patterns = [
            "Provider",
            "Processor",
            "Handler",
            "Service",
            "Repository",
            "Adapter",
            "Factory",
            "Publisher",
            "Subscriber",
        ];

        if migration_type_patterns
            .iter()
            .any(|p| type_name.ends_with(p))
        {
            // Check if any known migration pair is involved
            for (crate_a, crate_b) in &self.known_migration_pairs {
                if crates.contains(crate_a) || crates.contains(crate_b) {
                    return Severity::Warning; // Migration-related, but should be tracked
                }
            }
        }

        Severity::Warning
    }

    /// Check for source files without corresponding test files
    pub fn validate_missing_test_files(&self) -> Result<Vec<RefactoringViolation>> {
        let mut violations = Vec::new();
        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            let tests_dir = crate_dir.join("tests");

            if !src_dir.exists() {
                continue;
            }

            if self.should_skip_crate(&crate_dir) {
                continue;
            }

            // If tests directory doesn't exist, skip this crate (no test infrastructure)
            if !tests_dir.exists() {
                continue;
            }

            // Collect existing test files and directories
            let mut test_files: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let mut test_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

            for_each_rs_under_root(&self.config, &tests_dir, |path| {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    test_files.insert(stem.to_string());
                    if let Some(base) = stem.strip_suffix("_test") {
                        test_files.insert(base.to_string());
                    }
                    if let Some(base) = stem.strip_suffix("_tests") {
                        test_files.insert(base.to_string());
                    }
                }

                let mut parent = path.parent();
                while let Some(dir) = parent {
                    if !dir.starts_with(&tests_dir) {
                        break;
                    }
                    if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                        test_dirs.insert(name.to_string());
                    }
                    parent = dir.parent();
                }

                Ok(())
            })?;

            // Check each source file
            for_each_rs_under_root(&self.config, &src_dir, |path| {
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                // Skip common files that don't need dedicated tests
                if self.skip_files.contains(file_name) {
                    return Ok(());
                }

                // Get relative path for directory checks
                let relative = path.strip_prefix(&src_dir).unwrap_or(path);
                let Some(path_str) = relative.to_str() else {
                    return Ok(());
                };

                // Skip files in directories that are tested via integration tests
                let in_skip_dir = self
                    .skip_dir_patterns
                    .iter()
                    .any(|pattern| path_str.contains(pattern));
                if in_skip_dir {
                    return Ok(());
                }

                // Check if file has inline tests (#[cfg(test)] module)
                let content = std::fs::read_to_string(path)?;
                if content.contains("#[cfg(test)]") {
                    // File has inline tests, skip it
                    return Ok(());
                }

                // Check if this file or its parent module has a test
                let has_test = test_files.contains(file_name)
                    || test_files.contains(&format!("{file_name}_test"))
                    || test_files.contains(&format!("{file_name}_tests"));

                // For files in subdirectories, also check parent directory coverage
                let parent_covered = if relative.components().count() > 1 {
                    let parent_name = relative
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    test_files.contains(parent_name)
                        || test_dirs.contains(parent_name)
                        || test_files.contains(&format!("{parent_name}_test"))
                        || test_files.contains(&format!("{parent_name}_tests"))
                } else {
                    false
                };

                if !has_test && !parent_covered {
                    violations.push(RefactoringViolation::MissingTestFile {
                        source_file: path.to_path_buf(),
                        expected_test: tests_dir.join(format!("{file_name}_test.rs")),
                        severity: Severity::Warning, // Warning, not Error - tests are quality, not critical
                    });
                }

                Ok(())
            })?;
        }

        Ok(violations)
    }

    /// Check for mod declarations that reference non-existent files
    pub fn validate_mod_declarations(&self) -> Result<Vec<RefactoringViolation>> {
        let mut violations = Vec::new();
        let mod_pattern = Regex::new(r"(?:pub\s+)?mod\s+([a-z_][a-z0-9_]*)(?:\s*;)").unwrap();

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }

            for_each_scan_rs_path(&self.config, false, |path, candidate_src_dir| {
                if candidate_src_dir != src_dir {
                    return Ok(());
                }

                let parent_dir = path.parent().unwrap_or(Path::new("."));
                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(cap) = mod_pattern.captures(line) {
                        let mod_name = cap.get(1).map_or("", |m| m.as_str());

                        // Check if module file exists (Rust: same dir or parent_name/mod_name)
                        let mod_file = parent_dir.join(format!("{mod_name}.rs"));
                        let mod_dir = parent_dir.join(mod_name).join("mod.rs");
                        let module_subdir = path.file_stem().and_then(|s| s.to_str()).map(|stem| {
                            (
                                parent_dir.join(stem).join(format!("{mod_name}.rs")),
                                parent_dir.join(stem).join(mod_name).join("mod.rs"),
                            )
                        });

                        let exists = mod_file.exists()
                            || mod_dir.exists()
                            || module_subdir.is_some_and(|(f, d)| f.exists() || d.exists());

                        if !exists {
                            violations.push(RefactoringViolation::DeletedModuleReference {
                                referencing_file: path.to_path_buf(),
                                line: line_num + 1,
                                deleted_module: mod_name.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }

                Ok(())
            })?;
        }

        Ok(violations)
    }

    /// Get all crate directories
    fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
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
    fn should_skip_crate(&self, crate_dir: &std::path::Path) -> bool {
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
