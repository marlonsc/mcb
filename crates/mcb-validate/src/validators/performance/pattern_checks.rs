//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::constants::performance::{
    ARC_MUTEX_OVERUSE_PATTERNS, INEFFICIENT_ITERATOR_PATTERNS, INEFFICIENT_STRING_PATTERNS,
};
use crate::pattern_registry::compile_regex_triples;
use crate::{Result, Severity};

use super::PerformanceValidator;
use super::loops::scan_files_with_patterns;
use super::violation::PerformanceViolation;

#[derive(Clone, Copy)]
enum InfoViolationKind {
    ArcMutexOveruse,
    InefficientIterator,
    InefficientString,
}

fn build_info_violation(
    kind: InfoViolationKind,
    file: std::path::PathBuf,
    line: usize,
    pattern: &str,
    suggestion: &str,
) -> PerformanceViolation {
    match kind {
        InfoViolationKind::ArcMutexOveruse => PerformanceViolation::ArcMutexOveruse {
            file,
            line,
            pattern: pattern.to_owned(),
            suggestion: suggestion.to_owned(),
            severity: Severity::Info,
        },
        InfoViolationKind::InefficientIterator => PerformanceViolation::InefficientIterator {
            file,
            line,
            pattern: pattern.to_owned(),
            suggestion: suggestion.to_owned(),
            severity: Severity::Info,
        },
        InfoViolationKind::InefficientString => PerformanceViolation::InefficientString {
            file,
            line,
            pattern: pattern.to_owned(),
            suggestion: suggestion.to_owned(),
            severity: Severity::Info,
        },
    }
}

fn validate_patterns<F>(
    validator: &PerformanceValidator,
    patterns: &[(&str, &str, &str)],
    build_violation: F,
) -> Result<Vec<PerformanceViolation>>
where
    F: Fn(std::path::PathBuf, usize, &str, &str) -> PerformanceViolation,
{
    let compiled_patterns = compile_regex_triples(patterns)?;
    scan_files_with_patterns(validator, &compiled_patterns, build_violation)
}

/// Validate Arc/Mutex overuse patterns.
pub fn validate_arc_mutex_overuse(
    validator: &PerformanceValidator,
) -> Result<Vec<PerformanceViolation>> {
    validate_patterns(
        validator,
        ARC_MUTEX_OVERUSE_PATTERNS,
        |file, line, pattern, suggestion| {
            build_info_violation(
                InfoViolationKind::ArcMutexOveruse,
                file,
                line,
                pattern,
                suggestion,
            )
        },
    )
}

/// Validate inefficient iterator patterns.
pub fn validate_inefficient_iterators(
    validator: &PerformanceValidator,
) -> Result<Vec<PerformanceViolation>> {
    validate_patterns(
        validator,
        INEFFICIENT_ITERATOR_PATTERNS,
        |file, line, pattern, suggestion| {
            build_info_violation(
                InfoViolationKind::InefficientIterator,
                file,
                line,
                pattern,
                suggestion,
            )
        },
    )
}

/// Validate inefficient string patterns.
pub fn validate_inefficient_strings(
    validator: &PerformanceValidator,
) -> Result<Vec<PerformanceViolation>> {
    validate_patterns(
        validator,
        INEFFICIENT_STRING_PATTERNS,
        |file, line, pattern, suggestion| {
            build_info_violation(
                InfoViolationKind::InefficientString,
                file,
                line,
                pattern,
                suggestion,
            )
        },
    )
}
