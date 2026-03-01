//! Generic test assertion helpers.
//!
//! Centralized in `mcb-domain` so all crates share the same assertion patterns
//! and produce consistent error messages.

use std::fmt::Write;

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
    let normalize = |s: &str| {
        s.replace('\\', "/")
            .split('/')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("/")
    };
    let debug_strs: Vec<String> = violations
        .iter()
        .map(|v| normalize(&format!("{v:?}")))
        .collect();

    // Check each expected violation is present
    let mut missing: Vec<String> = Vec::new();
    for (file_suffix, line, msg_contains) in expected {
        let normalized_suffix = normalize(file_suffix);
        let found = debug_strs.iter().any(|d| {
            d.contains(&normalized_suffix)
                && (*line == 0 || d.contains(&format!("line: {line}")))
                && d.contains(msg_contains)
        });
        if !found {
            missing.push(format!("  {normalized_suffix}:{line} {msg_contains:?}"));
        }
    }

    // Check there are no unexpected violations (count mismatch means extras)
    let mut extras: Vec<String> = Vec::new();
    if violations.len() != expected.len() {
        for (i, d) in debug_strs.iter().enumerate() {
            let matched = expected.iter().any(|(file_suffix, line, msg_contains)| {
                let normalized_suffix = normalize(file_suffix);
                d.contains(&normalized_suffix)
                    && (*line == 0 || d.contains(&format!("line: {line}")))
                    && d.contains(msg_contains)
            });
            if !matched {
                extras.push(format!("  [{i}] {d}"));
            }
        }
    }

    let has_issues = !missing.is_empty() || !extras.is_empty();
    if has_issues {
        let mut msg = format!(
            "{}: expected {} violations, got {}\n",
            context,
            expected.len(),
            violations.len()
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
        assert!(!has_issues, "{msg}");
    }
}
