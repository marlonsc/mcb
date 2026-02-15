use crate::filters::LanguageId;
use crate::scan::for_each_file_under_root;
use crate::{Result, Severity};

use super::RefactoringValidator;
use super::violation::RefactoringViolation;

/// Check for source files without corresponding test files
pub fn validate_missing_test_files(
    validator: &RefactoringValidator,
) -> Result<Vec<RefactoringViolation>> {
    let mut violations = Vec::new();
    for crate_dir in validator.get_crate_dirs()? {
        let src_dir = crate_dir.join("src");
        let tests_dir = crate_dir.join("tests");

        if !src_dir.exists() {
            continue;
        }

        if validator.should_skip_crate(&crate_dir) {
            continue;
        }

        // If tests directory doesn't exist, skip this crate (no test infrastructure)
        if !tests_dir.exists() {
            continue;
        }

        // Collect existing test files and directories
        let mut test_files: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut test_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

        for_each_file_under_root(
            &validator.config,
            &tests_dir,
            Some(LanguageId::Rust),
            |entry| {
                let path = &entry.absolute_path;
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    test_files.insert(stem.to_owned());
                    if let Some(base) = stem.strip_suffix("_test") {
                        test_files.insert(base.to_owned());
                    }
                    if let Some(base) = stem.strip_suffix("_tests") {
                        test_files.insert(base.to_owned());
                    }
                }

                let mut parent = path.parent();
                while let Some(dir) = parent {
                    if !dir.starts_with(&tests_dir) {
                        break;
                    }
                    if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                        test_dirs.insert(name.to_owned());
                    }
                    parent = dir.parent();
                }

                Ok(())
            },
        )?;

        // Check each source file
        for_each_file_under_root(
            &validator.config,
            &src_dir,
            Some(LanguageId::Rust),
            |entry| {
                let path = &entry.absolute_path;
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                // Skip common files that don't need dedicated tests
                if validator.skip_files.contains(file_name) {
                    return Ok(());
                }

                // Get relative path for directory checks
                let relative = path.strip_prefix(&src_dir).unwrap_or(path);
                let Some(path_str) = relative.to_str() else {
                    return Ok(());
                };

                // Skip files in directories that are tested via integration tests
                let in_skip_dir = validator
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
                        source_file: path.clone(),
                        expected_test: tests_dir.join(format!("{file_name}_test.rs")),
                        severity: Severity::Warning, // Warning, not Error - tests are quality, not critical
                    });
                }

                Ok(())
            },
        )?;
    }

    Ok(violations)
}
