use std::path::Path;

use regex::Regex;

use super::super::violation::NamingViolation;
use crate::traits::violation::Severity;
use crate::utils::naming::is_camel_case;

pub fn validate_type_names(
    path: &Path,
    content: &str,
    struct_pattern: &Regex,
    enum_pattern: &Regex,
    trait_pattern: &Regex,
) -> Vec<NamingViolation> {
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        // Skip comments
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }

        // Check structs
        if let Some(cap) = struct_pattern.captures(line) {
            let name = cap.get(1).map_or("", |m| m.as_str());
            if !is_camel_case(name) {
                violations.push(NamingViolation::BadTypeName {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    name: name.to_owned(),
                    expected_case: "CamelCase".to_owned(),
                    severity: Severity::Warning,
                });
            }
        }

        // Check enums
        if let Some(cap) = enum_pattern.captures(line) {
            let name = cap.get(1).map_or("", |m| m.as_str());
            if !is_camel_case(name) {
                violations.push(NamingViolation::BadTypeName {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    name: name.to_owned(),
                    expected_case: "CamelCase".to_owned(),
                    severity: Severity::Warning,
                });
            }
        }

        // Check traits
        if let Some(cap) = trait_pattern.captures(line) {
            let name = cap.get(1).map_or("", |m| m.as_str());
            if !is_camel_case(name) {
                violations.push(NamingViolation::BadTypeName {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    name: name.to_owned(),
                    expected_case: "CamelCase".to_owned(),
                    severity: Severity::Warning,
                });
            }
        }
    }
    violations
}
