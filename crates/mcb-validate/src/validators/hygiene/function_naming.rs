use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_file_under_root;
use crate::{Result, Severity, ValidationConfig};

use super::violation::HygieneViolation;

const SMOKE_TEST_PATTERNS: [&str; 5] = [
    "_trait_object",
    "_exists",
    "_creation",
    "_compiles",
    "_factory",
];

/// Verifies that test functions follow the `test_*` naming pattern.
///
/// # Errors
///
/// Returns an error if source directory enumeration or file reading fails.
pub fn validate_test_function_naming(config: &ValidationConfig) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();
    let test_attr_pattern = compile_regex(r"#\[test\]")?;
    let tokio_test_pattern = compile_regex(r"#\[tokio::test\]")?;
    let fn_pattern = compile_regex(r"(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(")?;
    let assert_pattern = compile_regex(
        r"assert!|assert_eq!|assert_ne!|panic!|should_panic|\.unwrap\(|\.expect\(|Box<dyn\s|type_name::",
    )?;

    for crate_dir in config.get_source_dirs()? {
        let tests_dir = crate_dir.join("tests");
        if !tests_dir.exists() {
            continue;
        }

        for_each_file_under_root(config, &tests_dir, Some(LanguageId::Rust), |entry| {
            let path = &entry.absolute_path;
            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut i = 0;
            while i < lines.len() {
                let line = lines[i];

                // Check for #[test] or #[tokio::test]
                if test_attr_pattern.is_match(line) || tokio_test_pattern.is_match(line) {
                    // Find the function definition
                    let mut fn_line_idx = i + 1;
                    while fn_line_idx < lines.len() {
                        let potential_fn = lines[fn_line_idx];
                        if let Some(cap) = fn_pattern.captures(potential_fn) {
                            let fn_name = cap.get(1).map_or("", |m| m.as_str());

                            // Check naming convention - must start with test_
                            if !fn_name.starts_with("test_") {
                                violations.push(HygieneViolation::BadTestFunctionName {
                                    file: path.clone(),
                                    line: fn_line_idx + 1,
                                    function_name: fn_name.to_owned(),
                                    suggestion: format!("test_{fn_name}"),
                                    severity: Severity::Warning,
                                });
                            }

                            // Check for assertions in the function body
                            let mut has_assertion = false;

                            if let Some((body_lines, _)) =
                                crate::scan::extract_balanced_block(&lines, fn_line_idx)
                            {
                                for check_line in body_lines {
                                    if assert_pattern.is_match(check_line) {
                                        has_assertion = true;
                                        break;
                                    }
                                }
                            }

                            // Skip smoke tests - they verify compilation, not behavior
                            let is_smoke_test = SMOKE_TEST_PATTERNS
                                .iter()
                                .any(|pattern| fn_name.ends_with(pattern));

                            if !has_assertion && !is_smoke_test {
                                violations.push(HygieneViolation::TestWithoutAssertion {
                                    file: path.clone(),
                                    line: fn_line_idx + 1,
                                    function_name: fn_name.to_owned(),
                                    severity: Severity::Warning,
                                });
                            }

                            break;
                        }
                        fn_line_idx += 1;
                    }
                }
                i += 1;
            }
            Ok(())
        })?;
    }

    Ok(violations)
}
