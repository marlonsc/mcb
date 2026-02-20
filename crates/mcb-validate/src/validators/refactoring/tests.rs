//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#refactoring)
//!
use crate::constants::common::CFG_TEST_MARKER;
use crate::filters::LanguageId;
use crate::scan::for_each_file_under_root;
use crate::{Result, Severity};
use std::collections::HashSet;
use std::path::Path;

use super::RefactoringValidator;
use super::violation::RefactoringViolation;

fn insert_test_keys(test_files: &mut HashSet<String>, stem: &str) {
    test_files.insert(stem.to_owned());
    test_files.insert(format!("{stem}_test"));
    test_files.insert(format!("{stem}_tests"));
    for suffix in ["_test", "_tests"] {
        if let Some(base) = stem.strip_suffix(suffix) {
            test_files.insert(base.to_owned());
        }
    }
}

fn has_test_key(test_files: &HashSet<String>, stem: &str) -> bool {
    [stem, &format!("{stem}_test"), &format!("{stem}_tests")]
        .iter()
        .any(|name| test_files.contains(*name))
}

fn has_test_coverage(
    relative: &Path,
    test_files: &HashSet<String>,
    test_dirs: &HashSet<String>,
) -> bool {
    let file_name = relative.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if has_test_key(test_files, file_name) {
        return true;
    }

    let parent_name = relative
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("");

    relative.components().count() > 1
        && (test_dirs.contains(parent_name) || has_test_key(test_files, parent_name))
}

fn collect_test_index(
    config: &crate::ValidationConfig,
    tests_dir: &Path,
) -> Result<(HashSet<String>, HashSet<String>)> {
    let mut test_files: HashSet<String> = HashSet::new();
    let mut test_dirs: HashSet<String> = HashSet::new();

    for_each_file_under_root(config, tests_dir, Some(LanguageId::Rust), |entry| {
        let path = &entry.absolute_path;
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            insert_test_keys(&mut test_files, stem);
        }

        for dir in path
            .parent()
            .into_iter()
            .flat_map(std::path::Path::ancestors)
            .take_while(|dir| dir.starts_with(tests_dir))
        {
            if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
                test_dirs.insert(name.to_owned());
            }
        }

        Ok(())
    })?;

    Ok((test_files, test_dirs))
}

/// Check for source files without corresponding test files
pub fn validate_missing_test_files(
    validator: &RefactoringValidator,
) -> Result<Vec<RefactoringViolation>> {
    let mut violations = Vec::new();

    for crate_dir in validator.get_crate_dirs()? {
        let src_dir = crate_dir.join("src");
        let tests_dir = crate_dir.join("tests");

        if !src_dir.exists() || !tests_dir.exists() || validator.should_skip_crate(&crate_dir) {
            continue;
        }

        let (test_files, test_dirs) = collect_test_index(&validator.config, &tests_dir)?;

        for_each_file_under_root(
            &validator.config,
            &src_dir,
            Some(LanguageId::Rust),
            |entry| {
                let path = &entry.absolute_path;
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let relative = path.strip_prefix(&src_dir).unwrap_or(path);
                let Some(path_str) = relative.to_str() else {
                    return Ok(());
                };

                if validator.skip_files.contains(file_name)
                    || validator
                        .skip_dir_patterns
                        .iter()
                        .any(|pattern| path_str.contains(pattern))
                {
                    return Ok(());
                }

                let content = std::fs::read_to_string(path)?;
                if content.contains(CFG_TEST_MARKER) {
                    return Ok(());
                }

                if !has_test_coverage(relative, &test_files, &test_dirs) {
                    violations.push(RefactoringViolation::MissingTestFile {
                        source_file: path.clone(),
                        expected_test: tests_dir.join(format!("{file_name}_test.rs")),
                        severity: Severity::Warning,
                    });
                }

                Ok(())
            },
        )?;
    }

    Ok(violations)
}
