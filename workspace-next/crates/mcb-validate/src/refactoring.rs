//! Refactoring Completeness Validation
//!
//! Validates that refactorings are complete and not left halfway:
//! - Orphan imports (use statements pointing to deleted/moved modules)
//! - Duplicate definitions (same type in multiple locations)
//! - Missing test files for new source files
//! - Stale re-exports (pub use of items that don't exist)
//! - Deleted module references
//! - Dead code from refactoring

use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Refactoring completeness violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringViolation {
    /// Import statement referencing non-existent module/item
    OrphanImport {
        file: PathBuf,
        line: usize,
        import_path: String,
        suggestion: String,
        severity: Severity,
    },

    /// Same type name defined in multiple locations (incomplete migration)
    DuplicateDefinition {
        type_name: String,
        locations: Vec<PathBuf>,
        suggestion: String,
        severity: Severity,
    },

    /// New source file without corresponding test file
    MissingTestFile {
        source_file: PathBuf,
        expected_test: PathBuf,
        severity: Severity,
    },

    /// pub use/mod statement for item that doesn't exist
    StaleReExport {
        file: PathBuf,
        line: usize,
        re_export: String,
        severity: Severity,
    },

    /// File/module that was deleted but is still referenced
    DeletedModuleReference {
        referencing_file: PathBuf,
        line: usize,
        deleted_module: String,
        severity: Severity,
    },

    /// Dead code left from refactoring (unused after move)
    RefactoringDeadCode {
        file: PathBuf,
        item_name: String,
        item_type: String,
        severity: Severity,
    },
}

impl RefactoringViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::OrphanImport { severity, .. } => *severity,
            Self::DuplicateDefinition { severity, .. } => *severity,
            Self::MissingTestFile { severity, .. } => *severity,
            Self::StaleReExport { severity, .. } => *severity,
            Self::DeletedModuleReference { severity, .. } => *severity,
            Self::RefactoringDeadCode { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for RefactoringViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OrphanImport {
                file,
                line,
                import_path,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Orphan import at {}:{} - '{}' - {}",
                    file.display(),
                    line,
                    import_path,
                    suggestion
                )
            }
            Self::DuplicateDefinition {
                type_name,
                locations,
                suggestion,
                ..
            } => {
                let locs: Vec<String> = locations.iter().map(|p| p.display().to_string()).collect();
                write!(
                    f,
                    "Duplicate definition '{}' in {} locations: [{}] - {}",
                    type_name,
                    locations.len(),
                    locs.join(", "),
                    suggestion
                )
            }
            Self::MissingTestFile {
                source_file,
                expected_test,
                ..
            } => {
                write!(
                    f,
                    "Missing test file for {} - expected {}",
                    source_file.display(),
                    expected_test.display()
                )
            }
            Self::StaleReExport {
                file,
                line,
                re_export,
                ..
            } => {
                write!(
                    f,
                    "Stale re-export at {}:{} - '{}'",
                    file.display(),
                    line,
                    re_export
                )
            }
            Self::DeletedModuleReference {
                referencing_file,
                line,
                deleted_module,
                ..
            } => {
                write!(
                    f,
                    "Reference to deleted module at {}:{} - '{}'",
                    referencing_file.display(),
                    line,
                    deleted_module
                )
            }
            Self::RefactoringDeadCode {
                file,
                item_name,
                item_type,
                ..
            } => {
                write!(
                    f,
                    "Dead {} '{}' from refactoring at {}",
                    item_type,
                    item_name,
                    file.display()
                )
            }
        }
    }
}

/// Refactoring completeness validator
pub struct RefactoringValidator {
    config: ValidationConfig,
}

impl RefactoringValidator {
    /// Create a new refactoring validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all refactoring validations
    pub fn validate_all(&self) -> Result<Vec<RefactoringViolation>> {
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
                .expect("Invalid regex");

        // Map: type_name -> Vec<file_path>
        let mut definitions: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate itself
            if src_dir.to_string_lossy().contains("mcb-validate") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Skip test files
                if path_str.contains("/tests/") || path_str.contains("_test.rs") {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;

                for cap in definition_pattern.captures_iter(&content) {
                    let type_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                    // Skip common generic names that are expected to be duplicated
                    if type_name == "Error"
                        || type_name == "Result"
                        || type_name == "Config"
                        || type_name == "Builder"
                    {
                        continue;
                    }

                    definitions
                        .entry(type_name.to_string())
                        .or_default()
                        .push(path.to_path_buf());
                }
            }
        }

        // Find duplicates (same name in different files)
        for (type_name, locations) in definitions {
            if locations.len() > 1 {
                // Check if duplicates are in different crates (more serious)
                let crates: std::collections::HashSet<String> = locations
                    .iter()
                    .filter_map(|p| {
                        p.to_string_lossy()
                            .split("/crates/")
                            .nth(1)
                            .and_then(|s| s.split('/').next())
                            .map(|s| s.to_string())
                    })
                    .collect();

                if crates.len() > 1 {
                    // Cross-crate duplicate - more serious
                    violations.push(RefactoringViolation::DuplicateDefinition {
                        type_name: type_name.clone(),
                        locations: locations.clone(),
                        suggestion: format!(
                            "Type '{}' is defined in multiple crates: {:?}. Consider consolidating to one location.",
                            type_name, crates
                        ),
                        severity: Severity::Error,
                    });
                } else if locations.len() > 2 {
                    // Same crate but many duplicates
                    violations.push(RefactoringViolation::DuplicateDefinition {
                        type_name: type_name.clone(),
                        locations: locations.clone(),
                        suggestion: format!(
                            "Type '{}' is defined {} times. This may indicate incomplete migration.",
                            type_name,
                            locations.len()
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Check for source files without corresponding test files
    pub fn validate_missing_test_files(&self) -> Result<Vec<RefactoringViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            let tests_dir = crate_dir.join("tests");

            if !src_dir.exists() || !tests_dir.exists() {
                continue;
            }

            // Collect existing test files
            let test_files: std::collections::HashSet<String> = WalkDir::new(&tests_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                .filter_map(|e| {
                    e.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .collect();

            // Check each source file
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let file_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                // Skip mod.rs, lib.rs, main.rs - these are aggregators
                if file_name == "mod" || file_name == "lib" || file_name == "main" {
                    continue;
                }

                // Skip files in subdirectories (they use mod.rs pattern)
                let relative = path.strip_prefix(&src_dir).unwrap_or(path);
                if relative.components().count() > 1 {
                    // This is a file in a subdirectory, check if the parent has a test
                    let parent_name = relative
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|s| s.to_str())
                        .unwrap_or("");

                    let expected_test = format!("{}_test", parent_name);
                    let expected_test_alt = format!("{}_tests", parent_name);

                    if !test_files.contains(&expected_test)
                        && !test_files.contains(&expected_test_alt)
                        && !test_files.contains(parent_name)
                    {
                        // Check if there's a directory with tests
                        let test_subdir = tests_dir.join(parent_name);
                        if !test_subdir.exists() {
                            violations.push(RefactoringViolation::MissingTestFile {
                                source_file: path.to_path_buf(),
                                expected_test: tests_dir.join(format!("{}_test.rs", parent_name)),
                                severity: Severity::Error,
                            });
                        }
                    }
                } else {
                    // Top-level file
                    let expected_test = format!("{}_test", file_name);
                    let expected_test_alt = format!("{}_tests", file_name);

                    if !test_files.contains(&expected_test)
                        && !test_files.contains(&expected_test_alt)
                        && !test_files.contains(file_name)
                    {
                        violations.push(RefactoringViolation::MissingTestFile {
                            source_file: path.to_path_buf(),
                            expected_test: tests_dir.join(format!("{}_test.rs", file_name)),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for mod declarations that reference non-existent files
    pub fn validate_mod_declarations(&self) -> Result<Vec<RefactoringViolation>> {
        let mut violations = Vec::new();
        let mod_pattern = Regex::new(r"(?:pub\s+)?mod\s+([a-z_][a-z0-9_]*)(?:\s*;)").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate itself
            if src_dir.to_string_lossy().contains("mcb-validate") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let parent_dir = path.parent().unwrap_or(Path::new("."));
                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(cap) = mod_pattern.captures(line) {
                        let mod_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Check if module file exists
                        let mod_file = parent_dir.join(format!("{}.rs", mod_name));
                        let mod_dir = parent_dir.join(mod_name).join("mod.rs");

                        if !mod_file.exists() && !mod_dir.exists() {
                            violations.push(RefactoringViolation::DeletedModuleReference {
                                referencing_file: path.to_path_buf(),
                                line: line_num + 1,
                                deleted_module: mod_name.to_string(),
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }
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

                // Skip mcb-validate
                if path
                    .file_name()
                    .is_some_and(|n| n == "mcb-validate" || n == "mcb")
                {
                    continue;
                }

                if path.is_dir() {
                    dirs.push(path);
                }
            }
        }

        Ok(dirs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate(temp: &TempDir, name: &str, content: &str) {
        let crate_dir = temp.path().join("crates").join(name).join("src");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(crate_dir.join("lib.rs"), content).unwrap();

        let cargo_dir = temp.path().join("crates").join(name);
        fs::write(
            cargo_dir.join("Cargo.toml"),
            format!(
                r#"
[package]
name = "{}"
version = "0.1.0"
"#,
                name
            ),
        )
        .unwrap();

        // Create tests directory
        let tests_dir = temp.path().join("crates").join(name).join("tests");
        fs::create_dir_all(&tests_dir).unwrap();
    }

    #[test]
    fn test_duplicate_definition_detection() {
        let temp = TempDir::new().unwrap();

        // Create first crate with MyService
        create_test_crate(
            &temp,
            "mcb-domain",
            r#"
pub struct MyService {
    pub name: String,
}
"#,
        );

        // Create second crate with same MyService
        create_test_crate(
            &temp,
            "mcb-server",
            r#"
pub struct MyService {
    pub id: u64,
}
"#,
        );

        let validator = RefactoringValidator::new(temp.path());
        let violations = validator.validate_duplicate_definitions().unwrap();

        assert!(!violations.is_empty(), "Should detect duplicate MyService");
    }

    #[test]
    fn test_missing_module_reference() {
        let temp = TempDir::new().unwrap();

        // Create crate with reference to non-existent module
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub mod existing;
pub mod deleted_module;  // This module doesn't exist
"#,
        );

        // Create existing.rs
        let src_dir = temp.path().join("crates").join("mcb-test").join("src");
        fs::write(src_dir.join("existing.rs"), "// exists").unwrap();

        let validator = RefactoringValidator::new(temp.path());
        let violations = validator.validate_mod_declarations().unwrap();

        assert_eq!(violations.len(), 1, "Should detect missing deleted_module");
        match &violations[0] {
            RefactoringViolation::DeletedModuleReference { deleted_module, .. } => {
                assert_eq!(deleted_module, "deleted_module");
            }
            _ => panic!("Expected DeletedModuleReference violation"),
        }
    }

    #[test]
    fn test_no_false_positives_for_inline_mods() {
        let temp = TempDir::new().unwrap();

        // Create crate with inline module (not a reference to file)
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub mod inline {
    pub fn hello() {}
}
"#,
        );

        let validator = RefactoringValidator::new(temp.path());
        let violations = validator.validate_mod_declarations().unwrap();

        assert!(violations.is_empty(), "Inline modules should not trigger violations");
    }
}
