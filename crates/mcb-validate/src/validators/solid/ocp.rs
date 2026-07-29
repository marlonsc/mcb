//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use regex::Regex;

use crate::Result;
use crate::Severity;
use crate::ValidationConfig;
use crate::pattern_registry::required_pattern;
use crate::utils::source::{count_match_arms, count_matches_in_block, for_each_rust_file};
use crate::validators::solid::violation::SolidViolation;
use mcb_utils::constants::validate::MIN_STRING_MATCH_ARMS_FOR_DISPATCH;
use mcb_utils::constants::validate::SHORT_PREVIEW_LENGTH;

/// OCP: Check for excessive match statements
///
/// # Errors
/// Returns an error if pattern compilation fails.
pub fn validate_ocp(
    config: &ValidationConfig,
    max_match_arms: usize,
) -> Result<Vec<SolidViolation>> {
    let mut violations = Vec::new();
    let match_pattern = required_pattern("SOLID003.match_keyword")?;

    for_each_rust_file(config, |path, lines| {
        for (line_num, line) in lines.iter().enumerate() {
            if line.trim().starts_with("//") || !match_pattern.is_match(line) {
                continue;
            }
            if let Some(violation) =
                excessive_match_violation(&path, &lines, line_num, max_match_arms)?
            {
                violations.push(violation);
            }
        }
        Ok(())
    })?;

    Ok(violations)
}

/// Returns an `ExcessiveMatchArms` violation when the match at `line_num`
/// exceeds `max_match_arms`, else `None`.
///
/// # Errors
///
/// Returns an error if arm counting fails.
fn excessive_match_violation(
    path: &Path,
    lines: &[&str],
    line_num: usize,
    max_match_arms: usize,
) -> Result<Option<SolidViolation>> {
    let arm_count = count_match_arms(lines, line_num)?;
    Ok(
        (arm_count > max_match_arms).then(|| SolidViolation::ExcessiveMatchArms {
            file: path.to_path_buf(),
            line: line_num + 1,
            arm_count,
            max_recommended: max_match_arms,
            suggestion: "Consider using visitor pattern, enum dispatch, or trait objects"
                .to_owned(),
            severity: Severity::Info,
        }),
    )
}

/// OCP: Check for string-based type dispatch
///
/// # Errors
/// Returns an error if pattern compilation fails.
pub fn validate_string_dispatch(config: &ValidationConfig) -> Result<Vec<SolidViolation>> {
    let mut violations = Vec::new();
    let string_match_pattern = required_pattern("SOLID003.string_match")?;
    let string_arm_pattern = required_pattern("SOLID003.string_arm")?;

    for_each_rust_file(config, |path, lines| {
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || !string_match_pattern.is_match(line) {
                continue;
            }
            if let Some(violation) =
                string_dispatch_violation(&path, &lines, line_num, trimmed, string_arm_pattern)
            {
                violations.push(violation);
            }
        }
        Ok(())
    })?;

    Ok(violations)
}

/// Returns a `StringBasedDispatch` violation when the match at `line_num` has
/// enough string arms to suggest string-based dispatch, else `None`.
fn string_dispatch_violation(
    path: &Path,
    lines: &[&str],
    line_num: usize,
    trimmed: &str,
    string_arm_pattern: &Regex,
) -> Option<SolidViolation> {
    let string_arm_count = count_matches_in_block(lines, line_num, string_arm_pattern);
    (string_arm_count >= MIN_STRING_MATCH_ARMS_FOR_DISPATCH).then(|| {
        SolidViolation::StringBasedDispatch {
            file: path.to_path_buf(),
            line: line_num + 1,
            match_expression: trimmed.chars().take(SHORT_PREVIEW_LENGTH).collect(),
            suggestion: "Consider using enum types with FromStr or a registry pattern".to_owned(),
            severity: Severity::Info,
        }
    })
}
