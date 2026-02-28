//!
//! **Documentation**: [docs/modules/validate.md](../../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use rust_code_analysis::SpaceKind;

use super::super::violation::NamingViolation;
use crate::ast::rca_helpers;
use crate::traits::violation::Severity;
use crate::utils::naming::is_camel_case;

/// Validates that struct, enum, and trait names follow `CamelCase` convention using RCA AST.
pub fn validate_type_names(path: &Path, content: &str) -> Vec<NamingViolation> {
    let mut violations = Vec::new();

    let Some(root) = rca_helpers::parse_file_spaces(path, content) else {
        return violations;
    };

    // Check struct names
    for space in rca_helpers::collect_spaces_of_kind(&root, content, SpaceKind::Struct) {
        check_camel_case(
            path,
            space.name.as_deref(),
            space.start_line,
            &mut violations,
        );
    }
    // Check trait names
    for space in rca_helpers::collect_spaces_of_kind(&root, content, SpaceKind::Trait) {
        check_camel_case(
            path,
            space.name.as_deref(),
            space.start_line,
            &mut violations,
        );
    }
    // Enums are represented as Class in RCA for Rust
    for space in rca_helpers::collect_spaces_of_kind(&root, content, SpaceKind::Class) {
        check_camel_case(
            path,
            space.name.as_deref(),
            space.start_line,
            &mut violations,
        );
    }

    violations
}

fn check_camel_case(
    path: &Path,
    name: Option<&str>,
    start_line: usize,
    violations: &mut Vec<NamingViolation>,
) {
    let Some(name) = name else { return };
    if name.is_empty() || is_camel_case(name) {
        return;
    }
    violations.push(NamingViolation::BadTypeName {
        file: path.to_path_buf(),
        line: start_line,
        name: name.to_owned(),
        expected_case: "CamelCase".to_owned(),
        severity: Severity::Warning,
    });
}
