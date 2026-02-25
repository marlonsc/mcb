//!
//! **Documentation**: [docs/modules/validate.md](../../../../../../docs/modules/validate.md)
//!
use std::path::PathBuf;

use regex::Regex;

use super::super::violation::ImplementationViolation;
use crate::Result;
use crate::constants::implementation::{HARDCODED_RETURN_PATTERNS, STUB_SKIP_FILE_KEYWORDS};
use crate::traits::violation::Severity;
use crate::utils::source::{
    compile_pattern_pairs, extract_functions, is_fn_signature_or_brace, non_test_lines,
};

/// Detect hardcoded return values
pub fn validate_hardcoded_returns(
    files: &[(PathBuf, String)],
    fn_pattern: &Regex,
) -> Result<Vec<ImplementationViolation>> {
    let compiled = compile_pattern_pairs(HARDCODED_RETURN_PATTERNS)?;
    let mut violations = Vec::new();

    for (file_path, content) in files {
        let fname = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if STUB_SKIP_FILE_KEYWORDS.iter().any(|k| fname.contains(k)) {
            continue;
        }

        let non_test_lines = non_test_lines(&content.lines().collect::<Vec<_>>());

        for func in extract_functions(Some(fn_pattern), &non_test_lines)
            .into_iter()
            .filter(|func| !func.has_control_flow)
        {
            for line in func
                .body_lines
                .iter()
                .filter(|line| !is_fn_signature_or_brace(line))
            {
                for (_pattern, desc) in compiled
                    .iter()
                    .filter(|(pattern, _)| pattern.is_match(line))
                {
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
    Ok(violations)
}
