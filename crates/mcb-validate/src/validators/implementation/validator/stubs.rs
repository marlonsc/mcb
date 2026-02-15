use std::path::PathBuf;

use regex::Regex;

use super::super::violation::ImplementationViolation;
use super::utils::{compile_pattern_pairs, source_lines, track_fn_name};
use crate::Result;
use crate::traits::violation::Severity;

pub fn validate_stub_macros(
    files: &[(PathBuf, String)],
    fn_pattern: &Regex,
) -> Result<Vec<ImplementationViolation>> {
    use crate::constants::STUB_PANIC_LABEL;
    let stub_pattern_ids = [
        ("IMPL001.stub_todo", "todo"),
        ("IMPL001.stub_unimplemented", "unimplemented"),
        ("IMPL001.stub_panic_not_impl", "panic(not implemented)"),
        ("IMPL001.stub_panic_todo", STUB_PANIC_LABEL),
    ];

    let compiled = compile_pattern_pairs(&stub_pattern_ids)?;
    let mut violations = Vec::new();

    for (file_path, content) in files {
        let mut current_fn_name = String::new();
        for (line_num, trimmed) in source_lines(content) {
            track_fn_name(Some(fn_pattern), trimmed, &mut current_fn_name);
            for (pattern, macro_type) in &compiled {
                if pattern.is_match(trimmed) {
                    violations.push(ImplementationViolation::StubMacro {
                        file: file_path.clone(),
                        line: line_num,
                        method_name: current_fn_name.clone(),
                        macro_type: macro_type.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }
    Ok(violations)
}
