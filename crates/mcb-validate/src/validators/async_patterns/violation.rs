//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::PathBuf;

use crate::Severity;
use crate::define_violations;
use crate::traits::violation::ViolationCategory;

define_violations! {
    dynamic_severity,
    ViolationCategory::Async,
    pub enum AsyncViolation {
        /// Blocking call in async function
        #[violation(
            id = "ASYNC001",
            severity = Warning,
            message = "Blocking in async: {file}:{line} - {blocking_call} ({suggestion})",
            suggestion = "{suggestion}"
        )]
        BlockingInAsync {
            file: PathBuf,
            line: usize,
            blocking_call: String,
            suggestion: String,
            severity: Severity,
        },
        /// `block_on()` used in async context
        #[violation(
            id = "ASYNC002",
            severity = Warning,
            message = "block_on in async: {file}:{line} - {context}",
            suggestion = "Use .await instead of block_on() in async context"
        )]
        BlockOnInAsync {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
        /// `std::sync::Mutex` used in async code (should use `tokio::sync::Mutex`)
        #[violation(
            id = "ASYNC003",
            severity = Warning,
            message = "Wrong mutex type: {file}:{line} - {mutex_type} ({suggestion})",
            suggestion = "{suggestion}"
        )]
        WrongMutexType {
            file: PathBuf,
            line: usize,
            mutex_type: String,
            suggestion: String,
            severity: Severity,
        },
        /// Spawn without awaiting `JoinHandle`
        #[violation(
            id = "ASYNC004",
            severity = Info,
            message = "Unawaited spawn: {file}:{line} - {context}",
            suggestion = "Assign JoinHandle to a variable or use let _ = to explicitly ignore"
        )]
        UnawaitedSpawn {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
    }
}
