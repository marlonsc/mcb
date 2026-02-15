use std::path::Path;

use super::super::utils::is_snake_case;
use super::super::violation::NamingViolation;
use crate::traits::violation::Severity;

pub fn validate_module_name(path: &Path) -> Option<NamingViolation> {
    let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let parent_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str());

    // Skip special files
    if file_name == "lib" || file_name == "main" || file_name == "build" {
        return None;
    }

    // Check file name
    if !is_snake_case(file_name) {
        return Some(NamingViolation::BadModuleName {
            path: path.to_path_buf(),
            expected_case: "snake_case".to_owned(),
            severity: Severity::Warning,
        });
    }

    // Check directory name (if mod.rs)
    if file_name == "mod"
        && let Some(dir_name) = parent_name
        && !is_snake_case(dir_name)
    {
        return Some(NamingViolation::BadModuleName {
            path: path.to_path_buf(),
            expected_case: "snake_case".to_owned(),
            severity: Severity::Warning,
        });
    }

    None
}
