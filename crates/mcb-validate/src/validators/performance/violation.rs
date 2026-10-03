//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::PathBuf;

use crate::McbScriptsSeverity;
use crate::define_violations;
use mcb_domain::ports::validation::ViolationCategory;

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
            severity: McbScriptsSeverity,
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
            severity: McbScriptsSeverity,
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
            severity: McbScriptsSeverity,
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
            severity: McbScriptsSeverity,
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
            severity: McbScriptsSeverity,
        },
    }
}
