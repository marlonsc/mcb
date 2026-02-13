use crate::scan::for_each_rs_under_root;
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;

use super::violation::HygieneViolation;

/// Verifies that test functions follow the `test_*` naming pattern.
pub fn validate_test_function_naming(config: &ValidationConfig) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();
    let test_attr_pattern = Regex::new(r"#\[test\]").unwrap();
    let tokio_test_pattern = Regex::new(r"#\[tokio::test\]").unwrap();
    let fn_pattern = Regex::new(r"(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(").unwrap();
    // Standard assertions plus implicit assertions
    let assert_pattern = Regex::new(
        r"assert!|assert_eq!|assert_ne!|panic!|should_panic|\.unwrap\(|\.expect\(|Box<dyn\s|type_name::",
    )
    .unwrap();

    // Smoke test patterns - these verify compilation, not runtime behavior
    let smoke_test_patterns = [
        "_trait_object", // Tests that verify trait object construction compiles
        "_exists",       // Tests that verify types exist
        "_creation",     // Constructor tests with implicit unwrap assertions
        "_compiles",     // Explicit compile-time tests
        "_factory",      // Factory pattern tests (often smoke tests)
    ];

    for crate_dir in config.get_source_dirs()? {
        let tests_dir = crate_dir.join("tests");
        if !tests_dir.exists() {
            continue;
        }

        for_each_rs_under_root(config, &tests_dir, |path| {
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
                                    file: path.to_path_buf(),
                                    line: fn_line_idx + 1,
                                    function_name: fn_name.to_string(),
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
                            let is_smoke_test = smoke_test_patterns
                                .iter()
                                .any(|pattern| fn_name.ends_with(pattern));

                            if !has_assertion && !is_smoke_test {
                                violations.push(HygieneViolation::TestWithoutAssertion {
                                    file: path.to_path_buf(),
                                    line: fn_line_idx + 1,
                                    function_name: fn_name.to_string(),
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
