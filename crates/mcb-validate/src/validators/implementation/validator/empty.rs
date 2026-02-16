use std::path::PathBuf;

use regex::Regex;

use super::super::violation::ImplementationViolation;
use crate::Result;
use crate::traits::violation::Severity;
use crate::utils::source::{compile_pattern_pairs, source_lines, track_fn_name};

/// Detect empty method bodies
pub fn validate_empty_methods(
    files: &[(PathBuf, String)],
    fn_pattern: &Regex,
) -> Result<Vec<ImplementationViolation>> {
    let empty_pattern_ids = [
        ("IMPL001.empty_ok_unit", "Ok(())"),
        ("IMPL001.empty_none", "None"),
        ("IMPL001.empty_vec_new", "Vec::new()"),
        ("IMPL001.empty_string_new", "String::new()"),
        ("IMPL001.empty_default", "Default::default()"),
        ("IMPL001.empty_ok_vec", "Ok(Vec::new())"),
        ("IMPL001.empty_ok_none", "Ok(None)"),
        ("IMPL001.empty_ok_false", "Ok(false)"),
        ("IMPL001.empty_ok_zero", "Ok(0)"),
    ];

    let compiled = compile_pattern_pairs(&empty_pattern_ids)?;
    let mut violations = Vec::new();

    for (file_path, content) in files {
        // Skip null/fake provider files
        let fname = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if fname.contains("null") || fname.contains("fake") {
            continue;
        }

        let mut current_fn_name = String::new();
        for (line_num, trimmed) in source_lines(content) {
            track_fn_name(Some(fn_pattern), trimmed, &mut current_fn_name);
            for (pattern, desc) in &compiled {
                if pattern.is_match(trimmed) {
                    violations.push(ImplementationViolation::EmptyMethodBody {
                        file: file_path.clone(),
                        line: line_num,
                        method_name: current_fn_name.clone(),
                        pattern: desc.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }
    Ok(violations)
}
