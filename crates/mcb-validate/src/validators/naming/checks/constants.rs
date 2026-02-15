use std::path::Path;

use regex::Regex;

use super::super::utils::is_screaming_snake_case;
use super::super::violation::NamingViolation;
use crate::traits::violation::Severity;

pub fn validate_constant_names(
    path: &Path,
    content: &str,
    const_pattern: &Regex,
    static_pattern: &Regex,
) -> Vec<NamingViolation> {
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        // Check const
        if let Some(cap) = const_pattern.captures(line) {
            let name = cap.get(1).map_or("", |m| m.as_str());
            if !is_screaming_snake_case(name) {
                violations.push(NamingViolation::BadConstantName {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    name: name.to_owned(),
                    expected_case: "SCREAMING_SNAKE_CASE".to_owned(),
                    severity: Severity::Warning,
                });
            }
        }

        // Check static
        if let Some(cap) = static_pattern.captures(line) {
            let name = cap.get(1).map_or("", |m| m.as_str());
            if !is_screaming_snake_case(name) {
                violations.push(NamingViolation::BadConstantName {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    name: name.to_owned(),
                    expected_case: "SCREAMING_SNAKE_CASE".to_owned(),
                    severity: Severity::Warning,
                });
            }
        }
    }
    violations
}
