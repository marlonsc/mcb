use crate::pattern_registry::compile_regex_triples;
use crate::{Result, Severity};

use super::PerformanceValidator;
use super::loops::scan_files_with_patterns;
use super::violation::PerformanceViolation;

/// Detect Arc/Mutex overuse patterns.
pub fn validate_arc_mutex_overuse(
    validator: &PerformanceValidator,
) -> Result<Vec<PerformanceViolation>> {
    let overuse_patterns = [
        (r"Arc<Arc<", "Nested Arc<Arc<>>", "Use single Arc instead"),
        (r"Mutex<bool>", "Mutex<bool>", "Use AtomicBool instead"),
        (r"Mutex<usize>", "Mutex<usize>", "Use AtomicUsize instead"),
        (r"Mutex<u32>", "Mutex<u32>", "Use AtomicU32 instead"),
        (r"Mutex<u64>", "Mutex<u64>", "Use AtomicU64 instead"),
        (r"Mutex<i32>", "Mutex<i32>", "Use AtomicI32 instead"),
        (r"Mutex<i64>", "Mutex<i64>", "Use AtomicI64 instead"),
        (r"RwLock<bool>", "RwLock<bool>", "Use AtomicBool instead"),
    ];

    let compiled_patterns = compile_regex_triples(&overuse_patterns)?;

    scan_files_with_patterns(
        validator,
        &compiled_patterns,
        |file, line, pattern, suggestion| PerformanceViolation::ArcMutexOveruse {
            file,
            line,
            pattern: pattern.to_string(),
            suggestion: suggestion.to_string(),
            severity: Severity::Info,
        },
    )
}

/// Detect inefficient iterator patterns.
pub fn validate_inefficient_iterators(
    validator: &PerformanceValidator,
) -> Result<Vec<PerformanceViolation>> {
    let inefficient_patterns = [
        (
            r"\.iter\(\)\.cloned\(\)\.take\(",
            ".iter().cloned().take()",
            "Use .iter().take().cloned() instead",
        ),
        (
            r"\.iter\(\)\.cloned\(\)\.last\(",
            ".iter().cloned().last()",
            "Use .iter().last().cloned() instead",
        ),
        (
            r#"\.collect::<Vec<String>>\(\)\.join\(\s*""\s*\)"#,
            r#".collect::<Vec<String>>().join("")"#,
            "Use .collect::<String>() instead",
        ),
        (
            r"\.repeat\(1\)",
            ".repeat(1)",
            "Use .clone() instead of .repeat(1)",
        ),
    ];

    let compiled_patterns = compile_regex_triples(&inefficient_patterns)?;

    scan_files_with_patterns(
        validator,
        &compiled_patterns,
        |file, line, pattern, suggestion| PerformanceViolation::InefficientIterator {
            file,
            line,
            pattern: pattern.to_string(),
            suggestion: suggestion.to_string(),
            severity: Severity::Info,
        },
    )
}

/// Detect inefficient string handling patterns.
pub fn validate_inefficient_strings(
    validator: &PerformanceValidator,
) -> Result<Vec<PerformanceViolation>> {
    let inefficient_patterns = [
        (
            r#"format!\s*\(\s*"\{\}"\s*,\s*\w+\s*\)"#,
            "format!(\"{}\", var)",
            "Use var.to_string() or &var instead",
        ),
        (
            r"\.to_string\(\)\.to_string\(\)",
            ".to_string().to_string()",
            "Remove redundant .to_string()",
        ),
        (
            r"\.to_owned\(\)\.to_owned\(\)",
            ".to_owned().to_owned()",
            "Remove redundant .to_owned()",
        ),
    ];

    let compiled_patterns = compile_regex_triples(&inefficient_patterns)?;

    scan_files_with_patterns(
        validator,
        &compiled_patterns,
        |file, line, pattern, suggestion| PerformanceViolation::InefficientString {
            file,
            line,
            pattern: pattern.to_string(),
            suggestion: suggestion.to_string(),
            severity: Severity::Info,
        },
    )
}
