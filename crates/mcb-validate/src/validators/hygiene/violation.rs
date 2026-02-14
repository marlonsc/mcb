use std::path::PathBuf;

use crate::Severity;
use crate::traits::violation::ViolationCategory;

crate::define_violations! {
    ViolationCategory::Testing,
    pub enum HygieneViolation {
        /// Inline test module found in src/
        // TODO(TEST001): Move pre-existing inline tests (line 25) to tests/ directory.
        // This is part of the "Clean src/" initiative.
        #[violation(
            id = "TEST001",
            severity = Warning,
            message = "Inline test module: {file}:{line} - move to tests/ directory",
            suggestion = "Move test module to tests/ directory"
        )]
        InlineTestModule {
            file: PathBuf,
            line: usize,
            severity: Severity,
        },
        /// Test file with incorrect naming
        #[violation(
            id = "TEST002",
            severity = Warning,
            message = "Bad test file name: {file} (use {suggestion})",
            suggestion = "Rename to {suggestion}"
        )]
        BadTestFileName {
            file: PathBuf,
            suggestion: String,
            severity: Severity,
        },
        /// Test function with incorrect naming
        #[violation(
            id = "TEST003",
            severity = Warning,
            message = "Bad test function name: {file}:{line} - {function_name} (use {suggestion})",
            suggestion = "Rename to {suggestion}"
        )]
        BadTestFunctionName {
            file: PathBuf,
            line: usize,
            function_name: String,
            suggestion: String,
            severity: Severity,
        },
        /// Test without assertion
        #[violation(
            id = "TEST004",
            severity = Warning,
            message = "Test without assertion: {file}:{line} - {function_name}",
            suggestion = "Add assertion to {function_name} or use smoke test suffix"
        )]
        TestWithoutAssertion {
            file: PathBuf,
            line: usize,
            function_name: String,
            severity: Severity,
        },
        /// Trivial assertion that always passes
        #[violation(
            id = "TEST005",
            severity = Warning,
            message = "Trivial assertion: {file}:{line} - {function_name} uses '{assertion}' (always passes)",
            suggestion = "Replace {assertion} with meaningful assertion"
        )]
        TrivialAssertion {
            file: PathBuf,
            line: usize,
            function_name: String,
            assertion: String,
            severity: Severity,
        },
        /// Test only uses .unwrap() as assertion
        #[violation(
            id = "TEST006",
            severity = Warning,
            message = "Unwrap-only test: {file}:{line} - {function_name} has no real assertion, only .unwrap()",
            suggestion = "Add explicit assert! or assert_eq! instead of relying on .unwrap()"
        )]
        UnwrapOnlyAssertion {
            file: PathBuf,
            line: usize,
            function_name: String,
            severity: Severity,
        },
        /// Test body is only comments
        #[violation(
            id = "TEST007",
            severity = Warning,
            message = "Comment-only test: {file}:{line} - {function_name} has no executable code",
            suggestion = "Add executable test code or remove the test"
        )]
        CommentOnlyTest {
            file: PathBuf,
            line: usize,
            function_name: String,
            severity: Severity,
        },
        /// Mock type usage in tests is forbidden
        #[violation(
            id = "TEST008",
            severity = Warning,
            message = "Mock type usage in tests: {file}:{line} - {token}",
            suggestion = "Replace mock type usage with real local implementations"
        )]
        MockTypeUsage {
            file: PathBuf,
            line: usize,
            token: String,
            severity: Severity,
        },
        /// Skip branch that bypasses real implementation setup
        #[violation(
            id = "TEST009",
            severity = Warning,
            message = "Skip branch in tests: {file}:{line} - remove skip path and execute real implementation",
            suggestion = "Remove skip branches and initialize real provider/service in tests"
        )]
        SkipBranchUsage {
            file: PathBuf,
            line: usize,
            severity: Severity,
        },
        /// Stub macro in tests outside fixture paths
        #[violation(
            id = "TEST010",
            severity = Warning,
            message = "Stub macro in tests: {file}:{line} - {macro_name}!()",
            suggestion = "Replace stub macros (such as todo! or unimplemented!) with real test implementation"
        )]
        StubMacroUsage {
            file: PathBuf,
            line: usize,
            macro_name: String,
            severity: Severity,
        },
    }
}
