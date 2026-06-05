//! Generic test assertion helpers.
//!
//! Centralized in `mcb-domain` so all crates share the same assertion patterns
//! and produce consistent error messages.

use std::fmt::Write;

/// Normalizes a path-like string to forward slashes with empty segments removed.
fn normalize_path(s: &str) -> String {
    s.replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

/// Returns true when a normalized debug string matches an expected entry.
fn debug_matches_expected(debug: &str, expected: &(&str, usize, &str)) -> bool {
    let (file_suffix, line, msg_contains) = expected;
    debug.contains(&normalize_path(file_suffix))
        && (*line == 0 || debug.contains(&format!("line: {line}")))
        && debug.contains(msg_contains)
}

/// Builds the failure message for [`assert_violations_exact`].
fn format_violations_failure(
    context: &str,
    expected_len: usize,
    debug_strs: &[String],
    missing: &[String],
    extras: &[String],
) -> String {
    let mut msg = format!(
        "{context}: expected {expected_len} violations, got {}\n",
        debug_strs.len()
    );
    if !missing.is_empty() {
        let _ = write!(
            msg,
            "MISSING ({}):\n{}\n",
            missing.len(),
            missing.join("\n")
        );
    }
    if !extras.is_empty() {
        let _ = write!(
            msg,
            "UNEXPECTED ({}):\n{}\n",
            extras.len(),
            extras.join("\n")
        );
    }
    let _ = write!(msg, "ALL VIOLATIONS:\n{}", debug_strs.join("\n"));
    msg
}

/// Assert that violations list is empty with descriptive message.
///
/// # Panics
/// Panics if the violations list is not empty.
pub fn assert_no_violations<V: std::fmt::Debug>(violations: &[V], context: &str) {
    assert!(
        violations.is_empty(),
        "{}: expected no violations, got {} - {:?}",
        context,
        violations.len(),
        violations
    );
}

/// Asserts that at least one violation in the list satisfies the predicate.
///
/// # Panics
/// Panics if no violation satisfies the predicate.
pub fn assert_has_violation_matching<V: std::fmt::Debug>(
    violations: &[V],
    predicate: impl Fn(&V) -> bool,
    violation_name: &str,
) {
    assert!(
        violations.iter().any(predicate),
        "Expected at least one {violation_name} violation, got: {violations:?}"
    );
}

/// Asserts that NO violation in the list matches a file name pattern.
///
/// Useful for testing exemptions.
///
/// # Panics
/// Panics if any violation's debug representation contains `file_name`.
pub fn assert_no_violation_from_file<V: std::fmt::Debug>(violations: &[V], file_name: &str) {
    for v in violations {
        let msg = format!("{v:?}");
        assert!(
            !msg.contains(file_name),
            "{file_name} should be exempt from this check: {v:?}"
        );
    }
}

/// Asserts that the violations list matches the expected set **exactly**.
///
/// Each expected entry is `(file_suffix, line, msg_contains)`.
/// Fails if any expected violation is missing or if there are unexpected violations.
///
/// # Panics
/// Panics if the violations do not exactly match the expected set.
pub fn assert_violations_exact<V: std::fmt::Debug>(
    violations: &[V],
    expected: &[(&str, usize, &str)],
    context: &str,
) {
    let debug_strs: Vec<String> = violations
        .iter()
        .map(|v| normalize_path(&format!("{v:?}")))
        .collect();

    let missing: Vec<String> = expected
        .iter()
        .filter(|exp| !debug_strs.iter().any(|d| debug_matches_expected(d, exp)))
        .map(|(suffix, line, msg)| format!("  {}:{line} {msg:?}", normalize_path(suffix)))
        .collect();

    // A count mismatch is the only way to have unexpected violations.
    let extras: Vec<String> = if violations.len() == expected.len() {
        Vec::new()
    } else {
        debug_strs
            .iter()
            .enumerate()
            .filter(|(_, d)| !expected.iter().any(|exp| debug_matches_expected(d, exp)))
            .map(|(i, d)| format!("  [{i}] {d}"))
            .collect()
    };

    assert!(
        missing.is_empty() && extras.is_empty(),
        "{}",
        format_violations_failure(context, expected.len(), &debug_strs, &missing, &extras)
    );
}
