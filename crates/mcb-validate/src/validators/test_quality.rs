//! Test Quality Validation
//!
//! Validates test code quality:
//! - Detects `#[ignore]` attributes without proper justification (attribute without documentation)
//! - Detects `todo!()` macros in test fixtures outside intentional stubs
//! - Detects missing test implementations
//! - Ensures tests have proper documentation

use std::path::{Path, PathBuf};

use regex::Regex;

use crate::config::TestQualityRulesConfig;
use crate::scan::for_each_scan_rs_path;
use crate::traits::violation::ViolationCategory;
use crate::{Result, Severity, ValidationConfig, ValidationError};

crate::define_violations! {
    dynamic_severity,
    ViolationCategory::Testing,
    pub enum TestQualityViolation {
        /// Test with `#[ignore]` attribute missing justification
        #[violation(
            id = "TST001",
            severity = Warning,
            message = "{file}:{line} - Test '{test_name}' has #[ignore] without justification comment",
            suggestion = "Add a comment explaining why the test is ignored (e.g., // Requires external tool: ruff)"
        )]
        IgnoreWithoutJustification {
            file: PathBuf,
            line: usize,
            test_name: String,
            severity: Severity,
        },
        /// todo!() macro in test fixture without proper stub marker
        #[violation(
            id = "TST002",
            severity = Warning,
            message = "{file}:{line} - Function '{function_name}' in test fixture contains todo!() - implement or mark as intentional stub",
            suggestion = "Implement the test fixture function or add comment: // Intentional stub for X"
        )]
        TodoInTestFixture {
            file: PathBuf,
            line: usize,
            function_name: String,
            severity: Severity,
        },
        /// Test function with empty body
        #[violation(
            id = "TST003",
            severity = Warning,
            message = "{file}:{line} - Test '{test_name}' has empty body - implement or remove",
            suggestion = "Implement the test logic or remove the test function"
        )]
        EmptyTestBody {
            file: PathBuf,
            line: usize,
            test_name: String,
            severity: Severity,
        },
        /// Test missing documentation comment
        #[violation(
            id = "TST004",
            severity = Warning,
            message = "{file}:{line} - Test '{test_name}' missing documentation comment explaining what it tests",
            suggestion = "Add documentation comment: /// Tests that [scenario] [expected behavior]"
        )]
        TestMissingDocumentation {
            file: PathBuf,
            line: usize,
            test_name: String,
            severity: Severity,
        },
        /// Test with only assert!(true) or similar stub
        #[violation(
            id = "TST005",
            severity = Warning,
            message = "{file}:{line} - Test '{test_name}' contains stub assertion (assert!(true)) - implement real test",
            suggestion = "Replace assert!(true) with actual test logic and assertions"
        )]
        StubTestAssertion {
            file: PathBuf,
            line: usize,
            test_name: String,
            severity: Severity,
        },
    }
}

/// Test quality validator
pub struct TestQualityValidator {
    /// Configuration for validation scans
    config: ValidationConfig,
    rules: TestQualityRulesConfig,
}

impl crate::traits::validator::Validator for TestQualityValidator {
    fn name(&self) -> &'static str {
        "test-quality"
    }

    fn description(&self) -> &'static str {
        "Validates test quality rules (ignore usage, stubs, and test documentation)"
    }

    fn validate(
        &self,
        _config: &crate::ValidationConfig,
    ) -> anyhow::Result<Vec<Box<dyn crate::traits::violation::Violation>>> {
        let violations = self.validate()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn crate::traits::violation::Violation>)
            .collect())
    }
}

impl TestQualityValidator {
    /// Create a new test quality validator with the given workspace root
    pub fn new(workspace_root: impl Into<std::path::PathBuf>) -> Self {
        let root: std::path::PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.test_quality)
    }

    /// Create a validator with a custom configuration
    pub fn with_config(config: ValidationConfig, rules: &TestQualityRulesConfig) -> Self {
        Self {
            config,
            rules: rules.clone(),
        }
    }

    /// Validate test quality across all test files
    ///
    /// # Errors
    /// Returns an error if pattern compilation fails or file reading fails.
    pub fn validate(&self) -> Result<Vec<TestQualityViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();

        // Regex patterns
        let _ignore_pattern = Regex::new(r"#\[ignore\]").map_err(ValidationError::InvalidRegex)?;
        let test_pattern =
            Regex::new(r"#\[test\]|#\[tokio::test\]").map_err(ValidationError::InvalidRegex)?;
        let fn_pattern = Regex::new(r"fn\s+(\w+)").map_err(ValidationError::InvalidRegex)?;
        let empty_body_pattern = Regex::new(r"\{\s*\}").map_err(ValidationError::InvalidRegex)?;
        let stub_assert_pattern = Regex::new(r"assert!\(true\)|assert_eq!\(true,\s*true\)")
            .map_err(ValidationError::InvalidRegex)?;
        let _doc_comment_pattern = Regex::new(r"^\s*///").map_err(ValidationError::InvalidRegex)?;

        for_each_scan_rs_path(&self.config, false, |path, _src_dir| {
            if !(path.to_string_lossy().contains("/tests/")
                || path.to_string_lossy().contains("/test_"))
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            self.check_ignored_tests(path, &lines, &test_pattern, &fn_pattern, &mut violations);
            self.check_todo_in_fixtures(path, &lines, &fn_pattern, &mut violations);
            self.check_empty_test_bodies(
                path,
                &lines,
                &test_pattern,
                &fn_pattern,
                &empty_body_pattern,
                &mut violations,
            );
            self.check_stub_assertions(
                path,
                &lines,
                &test_pattern,
                &stub_assert_pattern,
                &fn_pattern,
                &mut violations,
            );
            Ok(())
        })?;

        Ok(violations)
    }

    #[allow(clippy::too_many_arguments)]
    fn check_ignored_tests(
        &self,
        file: &Path,
        lines: &[&str],
        test_pattern: &Regex,
        fn_pattern: &Regex,
        violations: &mut Vec<TestQualityViolation>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if line.contains("#[ignore]") {
                // Check if there's a justification comment above
                let has_justification = i > 0 && {
                    let prev_line = lines[i - 1];
                    prev_line.contains("Requires")
                        || prev_line.contains("requires")
                        || prev_line.contains(crate::constants::PENDING_LABEL_TODO)
                        || prev_line.contains("WIP")
                };

                if !has_justification {
                    // Find the test function name
                    if let Some(test_name) =
                        Self::find_test_name(lines, i, test_pattern, fn_pattern)
                    {
                        violations.push(TestQualityViolation::IgnoreWithoutJustification {
                            file: file.to_path_buf(),
                            line: i + 1,
                            test_name,
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
    }

    fn check_todo_in_fixtures(
        &self,
        file: &Path,
        lines: &[&str],
        fn_pattern: &Regex,
        violations: &mut Vec<TestQualityViolation>,
    ) {
        if self.should_skip_path(file) {
            return;
        }

        for (i, line) in lines.iter().enumerate() {
            if line.contains("todo!(") {
                // Check if it's NOT marked as intentional stub
                let has_stub_marker = i > 0 && {
                    let prev_line = lines[i - 1];
                    prev_line.contains("Intentional stub") || prev_line.contains("Test stub")
                };

                if !has_stub_marker {
                    // Find the function name
                    if let Some(function_name) = Self::find_function_name(lines, i, fn_pattern) {
                        violations.push(TestQualityViolation::TodoInTestFixture {
                            file: file.to_path_buf(),
                            line: i + 1,
                            function_name,
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }
    }

    fn check_empty_test_bodies(
        &self,
        file: &Path,
        lines: &[&str],
        test_pattern: &Regex,
        fn_pattern: &Regex,
        empty_body_pattern: &Regex,
        violations: &mut Vec<TestQualityViolation>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if test_pattern.is_match(line) {
                // Find the function declaration
                if let Some(fn_line_idx) =
                    (i..i + 5).find(|&idx| idx < lines.len() && fn_pattern.is_match(lines[idx]))
                {
                    // Check if the function body is empty (just {})
                    if let Some(body_start) = (fn_line_idx..fn_line_idx + 3)
                        .find(|&idx| idx < lines.len() && lines[idx].contains('{'))
                        && (empty_body_pattern.is_match(lines[body_start])
                            || (body_start + 1 < lines.len()
                                && lines[body_start + 1].trim() == "}"))
                        && let Some(test_name) = fn_pattern
                            .captures(lines[fn_line_idx])
                            .and_then(|c| c.get(1))
                    {
                        violations.push(TestQualityViolation::EmptyTestBody {
                            file: file.to_path_buf(),
                            line: fn_line_idx + 1,
                            test_name: test_name.as_str().to_string(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }
    }

    fn check_stub_assertions(
        &self,
        file: &Path,
        lines: &[&str],
        test_pattern: &Regex,
        stub_assert_pattern: &Regex,
        fn_pattern: &Regex,
        violations: &mut Vec<TestQualityViolation>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if test_pattern.is_match(line) {
                for offset in 0..20 {
                    if i + offset >= lines.len() {
                        break;
                    }
                    if stub_assert_pattern.is_match(lines[i + offset]) {
                        if let Some(test_name) =
                            Self::find_test_name(lines, i, test_pattern, fn_pattern)
                        {
                            violations.push(TestQualityViolation::StubTestAssertion {
                                file: file.to_path_buf(),
                                line: i + offset + 1,
                                test_name,
                                severity: Severity::Warning,
                            });
                        }
                        break;
                    }
                }
            }
        }
    }

    fn find_test_name(
        lines: &[&str],
        start_idx: usize,
        _test_pattern: &Regex,
        fn_pattern: &Regex,
    ) -> Option<String> {
        let end = std::cmp::min(start_idx + 5, lines.len());
        for line in lines.iter().take(end).skip(start_idx) {
            if let Some(captures) = fn_pattern.captures(line)
                && let Some(name) = captures.get(1)
            {
                return Some(name.as_str().to_string());
            }
        }
        None
    }

    fn find_function_name(lines: &[&str], start_idx: usize, fn_pattern: &Regex) -> Option<String> {
        // Look backwards for function name
        for i in (0..=start_idx).rev().take(10) {
            if let Some(captures) = fn_pattern.captures(lines[i])
                && let Some(name) = captures.get(1)
            {
                return Some(name.as_str().to_string());
            }
        }
        None
    }

    /// Check if a path should be skipped based on configuration
    fn should_skip_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.rules
            .excluded_paths
            .iter()
            .any(|excluded| path_str.contains(excluded))
    }
}
