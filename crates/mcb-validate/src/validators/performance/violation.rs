use std::path::PathBuf;

use crate::Severity;
use crate::traits::violation::{Violation, ViolationCategory};

define_violations! {
    dynamic_severity,
    ViolationCategory::Performance,
    pub enum PerformanceViolation {
        /// .`clone()` called inside a loop
        #[violation(
            id = "PERF001",
            severity = Warning,
            message = "Clone in loop: {file}:{line} - {context} ({suggestion})",
            suggestion = "{suggestion}"
        )]
        CloneInLoop {
            file: PathBuf,
            line: usize,
            context: String,
            suggestion: String,
            severity: Severity,
        },
        /// Vec/String allocation inside a loop
        #[violation(
            id = "PERF002",
            severity = Warning,
            message = "Allocation in loop: {file}:{line} - {allocation_type} ({suggestion})",
            suggestion = "{suggestion}"
        )]
        AllocationInLoop {
            file: PathBuf,
            line: usize,
            allocation_type: String,
            suggestion: String,
            severity: Severity,
        },
        /// `Arc<Mutex<T>>` where simpler patterns would work
        #[violation(
            id = "PERF003",
            severity = Info,
            message = "Arc/Mutex overuse: {file}:{line} - {pattern} ({suggestion})",
            suggestion = "{suggestion}"
        )]
        ArcMutexOveruse {
            file: PathBuf,
            line: usize,
            pattern: String,
            suggestion: String,
            severity: Severity,
        },
        /// Inefficient iterator pattern
        #[violation(
            id = "PERF004",
            severity = Info,
            message = "Inefficient iterator: {file}:{line} - {pattern} ({suggestion})",
            suggestion = "{suggestion}"
        )]
        InefficientIterator {
            file: PathBuf,
            line: usize,
            pattern: String,
            suggestion: String,
            severity: Severity,
        },
        /// Inefficient string handling
        #[violation(
            id = "PERF005",
            severity = Info,
            message = "Inefficient string: {file}:{line} - {pattern} ({suggestion})",
            suggestion = "{suggestion}"
        )]
        InefficientString {
            file: PathBuf,
            line: usize,
            pattern: String,
            suggestion: String,
            severity: Severity,
        },
    }
}

impl PerformanceViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}
