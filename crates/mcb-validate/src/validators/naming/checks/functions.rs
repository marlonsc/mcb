use std::path::Path;

use regex::Regex;

use super::super::utils::is_snake_case;
use super::super::violation::NamingViolation;
use crate::traits::violation::Severity;

pub fn validate_function_names(
    path: &Path,
    content: &str,
    fn_pattern: &Regex,
) -> Vec<NamingViolation> {
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if let Some(cap) = fn_pattern.captures(line) {
            let name = cap.get(1).map_or("", |m| m.as_str());

            // Skip test functions
            if name.starts_with("test_") {
                continue;
            }

            if !is_snake_case(name) {
                violations.push(NamingViolation::BadFunctionName {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    name: name.to_owned(),
                    expected_case: "snake_case".to_owned(),
                    severity: Severity::Warning,
                });
            }
        }
    }
    violations
}
