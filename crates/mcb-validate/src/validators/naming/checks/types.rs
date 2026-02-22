//!
//! **Documentation**: [docs/modules/validate.md](../../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use regex::Regex;

use super::super::violation::NamingViolation;
use crate::constants::common::COMMENT_PREFIX;
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
    let type_patterns = [struct_pattern, enum_pattern, trait_pattern];

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(COMMENT_PREFIX) {
            continue;
        }

        for pattern in type_patterns {
            if let Some(cap) = pattern.captures(line) {
                let name = cap.get(1).map_or("", |m| m.as_str());
                if is_camel_case(name) {
                    continue;
                }
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
