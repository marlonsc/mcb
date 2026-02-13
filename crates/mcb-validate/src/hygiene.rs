//! Test Organization Validation
//!
//! Validates test hygiene:
//! - No inline test modules in src/ (should be in tests/)
//! - Test file naming conventions
//! - Test function naming conventions

use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::pattern_registry::PATTERNS;
use crate::run_context::ValidationRunContext;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

/// Test hygiene violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HygieneViolation {
    /// Inline test module found in src/
    InlineTestModule {
        /// File containing the inline test module.
        file: PathBuf,
        /// Line number where the `mod tests` block or `#[cfg(test)]` starts.
        line: usize,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Test file with incorrect naming
    BadTestFileName {
        /// File with the non-compliant name.
        file: PathBuf,
        /// Description of the naming issue and suggested corrective action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Test function with incorrect naming
    BadTestFunctionName {
        /// File containing the incorrectly named test function.
        file: PathBuf,
        /// Line number where the test function is defined.
        line: usize,
        /// The current non-compliant name of the test function.
        function_name: String,
        /// The suggested compliant name for the test function.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Test without assertion
    TestWithoutAssertion {
        /// File containing the test function.
        file: PathBuf,
        /// Line number where the test function starts.
        line: usize,
        /// Name of the test function lacking assertions.
        function_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Trivial assertion that always passes
    TrivialAssertion {
        /// File containing the trivial assertion.
        file: PathBuf,
        /// Line number where the trivial assertion occurs.
        line: usize,
        /// Name of the test function containing the trivial assertion.
        function_name: String,
        /// The content of the trivial assertion (e.g., `assert!(true)`).
        assertion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Test only uses .unwrap() as assertion
    UnwrapOnlyAssertion {
        /// File containing the unwrap-only test.
        file: PathBuf,
        /// Line number of the test function.
        line: usize,
        /// Name of the test function that relies solely on `.unwrap()`.
        function_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Test body is only comments
    CommentOnlyTest {
        /// File containing the empty/comment-only test.
        file: PathBuf,
        /// Line number of the test function definition.
        line: usize,
        /// Name of the test function that contains no executable code.
        function_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Mock type usage in tests is forbidden
    MockTypeUsage {
        /// File containing the mock type usage.
        file: PathBuf,
        /// Line number where the usage appears.
        line: usize,
        /// Matched token (e.g., MockService).
        token: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Skip branch that bypasses real implementation setup
    SkipBranchUsage {
        /// File containing the skip branch.
        file: PathBuf,
        /// Line number where the skip branch starts.
        line: usize,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Stub macro in tests outside fixture paths
    StubMacroUsage {
        /// File containing the stub macro.
        file: PathBuf,
        /// Line number where macro appears.
        line: usize,
        /// Macro name (`todo`/`unimplemented`).
        macro_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
}

impl HygieneViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

impl std::fmt::Display for HygieneViolation {
    /// Formats the violation as a human-readable string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InlineTestModule { file, line, .. } => {
                write!(
                    f,
                    "Inline test module: {}:{} - move to tests/ directory",
                    file.display(),
                    line
                )
            }
            Self::BadTestFileName {
                file, suggestion, ..
            } => {
                write!(
                    f,
                    "Bad test file name: {} (use {})",
                    file.display(),
                    suggestion
                )
            }
            Self::BadTestFunctionName {
                file,
                line,
                function_name,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Bad test function name: {}:{} - {} (use {})",
                    file.display(),
                    line,
                    function_name,
                    suggestion
                )
            }
            Self::TestWithoutAssertion {
                file,
                line,
                function_name,
                ..
            } => {
                write!(
                    f,
                    "Test without assertion: {}:{} - {}",
                    file.display(),
                    line,
                    function_name
                )
            }
            Self::TrivialAssertion {
                file,
                line,
                function_name,
                assertion,
                ..
            } => {
                write!(
                    f,
                    "Trivial assertion: {}:{} - {} uses '{}' (always passes)",
                    file.display(),
                    line,
                    function_name,
                    assertion
                )
            }
            Self::UnwrapOnlyAssertion {
                file,
                line,
                function_name,
                ..
            } => {
                write!(
                    f,
                    "Unwrap-only test: {}:{} - {} has no real assertion, only .unwrap()",
                    file.display(),
                    line,
                    function_name
                )
            }
            Self::CommentOnlyTest {
                file,
                line,
                function_name,
                ..
            } => {
                write!(
                    f,
                    "Comment-only test: {}:{} - {} has no executable code",
                    file.display(),
                    line,
                    function_name
                )
            }
            Self::MockTypeUsage {
                file, line, token, ..
            } => {
                write!(
                    f,
                    "Mock type usage in tests: {}:{} - {}",
                    file.display(),
                    line,
                    token
                )
            }
            Self::SkipBranchUsage { file, line, .. } => {
                write!(
                    f,
                    "Skip branch in tests: {}:{} - remove skip path and execute real implementation",
                    file.display(),
                    line
                )
            }
            Self::StubMacroUsage {
                file,
                line,
                macro_name,
                ..
            } => {
                write!(
                    f,
                    "Stub macro in tests: {}:{} - {}!()",
                    file.display(),
                    line,
                    macro_name
                )
            }
        }
    }
}

impl Violation for HygieneViolation {
    /// Returns the unique violation identifier code.
    fn id(&self) -> &str {
        match self {
            Self::InlineTestModule { .. } => "TEST001",
            Self::BadTestFileName { .. } => "TEST002",
            Self::BadTestFunctionName { .. } => "TEST003",
            Self::TestWithoutAssertion { .. } => "TEST004",
            Self::TrivialAssertion { .. } => "TEST005",
            Self::UnwrapOnlyAssertion { .. } => "TEST006",
            Self::CommentOnlyTest { .. } => "TEST007",
            Self::MockTypeUsage { .. } => "TEST008",
            Self::SkipBranchUsage { .. } => "TEST009",
            Self::StubMacroUsage { .. } => "TEST010",
        }
    }

    /// Returns the category of the violation.
    fn category(&self) -> ViolationCategory {
        ViolationCategory::Testing
    }

    /// Returns the severity level of the violation.
    fn severity(&self) -> Severity {
        match self {
            Self::InlineTestModule { severity, .. }
            | Self::BadTestFileName { severity, .. }
            | Self::BadTestFunctionName { severity, .. }
            | Self::TestWithoutAssertion { severity, .. }
            | Self::TrivialAssertion { severity, .. }
            | Self::UnwrapOnlyAssertion { severity, .. }
            | Self::CommentOnlyTest { severity, .. }
            | Self::MockTypeUsage { severity, .. }
            | Self::SkipBranchUsage { severity, .. }
            | Self::StubMacroUsage { severity, .. } => *severity,
        }
    }

    /// Returns the file path associated with the violation, if any.
    fn file(&self) -> Option<&std::path::PathBuf> {
        match self {
            Self::InlineTestModule { file, .. }
            | Self::BadTestFileName { file, .. }
            | Self::BadTestFunctionName { file, .. }
            | Self::TestWithoutAssertion { file, .. }
            | Self::TrivialAssertion { file, .. }
            | Self::UnwrapOnlyAssertion { file, .. }
            | Self::CommentOnlyTest { file, .. }
            | Self::MockTypeUsage { file, .. }
            | Self::SkipBranchUsage { file, .. }
            | Self::StubMacroUsage { file, .. } => Some(file),
        }
    }

    /// Returns the line number associated with the violation, if any.
    fn line(&self) -> Option<usize> {
        match self {
            Self::BadTestFileName { .. } => None,
            Self::InlineTestModule { line, .. }
            | Self::BadTestFunctionName { line, .. }
            | Self::TestWithoutAssertion { line, .. }
            | Self::TrivialAssertion { line, .. }
            | Self::UnwrapOnlyAssertion { line, .. }
            | Self::CommentOnlyTest { line, .. }
            | Self::MockTypeUsage { line, .. }
            | Self::SkipBranchUsage { line, .. }
            | Self::StubMacroUsage { line, .. } => Some(*line),
        }
    }

    /// Returns a suggestion for fixing the violation, if available.
    fn suggestion(&self) -> Option<String> {
        match self {
            Self::InlineTestModule { .. } => {
                Some("Move test module to tests/ directory".to_string())
            }
            Self::BadTestFileName { suggestion, .. }
            | Self::BadTestFunctionName { suggestion, .. } => {
                Some(format!("Rename to {suggestion}"))
            }
            Self::TestWithoutAssertion { function_name, .. } => Some(format!(
                "Add assertion to {function_name} or use smoke test suffix"
            )),
            Self::TrivialAssertion { assertion, .. } => {
                Some(format!("Replace {assertion} with meaningful assertion"))
            }
            Self::UnwrapOnlyAssertion { .. } => Some(
                "Add explicit assert! or assert_eq! instead of relying on .unwrap()".to_string(),
            ),
            Self::CommentOnlyTest { .. } => {
                Some("Add executable test code or remove the test".to_string())
            }
            Self::MockTypeUsage { .. } => {
                Some("Replace mock type usage with real local implementations".to_string())
            }
            Self::SkipBranchUsage { .. } => Some(
                "Remove skip branches and initialize real provider/service in tests".to_string(),
            ),
            Self::StubMacroUsage { .. } => {
                Some("Replace todo!/unimplemented! with real test implementation".to_string())
            }
        }
    }
}

/// Validates test organization and quality across a codebase.
///
/// Checks for:
/// - Inline test modules in src/ (should be in tests/)
/// - Test file naming conventions
/// - Test function naming conventions
/// - Test quality (assertions, trivial tests, etc.)
pub struct HygieneValidator {
    config: ValidationConfig,
}

impl HygieneValidator {
    /// Create a new test validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Creates a validator with custom configuration for multi-directory support.
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Runs all test organization validations and returns violations found.
    pub fn validate_all(&self) -> Result<Vec<HygieneViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_no_inline_tests()?);
        violations.extend(self.validate_test_directory_structure()?);
        violations.extend(self.validate_test_naming()?);
        violations.extend(self.validate_test_function_naming()?);
        violations.extend(self.validate_test_quality()?);
        Ok(violations)
    }

    /// Verifies that no inline test declarations exist in src/ directories.
    pub fn validate_no_inline_tests(&self) -> Result<Vec<HygieneViolation>> {
        let mut violations = Vec::new();
        let cfg_test_pattern = PATTERNS.get("TEST001.cfg_test");
        let mod_tests_pattern = PATTERNS.get("TEST001.mod_tests");
        let test_attr_pattern = PATTERNS.get("TEST001.test_attr");
        let tokio_test_attr_pattern = PATTERNS.get("TEST001.tokio_test_attr");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            self.for_each_src_rs_path_in_crate(&crate_dir, |path| {
                if Self::is_fixture_path(path) {
                    return Ok(());
                }

                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();
                let mut last_cfg_test_line: Option<usize> = None;
                let mut has_inline_module_marker = false;

                for (line_num, line) in lines.iter().enumerate() {
                    if cfg_test_pattern.is_some_and(|p| p.is_match(line)) {
                        last_cfg_test_line = Some(line_num);
                        has_inline_module_marker = true;
                        violations.push(HygieneViolation::InlineTestModule {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            severity: Severity::Warning,
                        });
                        continue;
                    }

                    if mod_tests_pattern.is_some_and(|p| p.is_match(line)) {
                        if last_cfg_test_line.is_some_and(|cfg_line| line_num <= cfg_line + 5) {
                            continue;
                        }
                        has_inline_module_marker = true;
                        violations.push(HygieneViolation::InlineTestModule {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            severity: Severity::Warning,
                        });
                    }
                }

                if !has_inline_module_marker {
                    for (line_num, line) in lines.iter().enumerate() {
                        if test_attr_pattern.is_some_and(|p| p.is_match(line))
                            || tokio_test_attr_pattern.is_some_and(|p| p.is_match(line))
                        {
                            violations.push(HygieneViolation::InlineTestModule {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                severity: Severity::Warning,
                            });
                            break;
                        }
                    }
                }
                Ok(())
            })?;
        }

        Ok(violations)
    }

    /// Validates that tests are properly organized in subdirectories (unit/, integration/, e2e/).
    pub fn validate_test_directory_structure(&self) -> Result<Vec<HygieneViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let tests_dir = crate_dir.join("tests");
            if !tests_dir.exists() {
                continue;
            }

            // Check that at least unit/ or integration/ exists (e2e/ is optional)
            let unit_exists = tests_dir.join("unit").exists();
            let integration_exists = tests_dir.join("integration").exists();

            // Only flag if NEITHER unit/ nor integration/ exist and there are test files
            if !unit_exists && !integration_exists {
                let has_test_files = std::fs::read_dir(&tests_dir)
                    .map(|entries| {
                        entries.filter_map(std::result::Result::ok).any(|e| {
                            let path = e.path();
                            path.is_file()
                                && path.extension().and_then(|x| x.to_str()) == Some("rs")
                                && !matches!(
                                    path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                                    "lib.rs" | "mod.rs" | "test_utils.rs"
                                )
                        })
                    })
                    .unwrap_or(false);

                if has_test_files {
                    violations.push(HygieneViolation::BadTestFileName {
                        file: tests_dir.clone(),
                        suggestion: "Create tests/unit/ or tests/integration/ directory"
                            .to_string(),
                        severity: Severity::Warning,
                    });
                }
            }

            // Check for test files directly in tests/ directory (not in subdirs)
            for entry in std::fs::read_dir(&tests_dir)? {
                let entry = entry?;
                let path = entry.path();

                // Skip directories
                if path.is_dir() {
                    continue;
                }

                // Skip non-Rust files
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }

                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Skip allowed files in root tests directory
                // These are: lib.rs, mod.rs, test_utils.rs, and entry points for test subdirectories
                if matches!(
                    file_name,
                    "lib.rs" | "mod.rs" | "test_utils.rs" | "unit.rs" | "integration.rs" | "e2e.rs"
                ) {
                    continue;
                }

                // Any other .rs file directly in tests/ is a violation
                violations.push(HygieneViolation::BadTestFileName {
                    file: path,
                    suggestion: "Move to tests/unit/, tests/integration/, or tests/e2e/ directory"
                        .to_string(),
                    severity: Severity::Warning,
                });
            }
        }

        Ok(violations)
    }

    /// Checks test file naming conventions and directory structure compliance.
    pub fn validate_test_naming(&self) -> Result<Vec<HygieneViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let tests_dir = crate_dir.join("tests");
            if !tests_dir.exists() {
                continue;
            }

            // We don't require specific directories - tests can be organized
            // in any subdirectory structure as long as they have entry points

            self.for_each_test_rs_path_in_crate(&crate_dir, |path| {
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                // Skip lib.rs and mod.rs
                if file_name == "lib" || file_name == "mod" {
                    return Ok(());
                }

                // Skip test utility files (mocks, fixtures, helpers)
                let path_str = path.to_string_lossy();
                if path_str.contains("test_utils")
                    || file_name.contains("mock")
                    || file_name.contains("fixture")
                    || file_name.contains("helper")
                {
                    return Ok(());
                }

                // Check directory-based naming conventions
                let parent_dir = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                match parent_dir {
                    "unit" => {
                        // Unit tests must follow [module]_tests.rs pattern
                        if !file_name.ends_with("_tests") {
                            violations.push(HygieneViolation::BadTestFileName {
                                file: path.to_path_buf(),
                                suggestion: format!(
                                    "{file_name}_tests.rs (unit tests must end with _tests)"
                                ),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    "integration" => {
                        // Integration tests can be more flexible but should indicate their purpose
                        let is_valid_integration = file_name.contains("integration")
                            || file_name.contains("workflow")
                            || file_name.ends_with("_integration")
                            || file_name.ends_with("_workflow");

                        if !is_valid_integration {
                            violations.push(HygieneViolation::BadTestFileName {
                                file: path.to_path_buf(),
                                suggestion: format!("{file_name}_integration.rs or {file_name}_workflow.rs (integration tests should indicate their purpose)"),
                                severity: Severity::Info,
                            });
                        }
                    }
                    "e2e" => {
                        // E2E tests should clearly indicate they're end-to-end
                        let is_valid_e2e = file_name.contains("e2e")
                            || file_name.contains("end_to_end")
                            || file_name.starts_with("test_");

                        if !is_valid_e2e {
                            violations.push(HygieneViolation::BadTestFileName {
                                file: path.to_path_buf(),
                                suggestion: format!("{file_name}_e2e.rs or test_{file_name}.rs (e2e tests should indicate they're end-to-end)"),
                                severity: Severity::Info,
                            });
                        }
                    }
                    "tests" => {
                        // Files directly in tests/ directory (not in any subdirectory)
                        // are violations UNLESS they are entry points
                        let file_full = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if !matches!(
                            file_full,
                            "lib.rs"
                                | "mod.rs"
                                | "test_utils.rs"
                                | "unit.rs"
                                | "integration.rs"
                                | "e2e.rs"
                        ) {
                            violations.push(HygieneViolation::BadTestFileName {
                                file: path.to_path_buf(),
                                suggestion: "Move to a subdirectory (e.g., tests/unit/)"
                                    .to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    _ => {
                        // Files in subdirectories are allowed (module structure)
                        // No violation
                    }
                }
                Ok(())
            })?;
        }

        Ok(violations)
    }

    /// Verifies that test functions follow the `test_*` naming pattern.
    pub fn validate_test_function_naming(&self) -> Result<Vec<HygieneViolation>> {
        let mut violations = Vec::new();
        let test_attr_pattern = Regex::new(r"#\[test\]").unwrap();
        let tokio_test_pattern = Regex::new(r"#\[tokio::test\]").unwrap();
        let fn_pattern = Regex::new(r"(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(").unwrap();
        // Standard assertions plus implicit assertions
        let assert_pattern = Regex::new(
            r"assert!|assert_eq!|assert_ne!|panic!|should_panic|\.unwrap\(|\.expect\(|Box<dyn\s|type_name::",
        )
        .unwrap();

        // Smoke test patterns - these verify compilation, not runtime behavior
        let smoke_test_patterns = [
            "_trait_object", // Tests that verify trait object construction compiles
            "_exists",       // Tests that verify types exist
            "_creation",     // Constructor tests with implicit unwrap assertions
            "_compiles",     // Explicit compile-time tests
            "_factory",      // Factory pattern tests (often smoke tests)
        ];

        for crate_dir in self.get_crate_dirs()? {
            let tests_dir = crate_dir.join("tests");
            if !tests_dir.exists() {
                continue;
            }

            self.for_each_test_rs_path_in_crate(&crate_dir, |path| {
                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                let mut i = 0;
                while i < lines.len() {
                    let line = lines[i];

                    // Check for #[test] or #[tokio::test]
                    if test_attr_pattern.is_match(line) || tokio_test_pattern.is_match(line) {
                        // Find the function definition
                        let mut fn_line_idx = i + 1;
                        while fn_line_idx < lines.len() {
                            let potential_fn = lines[fn_line_idx];
                            if let Some(cap) = fn_pattern.captures(potential_fn) {
                                let fn_name = cap.get(1).map_or("", |m| m.as_str());

                                // Check naming convention - must start with test_
                                if !fn_name.starts_with("test_") {
                                    violations.push(HygieneViolation::BadTestFunctionName {
                                        file: path.to_path_buf(),
                                        line: fn_line_idx + 1,
                                        function_name: fn_name.to_string(),
                                        suggestion: format!("test_{fn_name}"),
                                        severity: Severity::Warning,
                                    });
                                }

                                // Check for assertions in the function body
                                let mut has_assertion = false;
                                let mut brace_depth = 0;
                                let mut started = false;

                                for check_line in &lines[fn_line_idx..] {
                                    if check_line.contains('{') {
                                        started = true;
                                    }
                                    if started {
                                        brace_depth +=
                                            check_line.chars().filter(|c| *c == '{').count();
                                        brace_depth -=
                                            check_line.chars().filter(|c| *c == '}').count();

                                        if assert_pattern.is_match(check_line) {
                                            has_assertion = true;
                                            break;
                                        }

                                        if brace_depth == 0 {
                                            break;
                                        }
                                    }
                                }

                                // Skip smoke tests - they verify compilation, not behavior
                                let is_smoke_test = smoke_test_patterns
                                    .iter()
                                    .any(|pattern| fn_name.ends_with(pattern));

                                if !has_assertion && !is_smoke_test {
                                    violations.push(HygieneViolation::TestWithoutAssertion {
                                        file: path.to_path_buf(),
                                        line: fn_line_idx + 1,
                                        function_name: fn_name.to_string(),
                                        severity: Severity::Warning,
                                    });
                                }

                                break;
                            }
                            fn_line_idx += 1;
                        }
                    }
                    i += 1;
                }
                Ok(())
            })?;
        }

        Ok(violations)
    }

    /// Validates test quality by checking for trivial assertions, unwrap-only tests, and comment-only tests.
    #[allow(clippy::too_many_lines)]
    pub fn validate_test_quality(&self) -> Result<Vec<HygieneViolation>> {
        let mut violations = Vec::new();

        let mock_type_pattern = Regex::new(r"\bMock[A-Za-z0-9_]+\b").ok();
        let skip_message_pattern = Regex::new(r"skipping:").ok();
        let todo_pattern = Regex::new(r"\btodo!\(").ok();
        let unimplemented_pattern = Regex::new(r"\bunimplemented!\(").ok();

        // Trivial assertion patterns
        let trivial_patterns = [
            (r"assert!\s*\(\s*true\s*\)", "assert!(true)"),
            (r"assert!\s*\(\s*!false\s*\)", "assert!(!false)"),
            (
                r"assert_eq!\s*\(\s*true\s*,\s*true\s*\)",
                "assert_eq!(true, true)",
            ),
            (r"assert_eq!\s*\(\s*1\s*,\s*1\s*\)", "assert_eq!(1, 1)"),
            (r"assert_ne!\s*\(\s*1\s*,\s*2\s*\)", "assert_ne!(1, 2)"),
            (
                r"assert_ne!\s*\(\s*true\s*,\s*false\s*\)",
                "assert_ne!(true, false)",
            ),
        ];

        let compiled_trivial: Vec<_> = trivial_patterns
            .iter()
            .filter_map(|(p, desc)| Regex::new(p).ok().map(|r| (r, *desc)))
            .collect();

        let test_attr_pattern = Regex::new(r"#\[(?:tokio::)?test\]").ok();
        let fn_pattern = Regex::new(r"(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(").ok();
        // Match common assertion macros - allow leading whitespace for indented code
        // The pattern checks for assertions at the start of a line (with optional whitespace)
        // or preceded by whitespace/punctuation to avoid false positives like "some_assert!"
        let real_assert_pattern = Regex::new(
            r"(?:^|\s)(assert!|assert_eq!|assert_ne!|assert_matches!|debug_assert!|debug_assert_eq!|debug_assert_ne!|panic!)",
        )
        .ok();
        let unwrap_pattern = Regex::new(r"\.unwrap\(|\.expect\(").ok();

        for crate_dir in self.get_crate_dirs()? {
            let tests_dir = crate_dir.join("tests");
            if !tests_dir.exists() {
                continue;
            }

            self.for_each_test_rs_path_in_crate(&crate_dir, |path| {
                if Self::is_fixture_path(path) {
                    return Ok(());
                }

                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                self.check_forbidden_patterns(
                    path,
                    &lines,
                    mock_type_pattern.as_ref(),
                    skip_message_pattern.as_ref(),
                    todo_pattern.as_ref(),
                    unimplemented_pattern.as_ref(),
                    &mut violations,
                );

                let mut i = 0;
                while i < lines.len() {
                    let line = lines[i];

                    // Skip module documentation comments (//!)
                    if line.trim().starts_with("//!") {
                        i += 1;
                        continue;
                    }

                    // Check for test attribute
                    let is_test_attr = test_attr_pattern.as_ref().is_some_and(|p| p.is_match(line));

                    if is_test_attr {
                        // Find the function definition
                        let mut fn_line_idx = i + 1;
                        while fn_line_idx < lines.len() {
                            let potential_fn = lines[fn_line_idx];
                            let fn_cap = fn_pattern.as_ref().and_then(|p| p.captures(potential_fn));

                            if let Some(cap) = fn_cap {
                                let fn_name = cap.get(1).map_or("", |m| m.as_str());
                                let fn_start = fn_line_idx;

                                // Collect function body
                                let mut body_lines: Vec<(usize, &str)> = Vec::new();
                                let mut brace_depth = 0;
                                let mut started = false;

                                for (idx, check_line) in lines.iter().enumerate().skip(fn_line_idx)
                                {
                                    if check_line.contains('{') {
                                        started = true;
                                    }
                                    if started {
                                        brace_depth += i32::try_from(
                                            check_line.chars().filter(|c| *c == '{').count(),
                                        )
                                        .unwrap_or(0);
                                        brace_depth -= i32::try_from(
                                            check_line.chars().filter(|c| *c == '}').count(),
                                        )
                                        .unwrap_or(0);
                                        body_lines.push((idx, check_line));
                                        if brace_depth <= 0 {
                                            break;
                                        }
                                    }
                                }

                                // Check for trivial assertions
                                for (line_idx, body_line) in &body_lines {
                                    for (pattern, desc) in &compiled_trivial {
                                        if pattern.is_match(body_line) {
                                            violations.push(HygieneViolation::TrivialAssertion {
                                                file: path.to_path_buf(),
                                                line: line_idx + 1,
                                                function_name: fn_name.to_string(),
                                                assertion: desc.to_string(),
                                                severity: Severity::Warning,
                                            });
                                        }
                                    }
                                }

                                // Check for unwrap-only tests (has unwrap but no real assertion)
                                let has_unwrap = unwrap_pattern
                                    .as_ref()
                                    .is_some_and(|p| body_lines.iter().any(|(_, l)| p.is_match(l)));
                                let has_real_assert = real_assert_pattern
                                    .as_ref()
                                    .is_some_and(|p| body_lines.iter().any(|(_, l)| p.is_match(l)));

                                if has_unwrap && !has_real_assert {
                                    // Use Warning severity since this is heuristic-based detection
                                    // Tests may have valid assertions that aren't detected by the pattern
                                    violations.push(HygieneViolation::UnwrapOnlyAssertion {
                                        file: path.to_path_buf(),
                                        line: fn_start + 1,
                                        function_name: fn_name.to_string(),
                                        severity: Severity::Warning,
                                    });
                                }

                                // Check for comment-only tests
                                let _meaningful_lines: Vec<_> = body_lines
                                    .iter()
                                    .filter(|(_, l)| {
                                        let trimmed = l.trim();
                                        !trimmed.is_empty()
                                            && !trimmed.starts_with("//")
                                            && !trimmed.starts_with('{')
                                            && !trimmed.starts_with('}')
                                            && trimmed != "{"
                                            && trimmed != "}"
                                    })
                                    .collect();

                                // If no meaningful lines (only comments/braces), it's comment-only
                                let all_comments = body_lines.iter().all(|(_, l)| {
                                    let trimmed = l.trim();
                                    trimmed.is_empty()
                                        || trimmed.starts_with("//")
                                        || trimmed == "{"
                                        || trimmed == "}"
                                        || trimmed.starts_with("fn ")
                                        || trimmed.starts_with("async fn ")
                                });

                                if all_comments && body_lines.len() > 2 {
                                    violations.push(HygieneViolation::CommentOnlyTest {
                                        file: path.to_path_buf(),
                                        line: fn_start + 1,
                                        function_name: fn_name.to_string(),
                                        severity: Severity::Warning,
                                    });
                                }

                                break;
                            }
                            fn_line_idx += 1;
                        }
                    }
                    i += 1;
                }
                Ok(())
            })?;
        }

        Ok(violations)
    }

    #[allow(clippy::too_many_arguments)]
    fn check_forbidden_patterns(
        &self,
        file: &std::path::Path,
        lines: &[&str],
        mock_type_pattern: Option<&Regex>,
        skip_message_pattern: Option<&Regex>,
        todo_pattern: Option<&Regex>,
        unimplemented_pattern: Option<&Regex>,
        violations: &mut Vec<HygieneViolation>,
    ) {
        for (idx, line) in lines.iter().enumerate() {
            let line_no = idx + 1;

            if let Some(pattern) = mock_type_pattern {
                for mat in pattern.find_iter(line) {
                    violations.push(HygieneViolation::MockTypeUsage {
                        file: file.to_path_buf(),
                        line: line_no,
                        token: mat.as_str().to_string(),
                        severity: Severity::Error,
                    });
                }
            }

            if let Some(pattern) = skip_message_pattern
                && pattern.is_match(line)
            {
                violations.push(HygieneViolation::SkipBranchUsage {
                    file: file.to_path_buf(),
                    line: line_no,
                    severity: Severity::Error,
                });
            }

            if let Some(pattern) = todo_pattern
                && pattern.is_match(line)
            {
                violations.push(HygieneViolation::StubMacroUsage {
                    file: file.to_path_buf(),
                    line: line_no,
                    macro_name: "todo".to_string(),
                    severity: Severity::Error,
                });
            }

            if let Some(pattern) = unimplemented_pattern
                && pattern.is_match(line)
            {
                violations.push(HygieneViolation::StubMacroUsage {
                    file: file.to_path_buf(),
                    line: line_no,
                    macro_name: "unimplemented".to_string(),
                    severity: Severity::Error,
                });
            }
        }

        for idx in 0..lines.len().saturating_sub(2) {
            let current = lines[idx].trim();
            let next = lines[idx + 1].trim();
            let next2 = lines[idx + 2].trim();
            if current.starts_with("let Ok(")
                && next.starts_with("else")
                && next2.contains("return;")
            {
                violations.push(HygieneViolation::SkipBranchUsage {
                    file: file.to_path_buf(),
                    line: idx + 1,
                    severity: Severity::Error,
                });
            }
        }
    }

    fn is_fixture_path(path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("/crates/mcb-validate/tests/fixtures/")
            || path_str.contains("/tests/fixtures/")
    }

    /// Retrieves the source directories to validate.
    fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
        self.config.get_source_dirs()
    }

    fn for_each_src_rs_path_in_crate<F>(&self, crate_dir: &Path, mut f: F) -> Result<()>
    where
        F: FnMut(&Path) -> Result<()>,
    {
        let src_dir = crate_dir.join("src");
        if !src_dir.exists() {
            return Ok(());
        }

        let context = ValidationRunContext::active_or_build(&self.config)?;
        for entry in context.file_inventory() {
            if entry.absolute_path.starts_with(&src_dir)
                && entry
                    .absolute_path
                    .extension()
                    .is_some_and(|ext| ext == "rs")
            {
                f(&entry.absolute_path)?;
            }
        }

        Ok(())
    }

    fn for_each_test_rs_path_in_crate<F>(&self, crate_dir: &Path, mut f: F) -> Result<()>
    where
        F: FnMut(&Path) -> Result<()>,
    {
        let tests_dir = crate_dir.join("tests");
        if !tests_dir.exists() {
            return Ok(());
        }

        let context = ValidationRunContext::active_or_build(&self.config)?;
        for entry in context.file_inventory() {
            if entry.absolute_path.starts_with(&tests_dir)
                && entry
                    .absolute_path
                    .extension()
                    .is_some_and(|ext| ext == "rs")
            {
                f(&entry.absolute_path)?;
            }
        }

        Ok(())
    }
}

impl_validator!(
    HygieneValidator,
    "hygiene",
    "Validates test hygiene and quality"
);
