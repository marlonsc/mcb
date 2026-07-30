//!
//! **Documentation**: [docs/modules/validate.md](../../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use regex::Regex;

use super::super::violation::NamingViolation;
use mcb_domain::ports::validation::Severity;
use mcb_utils::utils::naming::is_screaming_snake_case;

/// Validates that every `const` and `static` declaration uses
/// `SCREAMING_SNAKE_CASE`, returning a `BadConstantName` violation for each name
/// that does not.
pub fn validate_constant_names(
    path: &Path,
    content: &str,
    const_pattern: &Regex,
    static_pattern: &Regex,
) -> Vec<NamingViolation> {
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for pattern in [const_pattern, static_pattern] {
            if let Some(violation) = bad_constant_name(path, line, line_num, pattern) {
                violations.push(violation);
            }
        }
    }
    violations
}

/// Returns a `BadConstantName` violation if `pattern` captures a name on `line`
/// that is not `SCREAMING_SNAKE_CASE`, else `None`.
fn bad_constant_name(
    path: &Path,
    line: &str,
    line_num: usize,
    pattern: &Regex,
) -> Option<NamingViolation> {
    let cap = pattern.captures(line)?;
    let name = cap.get(1).map_or("", |m| m.as_str());
    (!is_screaming_snake_case(name)).then(|| NamingViolation::BadConstantName {
        file: path.to_path_buf(),
        line: line_num + 1,
        name: name.to_owned(),
        expected_case: "SCREAMING_SNAKE_CASE".to_owned(),
        severity: Severity::Warning,
    })
}
