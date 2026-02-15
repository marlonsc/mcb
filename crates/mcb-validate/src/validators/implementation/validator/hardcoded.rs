use std::path::PathBuf;

use regex::Regex;

use super::super::violation::ImplementationViolation;
use super::utils::{
    compile_pattern_pairs, extract_functions, is_fn_signature_or_brace, non_test_lines,
};
use crate::Result;
use crate::traits::violation::Severity;
use crate::validators::implementation::constants::{
    HARDCODED_RETURN_PATTERNS, STUB_SKIP_FILE_KEYWORDS,
};

/// Detect hardcoded return values
pub fn validate_hardcoded_returns(
    files: &[(PathBuf, String)],
    fn_pattern: &Regex,
) -> Result<Vec<ImplementationViolation>> {
    let compiled = compile_pattern_pairs(HARDCODED_RETURN_PATTERNS)?;
    let mut violations = Vec::new();

    for (file_path, content) in files {
        // Skip null/fake provider files
        let fname = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if STUB_SKIP_FILE_KEYWORDS.iter().any(|k| fname.contains(k)) {
            continue;
        }

        let lines: Vec<&str> = content.lines().collect();
        let non_test_lines = non_test_lines(&lines);

        for func in extract_functions(Some(fn_pattern), &non_test_lines) {
            if func.has_control_flow {
                continue;
            }
            for line in &func.body_lines {
                if is_fn_signature_or_brace(line) {
                    continue;
                }
                for (pattern, desc) in &compiled {
                    if pattern.is_match(line) {
                        violations.push(ImplementationViolation::HardcodedReturnValue {
                            file: file_path.clone(),
                            line: func.start_line,
                            method_name: func.name.clone(),
                            return_value: desc.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
    }
    Ok(violations)
}
