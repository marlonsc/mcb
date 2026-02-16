use super::constants::MIN_STRING_MATCH_ARMS_FOR_DISPATCH;
use crate::Result;
use crate::Severity;
use crate::ValidationConfig;
use crate::constants::common::SHORT_PREVIEW_LENGTH;
use crate::pattern_registry::required_pattern;
use crate::utils::source::{count_match_arms, count_matches_in_block, for_each_rust_file};
use crate::validators::solid::violation::SolidViolation;

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
            if match_pattern.is_match(line) {
                let arm_count = count_match_arms(&lines, line_num)?;

                if arm_count > max_match_arms {
                    violations.push(SolidViolation::ExcessiveMatchArms {
                        file: path.clone(),
                        line: line_num + 1,
                        arm_count,
                        max_recommended: max_match_arms,
                        suggestion:
                            "Consider using visitor pattern, enum dispatch, or trait objects"
                                .to_owned(),
                        severity: Severity::Info,
                    });
                }
            }
        }
        Ok(())
    })?;

    Ok(violations)
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

            if string_match_pattern.is_match(line) {
                let string_arm_count = count_matches_in_block(&lines, line_num, string_arm_pattern);

                if string_arm_count >= MIN_STRING_MATCH_ARMS_FOR_DISPATCH {
                    violations.push(SolidViolation::StringBasedDispatch {
                        file: path.clone(),
                        line: line_num + 1,
                        match_expression: trimmed.chars().take(SHORT_PREVIEW_LENGTH).collect(),
                        suggestion: "Consider using enum types with FromStr or a registry pattern"
                            .to_owned(),
                        severity: Severity::Info,
                    });
                }
            }
        }
        Ok(())
    })?;

    Ok(violations)
}
