//!
//! **Documentation**: [docs/modules/validate.md](../../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use rust_code_analysis::SpaceKind;

use super::super::violation::NamingViolation;
use crate::ast::rca_helpers;
use crate::constants::common::TEST_FUNCTION_PREFIX;
use crate::utils::naming::is_snake_case;
use mcb_domain::ports::validation::Severity;

/// Validates that function names follow `snake_case` convention using RCA AST.
pub fn validate_function_names(path: &Path, content: &str) -> Vec<NamingViolation> {
    let mut violations = Vec::new();

    let Some(root) = rca_helpers::parse_file_spaces(path, content) else {
        return violations;
    };

    for space in rca_helpers::collect_spaces_of_kind(&root, content, SpaceKind::Function) {
        let name = space.name.as_deref().unwrap_or("");
        if name.is_empty() || name.starts_with(TEST_FUNCTION_PREFIX) {
            continue;
        }
        if !is_snake_case(name) {
            violations.push(NamingViolation::BadFunctionName {
                file: path.to_path_buf(),
                line: space.start_line,
                name: name.to_owned(),
                expected_case: "snake_case".to_owned(),
                severity: Severity::Warning,
            });
        }
    }
    violations
}
