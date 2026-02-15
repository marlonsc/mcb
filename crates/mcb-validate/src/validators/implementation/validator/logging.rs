use std::path::PathBuf;

use regex::Regex;

use super::super::violation::ImplementationViolation;
use super::utils::{extract_functions_with_body, non_test_lines, required_patterns};
use crate::Result;
use crate::traits::violation::Severity;

pub fn validate_log_only_methods(
    files: &[(PathBuf, String)],
    fn_pattern: &Regex,
) -> Result<Vec<ImplementationViolation>> {
    let log_pattern_ids = [
        "IMPL001.log_tracing",
        "IMPL001.log_log",
        "IMPL001.log_println",
        "IMPL001.log_eprintln",
    ];

    let compiled_log = required_patterns(log_pattern_ids.iter().copied())?;
    let mut violations = Vec::new();

    for (file_path, content) in files {
        let lines: Vec<&str> = content.lines().collect();
        let non_test = non_test_lines(&lines);
        let mut dummy = String::new();

        for func in extract_functions_with_body(Some(fn_pattern), None, &non_test, &mut dummy) {
            if func.meaningful_body.is_empty() || func.meaningful_body.len() > 3 {
                continue;
            }
            let all_logging = func
                .meaningful_body
                .iter()
                .all(|line| compiled_log.iter().any(|p| p.is_match(line)));
            if all_logging {
                violations.push(ImplementationViolation::LogOnlyMethod {
                    file: file_path.to_path_buf(),
                    line: func.start_line,
                    method_name: func.name.clone(),
                    severity: Severity::Warning,
                });
            }
        }
    }
    Ok(violations)
}
