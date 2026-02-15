use crate::pattern_registry::{compile_regex, compile_regexes};
use crate::{Result, Severity};

use super::PerformanceValidator;
use super::loops::scan_files_with_patterns_in_loops;
use super::violation::PerformanceViolation;

/// Detect .`clone()` calls inside loops.
pub fn validate_clone_in_loops(
    validator: &PerformanceValidator,
) -> Result<Vec<PerformanceViolation>> {
    let clone_pattern = compile_regex(r"\.clone\(\)")?;
    scan_files_with_patterns_in_loops(validator, &[clone_pattern], |file, line_num, line| {
        let trimmed = line.trim();
        if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
            return None;
        }
        if trimmed.starts_with("let ") {
            return None;
        }
        if trimmed.contains(".insert(") {
            return None;
        }
        if trimmed.contains(": ") && trimmed.ends_with(".clone(),") {
            return None;
        }
        Some(PerformanceViolation::CloneInLoop {
            file,
            line: line_num,
            context: line.trim().chars().take(80).collect(),
            suggestion: "Consider borrowing or moving instead of cloning".to_owned(),
            severity: Severity::Warning,
        })
    })
}

/// Detect `Vec::new()` or `String::new()` inside loops.
pub fn validate_allocation_in_loops(
    validator: &PerformanceValidator,
) -> Result<Vec<PerformanceViolation>> {
    let allocation_patterns = [
        r"Vec::new\(\)",
        r"Vec::with_capacity\(",
        r"String::new\(\)",
        r"String::with_capacity\(",
        r"HashMap::new\(\)",
        r"HashSet::new\(\)",
    ];

    let compiled_patterns = compile_regexes(allocation_patterns)?;

    scan_files_with_patterns_in_loops(validator, &compiled_patterns, |file, line_num, line| {
        let allocation_type = if line.contains("Vec::") {
            "Vec allocation"
        } else if line.contains("String::") {
            "String allocation"
        } else if line.contains("HashMap::") {
            "HashMap allocation"
        } else if line.contains("HashSet::") {
            "HashSet allocation"
        } else {
            "Allocation"
        };

        Some(PerformanceViolation::AllocationInLoop {
            file,
            line: line_num,
            allocation_type: allocation_type.to_owned(),
            suggestion: "Move allocation outside loop or reuse buffer".to_owned(),
            severity: Severity::Warning,
        })
    })
}
