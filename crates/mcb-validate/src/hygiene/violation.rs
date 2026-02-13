use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Severity;
use crate::violation_trait::{Violation, ViolationCategory};

/// Test hygiene violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HygieneViolation {
    /// Inline test module found in src/
    // TODO(TEST001): Move pre-existing inline tests (line 25) to tests/ directory.
    // This is part of the "Clean src/" initiative.
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
                // TODO(SOLID006): Avoid using stub macros in tests to ensure complete validation
                Some("Replace stub macros (such as todo! or unimplemented!) with real test implementation".to_string())
            }
        }
    }
}
